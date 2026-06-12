use crate::model::{DependencyType, LicenseCategory, LicenseSource, Report};

struct Labels {
    title: &'static str,
    project: &'static str,
    scanned: &'static str,
    path: &'static str,
    total: &'static str,
    search_placeholder: &'static str,
    showing: &'static str,
    of: &'static str,
    packages: &'static str,
    col_package: &'static str,
    col_version: &'static str,
    col_license: &'static str,
    col_ecosystem: &'static str,
    col_type: &'static str,
    col_category: &'static str,
    col_risk_score: &'static str,
    col_risk_level: &'static str,
    col_commercial: &'static str,
    col_source: &'static str,
    cat_all: &'static str,
    cat_permissive: &'static str,
    cat_weak: &'static str,
    cat_strong: &'static str,
    cat_unknown: &'static str,
}

const EN: Labels = Labels {
    title: "dep-lens license report",
    project: "Project",
    scanned: "Scanned",
    path: "Path",
    total: "Total packages",
    search_placeholder: "Filter by package, license, or category...",
    showing: "Showing",
    of: "of",
    packages: "packages",
    col_package: "Package",
    col_version: "Version",
    col_license: "License",
    col_ecosystem: "Ecosystem",
    col_type: "Type",
    col_category: "Category",
    col_risk_score: "Risk Score",
    col_risk_level: "Risk Level",
    col_commercial: "Commercial Use",
    col_source: "Source",
    cat_all: "All",
    cat_permissive: "Permissive",
    cat_weak: "Weak Copyleft",
    cat_strong: "Strong Copyleft",
    cat_unknown: "Unknown",
};

const TR: Labels = Labels {
    title: "dep-lens lisans raporu",
    project: "Proje",
    scanned: "Tarama Tarihi",
    path: "Dizin",
    total: "Toplam paket",
    search_placeholder: "Paket, lisans veya kategoriye göre filtrele...",
    showing: "Gösterilen:",
    of: "/",
    packages: "paket",
    col_package: "Paket",
    col_version: "Sürüm",
    col_license: "Lisans",
    col_ecosystem: "Ekosistem",
    col_type: "Tür",
    col_category: "Kategori",
    col_risk_score: "Risk Puanı",
    col_risk_level: "Risk Seviyesi",
    col_commercial: "Ticari Kullanım",
    col_source: "Kaynak",
    cat_all: "Tümü",
    cat_permissive: "İzinli",
    cat_weak: "Zayıf Copyleft",
    cat_strong: "Güçlü Copyleft",
    cat_unknown: "Bilinmeyen",
};

/// Escape text for safe interpolation into HTML.
fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn category_meta(
    category: LicenseCategory,
    labels: &Labels,
) -> (&'static str, &'static str, &'static str) {
    match category {
        LicenseCategory::Permissive => (labels.cat_permissive, "permissive", "#2e7d32"),
        LicenseCategory::WeakCopyleft => (labels.cat_weak, "weak", "#b58900"),
        LicenseCategory::StrongCopyleft => (labels.cat_strong, "strong", "#c62828"),
        LicenseCategory::Unknown => (labels.cat_unknown, "unknown", "#616161"),
    }
}

fn source_label(source: LicenseSource) -> &'static str {
    match source {
        LicenseSource::Declared => "declared",
        LicenseSource::LicenseFile => "license file",
        LicenseSource::None => "none",
    }
}

fn percent(part: usize, total: usize) -> String {
    if total == 0 {
        return "0.0".to_string();
    }
    format!("{:.1}", part as f64 * 100.0 / total as f64)
}

