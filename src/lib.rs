use std::collections::BTreeMap;
use std::io::Read;

use flate2::read::GzDecoder;
use oxidelta::compress::decoder;
use oxidelta::compress::encoder::{self, CompressOptions};
use similar::TextDiff;

/// Compute a VCDIFF delta from `source` bytes to `target` bytes.
/// Returns the raw patch bytes on success.
pub fn compute_patch(source: &[u8], target: &[u8]) -> Result<Vec<u8>, String> {
    let mut delta: Vec<u8> = Vec::new();
    encoder::encode_all(&mut delta, source, target, CompressOptions::default())
        .map_err(|e| format!("encode error: {e}"))?;

    // Round-trip verification: decoded patch must reproduce target exactly.
    let restored = decoder::decode_all(source, &delta).map_err(|e| format!("decode error: {e}"))?;
    if restored != target {
        return Err("delta round-trip mismatch: decoded output does not match target".into());
    }

    Ok(delta)
}

/// Apply a VCDIFF `patch` to `source` bytes and return the reconstructed target.
pub fn apply_patch(source: &[u8], patch: &[u8]) -> Result<Vec<u8>, String> {
    decoder::decode_all(source, patch).map_err(|e| format!("decode error: {e}"))
}

/// Extract all entries from a `.tgz` blob into a `BTreeMap<path, bytes>`.
fn extract_tgz(data: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let gz = GzDecoder::new(data);
    let mut archive = tar::Archive::new(gz);
    let mut files = BTreeMap::new();
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry
            .path()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned();
        if entry.header().entry_type().is_file() {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            files.insert(path, buf);
        }
    }
    Ok(files)
}

fn is_binary(data: &[u8]) -> bool {
    // Treat any content with a NUL byte in the first 8 KiB as binary.
    data.iter().take(8192).any(|&b| b == 0)
}

