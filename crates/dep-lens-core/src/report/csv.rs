use crate::model::Report;

/// Render a CSV report.
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("Package,Version,License,Ecosystem,Type,Category,Risk Score,Risk Level,Commercial Use,Source\n");
    for pkg in &report.packages {
        out.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{:?}\",\"{:?}\",\"{:?}\",{},\"{:?}\",\"{:?}\",\"{:?}\"\n",
            pkg.name,
            pkg.version,
            pkg.license,
            pkg.ecosystem,
            pkg.dependency_type,
            pkg.category,
            pkg.risk_score,
            pkg.risk_level,
            pkg.commercial_use,
            pkg.license_source
        ));
    }
    out
}
