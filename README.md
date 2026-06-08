# diff-tgz

A command-line tool for diffing and patching `.tgz` archives.

- **`show`** — human-readable unified diff of two archives (for use as a git difftool)
- **`diff`** — produce a compact binary [VCDIFF](https://www.rfc-editor.org/rfc/rfc3284) patch between two archives
- **`apply`** — reconstruct an archive from a source archive and a patch

## Installation

```sh
cargo install --path .
```

## Usage

### Show a human-readable diff

```sh
diff-tgz show old.tgz new.tgz
```

Extracts both archives and prints a unified diff of their contents to stdout.
Text files are shown as standard `--- / +++` unified diffs; added/removed files
and binary changes are summarised in a single line each.

Exit codes follow the `diff(1)` convention:

| Exit code | Meaning |
|-----------|---------|
| `0` | Archives are identical |
| `1` | Archives differ |
| `2` | Error (unreadable file, corrupt archive, …) |

### Produce a binary patch

```sh
diff-tgz diff old.tgz new.tgz > patch.vcdiff
```

Writes a binary VCDIFF patch to stdout. The patch is self-verifying: the tool
decodes it immediately and aborts if the output does not match `new.tgz`.

### Apply a binary patch

```sh
diff-tgz apply old.tgz patch.vcdiff > new.tgz
```

Reconstructs `new.tgz` from `old.tgz` and a previously produced patch.

## Git integration

### As a difftool

Add to `~/.gitconfig`:

```ini
[difftool "tgz"]
    cmd = diff-tgz show "$LOCAL" "$REMOTE"
```

Add to `.gitattributes` in your repository:

```
*.tgz    diff=tgz
*.tar.gz diff=tgz
```

Then run:

```sh
git difftool --tool=tgz HEAD~ -- archive.tgz
```

### As an external diff driver

Git's external diff driver receives seven arguments
(`path old-file old-hex old-mode new-file new-hex new-mode`).
Add a small wrapper script to map them:

```sh
#!/bin/sh
# ~/.local/bin/git-diff-tgz
exec diff-tgz show "$2" "$5"
```

```ini
# ~/.gitconfig
[diff "tgz"]
    command = git-diff-tgz
```

## Library

The core logic is exposed as a Rust library crate for use in other tools:

```rust
use workspace::{compute_patch, apply_patch, show_diff};

// Produce a VCDIFF patch
let patch = compute_patch(&old_bytes, &new_bytes)?;

// Apply a patch
let restored = apply_patch(&old_bytes, &patch)?;

// Human-readable diff
let (text, differs) = show_diff(&old_bytes, &new_bytes)?;
```

## Development

```sh
cargo test       # run all tests
cargo build      # debug build  ->  target/debug/diff-tgz
cargo build -r   # release build -> target/release/diff-tgz
```