/// Compare two `.tgz` archives and return a unified-diff string suitable for
/// display (e.g. as a git difftool) plus a flag indicating whether they differ.
///
/// Returns `(output, differs)`.
pub fn show_diff(old: &[u8], new: &[u8]) -> Result<(String, bool), String> {
    let old_files = extract_tgz(old)?;
    let new_files = extract_tgz(new)?;

    let all_paths: std::collections::BTreeSet<&String> =
        old_files.keys().chain(new_files.keys()).collect();

    let mut output = String::new();
    let mut differs = false;

    for path in all_paths {
        match (old_files.get(path), new_files.get(path)) {
            (None, Some(_)) => {
                use std::fmt::Write;
                writeln!(output, "Added: {path}").unwrap();
                differs = true;
            }
            (Some(_), None) => {
                use std::fmt::Write;
                writeln!(output, "Removed: {path}").unwrap();
                differs = true;
            }
            (Some(a), Some(b)) => {
                if a == b {
                    continue;
                }
                differs = true;
                if is_binary(a) || is_binary(b) {
                    use std::fmt::Write;
                    writeln!(output, "Binary files differ: {path}").unwrap();
                    continue;
                }
                let a_str = String::from_utf8_lossy(a);
                let b_str = String::from_utf8_lossy(b);
                let diff = TextDiff::from_lines(a_str.as_ref(), b_str.as_ref());
                let unified = diff
                    .unified_diff()
                    .header(&format!("a/{path}"), &format!("b/{path}"))
                    .to_string();
                output.push_str(&unified);
            }
            (None, None) => unreachable!(),
        }
    }

    Ok((output, differs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tgz(content: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gz.write_all(content).unwrap();
        gz.finish().unwrap()
    }

    #[test]
    fn patch_identical_blobs_is_valid() {
        let blob = make_tgz(b"file content");
        let delta = compute_patch(&blob, &blob).expect("patch of identical blobs should succeed");
        let restored = apply_patch(&blob, &delta).unwrap();
        assert_eq!(restored, blob);
    }

    #[test]
    fn patch_different_blobs_round_trips() {
        let old = make_tgz(b"hello old world");
        let new = make_tgz(b"hello new world");
        let delta = compute_patch(&old, &new).expect("patch should succeed");
        let restored = apply_patch(&old, &delta).unwrap();
        assert_eq!(restored, new);
    }

    #[test]
    fn patch_from_empty_source() {
        let old: Vec<u8> = Vec::new();
        let new = make_tgz(b"brand new content");
        let delta = compute_patch(&old, &new).expect("patch from empty source should succeed");
        let restored = apply_patch(&old, &delta).unwrap();
        assert_eq!(restored, new);
    }

    #[test]
    fn patch_to_empty_target() {
        let old = make_tgz(b"existing content");
        let new: Vec<u8> = Vec::new();
        let delta = compute_patch(&old, &new).expect("patch to empty target should succeed");
        let restored = apply_patch(&old, &delta).unwrap();
        assert_eq!(restored, new);
    }

    #[test]
    fn patch_large_similar_blobs() {
        let base = "a".repeat(64 * 1024);
        let old = make_tgz(base.as_bytes());
        let mut modified = base.clone();
        modified.push_str(" CHANGED");
        let new = make_tgz(modified.as_bytes());
        let delta = compute_patch(&old, &new).expect("patch of large similar blobs should succeed");
        let restored = apply_patch(&old, &delta).unwrap();
        assert_eq!(restored, new);
    }

    #[test]
    fn patch_binary_content() {
        let old: Vec<u8> = (0u8..=255).collect();
        let new: Vec<u8> = (0u8..=255).rev().collect();
        let delta = compute_patch(&old, &new).expect("patch of binary content should succeed");
        let restored = apply_patch(&old, &delta).unwrap();
        assert_eq!(restored, new);
    }

    #[test]
    fn apply_patch_round_trips() {
        let old = make_tgz(b"hello old world");
        let new = make_tgz(b"hello new world");
        let delta = compute_patch(&old, &new).unwrap();
        let restored = apply_patch(&old, &delta).expect("apply should succeed");
        assert_eq!(restored, new);
    }

    #[test]
    fn apply_patch_invalid_data_returns_error() {
        let old = make_tgz(b"some content");
        let garbage = b"this is not a valid vcdiff patch";
        assert!(apply_patch(&old, garbage).is_err());
    }

    #[test]
    fn diff_then_apply_is_identity() {
        let old = make_tgz(b"version one");
        let new = make_tgz(b"version two");
        let patch = compute_patch(&old, &new).unwrap();
        let result = apply_patch(&old, &patch).unwrap();
        assert_eq!(result, new);
    }

    // --- show_diff tests ---

    /// Build a real tar.gz archive containing a single file at `name` with `content`.
    fn make_archive(files: &[(&str, &[u8])]) -> Vec<u8> {
        let buf = Vec::new();
        let gz = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);
        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, name, *content).unwrap();
        }
        let gz = tar.into_inner().unwrap();
        gz.finish().unwrap()
    }

    #[test]
    fn show_diff_identical_archives_no_diff() {
        let archive = make_archive(&[("a.txt", b"hello\n")]);
        let (output, differs) = show_diff(&archive, &archive).unwrap();
        assert!(!differs);
        assert!(output.is_empty());
    }

    #[test]
    fn show_diff_modified_file_produces_unified_diff() {
        let old = make_archive(&[("a.txt", b"hello\nworld\n")]);
        let new = make_archive(&[("a.txt", b"hello\nrust\n")]);
        let (output, differs) = show_diff(&old, &new).unwrap();
        assert!(differs);
        assert!(output.contains("a/a.txt"), "expected unified diff header");
        assert!(output.contains("-world"), "expected removed line");
        assert!(output.contains("+rust"), "expected added line");
    }

    #[test]
    fn show_diff_added_file() {
        let old = make_archive(&[("a.txt", b"existing\n")]);
        let new = make_archive(&[("a.txt", b"existing\n"), ("b.txt", b"new\n")]);
        let (output, differs) = show_diff(&old, &new).unwrap();
        assert!(differs);
        assert!(output.contains("Added: b.txt"));
    }

    #[test]
    fn show_diff_removed_file() {
        let old = make_archive(&[("a.txt", b"existing\n"), ("b.txt", b"gone\n")]);
        let new = make_archive(&[("a.txt", b"existing\n")]);
        let (output, differs) = show_diff(&old, &new).unwrap();
        assert!(differs);
        assert!(output.contains("Removed: b.txt"));
    }

    #[test]
    fn show_diff_binary_files_differ_notice() {
        let old = make_archive(&[("img.bin", b"\x00\x01\x02")]);
        let new = make_archive(&[("img.bin", b"\x00\xFF\xFE")]);
        let (output, differs) = show_diff(&old, &new).unwrap();
        assert!(differs);
        assert!(output.contains("Binary files differ: img.bin"));
    }
}
