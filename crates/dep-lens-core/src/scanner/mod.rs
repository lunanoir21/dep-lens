pub mod cargo;
pub mod cpp;
pub mod dart;
pub mod go;
pub mod java;
pub mod npm;
pub mod php;
pub mod python;
pub mod ruby;

use std::path::{Path, PathBuf};

use crate::license::classifier::identify_license_text;

/// User home directory: `HOME` on Linux/macOS, falling back to `USERPROFILE`
/// on Windows where `HOME` is typically unset.
pub(crate) fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}

/// File names probed when looking for a license text on disk. Shared by all
/// ecosystem scanners.
pub(crate) const LICENSE_FILE_NAMES: &[&str] = &[
    "LICENSE",
    "LICENSE.md",
    "LICENSE.txt",
    "LICENSE.rst",
    "LICENCE",
    "LICENCE.md",
    "LICENCE.txt",
    "license",
    "license.md",
    "license.txt",
    "License",
    "License.md",
    "License.txt",
    "COPYING",
    "COPYING.md",
    "COPYING.txt",
    "COPYING.LESSER",
    "LICENSE-MIT",
    "LICENSE-MIT.md",
    "LICENSE-MIT.txt",
    "LICENSE-APACHE",
    "LICENSE-APACHE.md",
    "LICENSE-APACHE.txt",
    "MIT-LICENSE",
    "MIT-LICENSE.txt",
    "UNLICENSE",
    "UNLICENSE.md",
    "UNLICENSE.txt",
];

/// Probe well-known license file names inside `dir` and identify the license
/// from the file contents. Returns a normalized SPDX identifier.
pub(crate) fn detect_license_in_dir(dir: &Path) -> Option<String> {
    for name in LICENSE_FILE_NAMES {
        let path = dir.join(name);
        if !path.is_file() {
            continue;
        }
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Some(id) = identify_license_text(&raw) {
                return Some(id.to_string());
            }
        }
    }
    None
}
