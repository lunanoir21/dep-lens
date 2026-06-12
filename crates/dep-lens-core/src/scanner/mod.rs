pub mod cargo;
pub mod go;
pub mod npm;
pub mod php;
pub mod python;
pub mod ruby;

use std::path::Path;

use crate::license::classifier::identify_license_text;

/// File names probed when looking for a license text on disk. Shared by all
/// ecosystem scanners.
pub(crate) const LICENSE_FILE_NAMES: &[&str] = &[
    "LICENSE",
    "LICENSE.md",
    "LICENSE.txt",
    "LICENCE",
    "LICENCE.md",
    "COPYING",
    "LICENSE-MIT",
    "MIT-LICENSE",
    "UNLICENSE",
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
