use crate::model::{LicenseCategory, LicenseSource, Report};

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

fn category_meta(category: LicenseCategory) -> (&'static str, &'static str, &'static str) {
    match category {
        LicenseCategory::Permissive => ("Permissive", "permissive", "#2e7d32"),
        LicenseCategory::WeakCopyleft => ("Weak Copyleft", "weak", "#b58900"),
        LicenseCategory::StrongCopyleft => ("Strong Copyleft", "strong", "#c62828"),
        LicenseCategory::Unknown => ("Unknown", "unknown", "#616161"),
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

/// Render a standalone interactive HTML report: summary cards double as
/// category filters, a search box narrows the table, and every column header
/// sorts on click. No external assets; everything is inlined, no emoji.
pub fn render(report: &Report) -> String {
    let total = report.summary.total;
    let summary_cards = [
        ("All", "all", total, "#1565c0"),
        (
            "Permissive",
            "permissive",
            report.summary.permissive,
            "#2e7d32",
        ),
        (
            "Weak Copyleft",
            "weak",
            report.summary.weak_copyleft,
            "#b58900",
        ),
        (
            "Strong Copyleft",
            "strong",
            report.summary.strong_copyleft,
            "#c62828",
        ),
        ("Unknown", "unknown", report.summary.unknown, "#616161"),
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
            let (category_label, category_slug, color) = category_meta(pkg.category);
            format!(
                "<tr data-category=\"{category_slug}\" data-search=\"{search}\">\
                 <td>{name}</td><td>{version}</td><td>{license}</td><td>{ecosystem}</td>\
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
<html lang="en">
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
<h1>dep-lens license report</h1>
<p class="meta">Project: {project} | Scanned: {scanned_at} | Path: {path} | Total packages: {total}</p>
<div class="cards">
{summary_cards}
</div>
<input id="search" type="search" placeholder="Filter by package, license, or category...">
<p id="count"></p>
<table id="pkgs">
<thead><tr>
<th data-col="0">Package <span class="dir"></span></th>
<th data-col="1">Version <span class="dir"></span></th>
<th data-col="2">License <span class="dir"></span></th>
<th data-col="3">Ecosystem <span class="dir"></span></th>
<th data-col="4">Category <span class="dir"></span></th>
<th data-col="5" data-numeric="1">Risk Score <span class="dir"></span></th>
<th data-col="6">Risk Level <span class="dir"></span></th>
<th data-col="7">Commercial Use <span class="dir"></span></th>
<th data-col="8">Source <span class="dir"></span></th>
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
    countEl.textContent = 'Showing ' + shown + ' of ' + rows.length + ' packages';
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
        project = escape(&report.project),
        scanned_at = escape(&report.scanned_at),
        path = escape(&report.path),
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
        let html = render(&sample_report());
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("x&lt;y&gt;&amp;&quot;z&quot;"));
    }

    #[test]
    fn contains_summary_counts_and_packages() {
        let html = render(&sample_report());
        assert!(html.contains("Total packages: 2"));
        assert!(html.contains("gplware"));
        assert!(html.contains("Strong Copyleft"));
        assert!(html.contains("50.0"));
    }

    #[test]
    fn contains_interactive_controls() {
        let html = render(&sample_report());
        assert!(html.contains("id=\"search\""));
        assert!(html.contains("data-filter=\"strong\""));
        assert!(html.contains("data-category=\"strong\""));
        assert!(html.contains("data-numeric=\"1\""));
        assert!(html.contains("<script>\n(function () {"));
    }

    #[test]
    fn shows_license_source() {
        let html = render(&sample_report());
        assert!(html.contains("<td>declared</td>"));
        assert!(html.contains("<td>license file</td>"));
    }

    #[test]
    fn empty_report_renders_zero_percentages() {
        let report = Report {
            project: "empty".to_string(),
            scanned_at: "2026-06-12T00:00:00Z".to_string(),
            path: "/tmp/empty".to_string(),
            summary: Summary::default(),
            packages: vec![],
        };
        let html = render(&report);
        assert!(html.contains("Total packages: 0"));
        assert!(html.contains("0.0%"));
    }
}
