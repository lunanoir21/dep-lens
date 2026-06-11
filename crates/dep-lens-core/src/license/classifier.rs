use crate::model::LicenseCategory;

const PERMISSIVE: &[&str] = &[
    "MIT",
    "MIT-0",
    "X11",
    "APACHE-1.1",
    "APACHE-2.0",
    "BSD-2-CLAUSE",
    "BSD-3-CLAUSE",
    "ISC",
    "0BSD",
    "UNLICENSE",
    "CC0",
    "CC0-1.0",
    "ZLIB",
    "BSL-1.0",
    "ARTISTIC-2.0",
    "PYTHON-2.0",
    "PSF-2.0",
    "BLUEOAK-1.0.0",
    "WTFPL",
];

const WEAK_COPYLEFT: &[&str] = &[
    "LGPL-2.0", "LGPL-2.1", "LGPL-3.0", "MPL-1.1", "MPL-2.0", "EPL-1.0", "EPL-2.0", "CDDL-1.0",
    "CDDL-1.1",
];

const STRONG_COPYLEFT: &[&str] = &[
    "GPL-1.0", "GPL-2.0", "GPL-3.0", "AGPL-1.0", "AGPL-3.0", "SSPL-1.0", "EUPL-1.1", "EUPL-1.2",
    "OSL-3.0",
];

/// How restrictive a category is, used to fold SPDX expressions.
/// OR picks the least restrictive operand, AND picks the most restrictive.
fn restrictiveness(category: LicenseCategory) -> u8 {
    match category {
        LicenseCategory::Permissive => 0,
        LicenseCategory::WeakCopyleft => 1,
        LicenseCategory::StrongCopyleft => 2,
        LicenseCategory::Unknown => 3,
    }
}

/// Normalize a single SPDX identifier: uppercase, strip the `+` operator
/// and the `-only` / `-or-later` suffixes so `GPL-3.0-only` matches `GPL-3.0`.
fn normalize(id: &str) -> String {
    let mut s = id.trim().trim_end_matches('+').to_uppercase();
    for suffix in ["-ONLY", "-OR-LATER"] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped.to_string();
        }
    }
    s
}

fn classify_single(id: &str) -> LicenseCategory {
    let normalized = normalize(id);
    if PERMISSIVE.contains(&normalized.as_str()) {
        LicenseCategory::Permissive
    } else if WEAK_COPYLEFT.contains(&normalized.as_str()) {
        LicenseCategory::WeakCopyleft
    } else if STRONG_COPYLEFT.contains(&normalized.as_str()) {
        LicenseCategory::StrongCopyleft
    } else {
        LicenseCategory::Unknown
    }
}

/// Classify a license string, which may be a single SPDX identifier or an
/// SPDX expression (`MIT OR Apache-2.0`, `GPL-2.0 AND MIT`, legacy `MIT/X11`).
/// `WITH` exception clauses are ignored. `None` or empty input is `Unknown`.
pub fn classify(license: Option<&str>) -> LicenseCategory {
    let Some(raw) = license else {
        return LicenseCategory::Unknown;
    };
    // Legacy npm metadata uses `/` as an OR separator; parentheses only group.
    let cleaned = raw.replace(['(', ')'], " ").replace('/', " OR ");
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    if tokens.is_empty() {
        return LicenseCategory::Unknown;
    }

    let mut operands: Vec<LicenseCategory> = Vec::new();
    let mut has_or = false;
    let mut has_and = false;
    let mut skip_next = false;
    for token in &tokens {
        if skip_next {
            skip_next = false;
            continue;
        }
        match token.to_uppercase().as_str() {
            "OR" => has_or = true,
            "AND" => has_and = true,
            "WITH" => skip_next = true,
            _ => operands.push(classify_single(token)),
        }
    }

    if operands.is_empty() {
        return LicenseCategory::Unknown;
    }
    if has_or && !has_and {
        // Dual licensing: the consumer may pick the friendliest option.
        return operands
            .into_iter()
            .min_by_key(|c| restrictiveness(*c))
            .unwrap_or(LicenseCategory::Unknown);
    }
    // AND (or mixed expressions, conservatively): every term applies.
    operands
        .into_iter()
        .max_by_key(|c| restrictiveness(*c))
        .unwrap_or(LicenseCategory::Unknown)
}