/// Render a standalone interactive HTML report.
pub fn render(report: &Report, lang: &str) -> String {
    let labels = if lang == "tr" { &TR } else { &EN };
    let total = report.summary.total;

    let summary_cards = [
        (labels.cat_all, "all", total, "#1565c0"),
        (
            labels.cat_permissive,
            "permissive",
            report.summary.permissive,
            "#2e7d32",
        ),
        (
            labels.cat_weak,
            "weak",
            report.summary.weak_copyleft,
            "#b58900",
        ),
        (
            labels.cat_strong,
            "strong",
            report.summary.strong_copyleft,
            "#c62828",
        ),
        (
            labels.cat_unknown,
            "unknown",
            report.summary.unknown,
            "#616161",
        ),
    ]
    .iter()
    .map(|(label, slug, count, color)| {
        format!(
            "<button class=\"card\" data-filter=\"{slug}\" style=\"border-top: 4px solid {color}\">\
             <span class=\"card-count\">{count}</span>\
             <span class=\"card-label\">{label}</span>\
             <span class=\"card-pct\">{}%</span></button>",
            percent(*count, total)
        )
    })
    .collect::<Vec<_>>()
    .join("\n");

    let rows = report
        .packages
        .iter()
        .map(|pkg| {
            let (category_label, category_slug, color) = category_meta(pkg.category, labels);
            let type_label = match pkg.dependency_type {
                DependencyType::Direct => "direct",
                DependencyType::Transitive => "transitive",
            };
            format!(
                "<tr data-category=\"{category_slug}\" data-search=\"{search}\">\
                 <td>{name}</td><td>{version}</td><td>{license}</td><td>{ecosystem}</td>\
                 <td>{dep_type}</td>\
                 <td><span class=\"badge\" style=\"background:{color}\">{category_label}</span></td>\
                 <td class=\"num\" data-value=\"{score}\">{score}</td>\
                 <td>{level:?}</td><td>{commercial:?}</td><td>{source}</td></tr>",
                search = escape(&format!(
                    "{} {} {}",
                    pkg.name.to_lowercase(),
                    pkg.license.to_lowercase(),
                    category_label.to_lowercase()
                )),
                name = escape(&pkg.name),
                version = escape(&pkg.version),
                license = escape(&pkg.license),
                ecosystem = format!("{:?}", pkg.ecosystem).to_lowercase(),
                dep_type = type_label,
                score = pkg.risk_score,
                level = pkg.risk_level,
                commercial = pkg.commercial_use,
                source = source_label(pkg.license_source),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>dep-lens report - {project}</title>
<style>
  body {{ font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; margin: 2rem; color: #1c1c1c; background: #fafafa; }}
  h1 {{ font-size: 1.5rem; margin-bottom: 0.25rem; }}
  .meta {{ color: #616161; margin-bottom: 1.5rem; }}
  .cards {{ display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.5rem; }}
  .card {{ background: #fff; border: 1px solid #e0e0e0; border-radius: 6px; padding: 1rem 1.5rem; min-width: 9rem; cursor: pointer; font: inherit; text-align: left; display: flex; flex-direction: column; }}
  .card.active {{ outline: 2px solid #1565c0; }}
  .card-count {{ font-size: 2rem; font-weight: 700; }}
  .card-label {{ color: #616161; }}
  .card-pct {{ color: #9e9e9e; font-size: 0.875rem; }}
  #search {{ width: 100%; max-width: 24rem; padding: 0.5rem 0.75rem; margin-bottom: 1rem; border: 1px solid #bdbdbd; border-radius: 6px; font: inherit; }}
  table {{ border-collapse: collapse; width: 100%; background: #fff; border: 1px solid #e0e0e0; }}
  th, td {{ text-align: left; padding: 0.5rem 0.75rem; border-bottom: 1px solid #eee; font-size: 0.875rem; }}
  th {{ background: #f5f5f5; position: sticky; top: 0; cursor: pointer; user-select: none; }}
  th .dir {{ color: #1565c0; }}
  td.num {{ text-align: right; font-variant-numeric: tabular-nums; }}
  .badge {{ color: #fff; border-radius: 4px; padding: 0.125rem 0.5rem; font-size: 0.75rem; white-space: nowrap; }}
  #count {{ color: #616161; margin: 0.5rem 0; }}
</style>
</head>
<body>
<h1>{title}</h1>
<p class="meta">{lbl_project}: {project} | {lbl_scanned}: {scanned_at} | {lbl_path}: {path} | {lbl_total}: {total}</p>
<div class="cards">
{summary_cards}
</div>
<input id="search" type="search" placeholder="{search_placeholder}">
<p id="count"></p>
<table id="pkgs">
<thead><tr>
<th data-col="0">{col_package} <span class="dir"></span></th>
<th data-col="1">{col_version} <span class="dir"></span></th>
<th data-col="2">{col_license} <span class="dir"></span></th>
<th data-col="3">{col_ecosystem} <span class="dir"></span></th>
<th data-col="4">{col_type} <span class="dir"></span></th>
<th data-col="5">{col_category} <span class="dir"></span></th>
<th data-col="6" data-numeric="1">{col_risk_score} <span class="dir"></span></th>
<th data-col="7">{col_risk_level} <span class="dir"></span></th>
<th data-col="8">{col_commercial} <span class="dir"></span></th>
<th data-col="9">{col_source} <span class="dir"></span></th>
</tr></thead>
<tbody>
{rows}
</tbody>
</table>
<script>
(function () {{
  var search = document.getElementById('search');
  var tbody = document.querySelector('#pkgs tbody');
  var countEl = document.getElementById('count');
  var cards = Array.prototype.slice.call(document.querySelectorAll('.card'));
  var activeFilter = 'all';

  function apply() {{
    var query = search.value.trim().toLowerCase();
    var rows = Array.prototype.slice.call(tbody.rows);
    var shown = 0;
    rows.forEach(function (row) {{
      var matchesText = query === '' || row.getAttribute('data-search').indexOf(query) !== -1;
      var matchesCat = activeFilter === 'all' || row.getAttribute('data-category') === activeFilter;
      var visible = matchesText && matchesCat;
      row.style.display = visible ? '' : 'none';
      if (visible) shown += 1;
    }});
    countEl.textContent = '{showing} ' + shown + ' {of} ' + rows.length + ' {packages}';
  }}

  cards.forEach(function (card) {{
    card.addEventListener('click', function () {{
      activeFilter = card.getAttribute('data-filter');
      cards.forEach(function (c) {{ c.classList.toggle('active', c === card); }});
      apply();
    }});
  }});

  search.addEventListener('input', apply);

  var sortCol = -1;
  var sortAsc = true;
  Array.prototype.slice.call(document.querySelectorAll('#pkgs th')).forEach(function (th) {{
    th.addEventListener('click', function () {{
      var col = parseInt(th.getAttribute('data-col'), 10);
      sortAsc = col === sortCol ? !sortAsc : true;
      sortCol = col;
      var numeric = th.getAttribute('data-numeric') === '1';
      var rows = Array.prototype.slice.call(tbody.rows);
      rows.sort(function (a, b) {{
        var av = numeric ? parseFloat(a.cells[col].getAttribute('data-value')) : a.cells[col].textContent.toLowerCase();
        var bv = numeric ? parseFloat(b.cells[col].getAttribute('data-value')) : b.cells[col].textContent.toLowerCase();
        if (av < bv) return sortAsc ? -1 : 1;
        if (av > bv) return sortAsc ? 1 : -1;
        return 0;
      }});
      rows.forEach(function (row) {{ tbody.appendChild(row); }});
      document.querySelectorAll('#pkgs th .dir').forEach(function (el) {{ el.textContent = ''; }});
      th.querySelector('.dir').textContent = sortAsc ? '^' : 'v';
    }});
  }});

  cards[0].classList.add('active');
  apply();
}})();
</script>
</body>
</html>
"#,
        lang = lang,
        project = escape(&report.project),
        scanned_at = escape(&report.scanned_at),
        path = escape(&report.path),
        title = labels.title,
        lbl_project = labels.project,
        lbl_scanned = labels.scanned,
        lbl_path = labels.path,
        lbl_total = labels.total,
        search_placeholder = labels.search_placeholder,
        showing = labels.showing,
        of = labels.of,
        packages = labels.packages,
        col_package = labels.col_package,
        col_version = labels.col_version,
        col_license = labels.col_license,
        col_ecosystem = labels.col_ecosystem,
        col_type = labels.col_type,
        col_category = labels.col_category,
        col_risk_score = labels.col_risk_score,
        col_risk_level = labels.col_risk_level,
        col_commercial = labels.col_commercial,
        col_source = labels.col_source,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn sample_report() -> Report {
        Report {
            project: "x<y>&\"z\"".to_string(),
            scanned_at: "2026-06-12T00:00:00Z".to_string(),
            path: "/tmp/x".to_string(),
            summary: Summary {
                total: 2,
                permissive: 1,
                weak_copyleft: 0,
                strong_copyleft: 1,
                unknown: 0,
            },
            packages: vec![
                ClassifiedPackage {
                    name: "<script>alert(1)</script>".to_string(),
                    version: "1.0.0".to_string(),
                    license: "MIT".to_string(),
                    license_source: LicenseSource::Declared,
                    ecosystem: Ecosystem::Npm,
                    dependency_type: DependencyType::Direct,
                    category: LicenseCategory::Permissive,
                    risk_score: 0,
                    risk_level: RiskLevel::Low,
                    commercial_use: CommercialUse::Yes,
                },
                ClassifiedPackage {
                    name: "gplware".to_string(),
                    version: "2.0.0".to_string(),
                    license: "GPL-3.0".to_string(),
                    license_source: LicenseSource::LicenseFile,
                    ecosystem: Ecosystem::Cargo,
                    dependency_type: DependencyType::Transitive,
                    category: LicenseCategory::StrongCopyleft,
                    risk_score: 90,
                    risk_level: RiskLevel::High,
                    commercial_use: CommercialUse::Restricted,
                },
            ],
        }
    }

    #[test]
    fn escapes_html_in_package_names_and_project() {
        let html = render(&sample_report(), "en");
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("x&lt;y&gt;&amp;&quot;z&quot;"));
    }

    #[test]
    fn contains_summary_counts_and_packages() {
        let html = render(&sample_report(), "en");
        assert!(html.contains("Total packages: 2"));
        assert!(html.contains("gplware"));
        assert!(html.contains("Strong Copyleft"));
        assert!(html.contains("50.0"));
    }

    #[test]
    fn supports_turkish_localization() {
        let html = render(&sample_report(), "tr");
        assert!(html.contains("dep-lens lisans raporu"));
        assert!(html.contains("Toplam paket: 2"));
    }
}