/// Identify a license from the text of a LICENSE/COPYING file. Used as a
/// fallback for packages that declare nothing in their manifest. Returns a
/// normalized SPDX identifier. Match order matters: Affero before Lesser
/// before plain GPL, and the generic MIT phrasing last.
pub fn identify_license_text(text: &str) -> Option<&'static str> {
    let head: String = text.chars().take(4096).collect::<String>().to_lowercase();
    if head.contains("gnu affero general public license") {
        return Some("AGPL-3.0");
    }
    if head.contains("gnu lesser general public license") {
        return Some(if head.contains("version 3") {
            "LGPL-3.0"
        } else {
            "LGPL-2.1"
        });
    }
    if head.contains("gnu library general public license") {
        return Some("LGPL-2.0");
    }
    if head.contains("gnu general public license") {
        return Some(if head.contains("version 3") {
            "GPL-3.0"
        } else {
            "GPL-2.0"
        });
    }
    if head.contains("server side public license") {
        return Some("SSPL-1.0");
    }
    if head.contains("mozilla public license") {
        return Some(if head.contains("2.0") {
            "MPL-2.0"
        } else {
            "MPL-1.1"
        });
    }
    if head.contains("apache license") {
        return Some(if head.contains("version 2.0") {
            "Apache-2.0"
        } else {
            "Apache-1.1"
        });
    }
    if head.contains("eclipse public license") {
        return Some(if head.contains("2.0") {
            "EPL-2.0"
        } else {
            "EPL-1.0"
        });
    }
    if head.contains("boost software license") {
        return Some("BSL-1.0");
    }
    if head.contains("this is free and unencumbered software") {
        return Some("Unlicense");
    }
    if head.contains("creative commons zero") || head.contains("cc0 1.0") {
        return Some("CC0-1.0");
    }
    if head.contains("isc license")
        || head.contains("permission to use, copy, modify, and/or distribute")
    {
        return Some("ISC");
    }
    if head.contains("redistribution and use in source and binary forms") {
        return Some(if head.contains("neither the name") {
            "BSD-3-Clause"
        } else {
            "BSD-2-Clause"
        });
    }
    if head.contains("mit license") || head.contains("permission is hereby granted, free of charge")
    {
        return Some("MIT");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_permissive_licenses() {
        for id in [
            "MIT",
            "Apache-2.0",
            "BSD-2-Clause",
            "BSD-3-Clause",
            "ISC",
            "0BSD",
            "Unlicense",
            "CC0-1.0",
        ] {
            assert_eq!(classify(Some(id)), LicenseCategory::Permissive, "{id}");
        }
    }

    #[test]
    fn classifies_weak_copyleft_licenses() {
        for id in ["LGPL-2.0", "LGPL-2.1", "LGPL-3.0", "MPL-2.0", "EPL-2.0"] {
            assert_eq!(classify(Some(id)), LicenseCategory::WeakCopyleft, "{id}");
        }
    }

    #[test]
    fn classifies_strong_copyleft_licenses() {
        for id in ["GPL-2.0", "GPL-3.0", "AGPL-3.0"] {
            assert_eq!(classify(Some(id)), LicenseCategory::StrongCopyleft, "{id}");
        }
    }

    #[test]
    fn unknown_for_missing_or_unrecognized() {
        assert_eq!(classify(None), LicenseCategory::Unknown);
        assert_eq!(classify(Some("")), LicenseCategory::Unknown);
        assert_eq!(classify(Some("   ")), LicenseCategory::Unknown);
        assert_eq!(classify(Some("Proprietary")), LicenseCategory::Unknown);
        assert_eq!(classify(Some("Elastic-2.0")), LicenseCategory::Unknown);
        assert_eq!(
            classify(Some("SEE LICENSE IN LICENSE.txt")),
            LicenseCategory::Unknown
        );
    }

    #[test]
    fn normalizes_spdx_suffixes() {
        assert_eq!(
            classify(Some("GPL-3.0-only")),
            LicenseCategory::StrongCopyleft
        );
        assert_eq!(
            classify(Some("GPL-2.0-or-later")),
            LicenseCategory::StrongCopyleft
        );
        assert_eq!(classify(Some("LGPL-2.1+")), LicenseCategory::WeakCopyleft);
        assert_eq!(classify(Some("mit")), LicenseCategory::Permissive);
    }

    #[test]
    fn or_expression_takes_least_restrictive() {
        assert_eq!(
            classify(Some("MIT OR Apache-2.0")),
            LicenseCategory::Permissive
        );
        assert_eq!(
            classify(Some("GPL-3.0 OR MIT")),
            LicenseCategory::Permissive
        );
        assert_eq!(
            classify(Some("(MIT OR GPL-2.0)")),
            LicenseCategory::Permissive
        );
        assert_eq!(classify(Some("MIT/GPL-2.0")), LicenseCategory::Permissive);
    }

    #[test]
    fn and_expression_takes_most_restrictive() {
        assert_eq!(
            classify(Some("MIT AND GPL-2.0")),
            LicenseCategory::StrongCopyleft
        );
        assert_eq!(
            classify(Some("MIT AND LGPL-3.0")),
            LicenseCategory::WeakCopyleft
        );
    }

    #[test]
    fn with_exception_is_ignored() {
        assert_eq!(
            classify(Some("Apache-2.0 WITH LLVM-exception")),
            LicenseCategory::Permissive
        );
        assert_eq!(
            classify(Some("GPL-2.0 WITH Classpath-exception-2.0")),
            LicenseCategory::StrongCopyleft
        );
    }

    #[test]
    fn classifies_extended_license_set() {
        for id in ["Zlib", "BSL-1.0", "Artistic-2.0", "WTFPL", "MIT-0"] {
            assert_eq!(classify(Some(id)), LicenseCategory::Permissive, "{id}");
        }
        for id in ["MPL-1.1", "EPL-1.0", "CDDL-1.0"] {
            assert_eq!(classify(Some(id)), LicenseCategory::WeakCopyleft, "{id}");
        }
        for id in ["SSPL-1.0", "EUPL-1.2", "OSL-3.0", "GPL-1.0"] {
            assert_eq!(classify(Some(id)), LicenseCategory::StrongCopyleft, "{id}");
        }
    }

    #[test]
    fn identifies_mit_license_text() {
        let text = "MIT License\n\nCopyright (c) 2026\n\nPermission is hereby \
                    granted, free of charge, to any person obtaining a copy...";
        assert_eq!(identify_license_text(text), Some("MIT"));
    }

    #[test]
    fn identifies_gpl_family_texts_in_correct_order() {
        assert_eq!(
            identify_license_text("GNU AFFERO GENERAL PUBLIC LICENSE Version 3"),
            Some("AGPL-3.0")
        );
        assert_eq!(
            identify_license_text("GNU LESSER GENERAL PUBLIC LICENSE Version 2.1"),
            Some("LGPL-2.1")
        );
        assert_eq!(
            identify_license_text("GNU LESSER GENERAL PUBLIC LICENSE Version 3"),
            Some("LGPL-3.0")
        );
        assert_eq!(
            identify_license_text("GNU GENERAL PUBLIC LICENSE Version 2, June 1991"),
            Some("GPL-2.0")
        );
        assert_eq!(
            identify_license_text("GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007"),
            Some("GPL-3.0")
        );
    }

    #[test]
    fn identifies_apache_bsd_and_isc_texts() {
        assert_eq!(
            identify_license_text("Apache License\nVersion 2.0, January 2004"),
            Some("Apache-2.0")
        );
        assert_eq!(
            identify_license_text(
                "Redistribution and use in source and binary forms, with or without \
                 modification, are permitted provided that... Neither the name of..."
            ),
            Some("BSD-3-Clause")
        );
        assert_eq!(
            identify_license_text(
                "Permission to use, copy, modify, and/or distribute this software..."
            ),
            Some("ISC")
        );
    }

    #[test]
    fn unidentifiable_text_returns_none() {
        assert_eq!(
            identify_license_text("All rights reserved. Proprietary."),
            None
        );
        assert_eq!(identify_license_text(""), None);
    }
}
