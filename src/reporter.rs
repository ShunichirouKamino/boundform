//! Output results in various formats (terminal, JSON).

use colored::Colorize;

/// Output format for reports.
pub enum OutputFormat {
    /// Human-readable colored terminal output.
    Terminal,
    /// JSON output for programmatic consumption.
    Json,
}

// ── Validation report rendering ──────────────────────────────────────

use crate::comparator::ValidationReport;

/// Render a validation report to a string in the specified format.
pub fn render_validation_report(report: &ValidationReport, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Terminal => render_validation_terminal(report),
        OutputFormat::Json => render_validation_json(report),
    }
}

fn render_validation_terminal(report: &ValidationReport) -> String {
    let mut out = String::new();
    let mut total_ok: usize = 0;
    let mut total_mismatch: usize = 0;
    let mut total_warnings: usize = 0;
    let mut total_pages: usize = 0;
    let mut total_forms: usize = 0;

    for page in &report.pages {
        total_pages += 1;

        for form in &page.form_results {
            total_forms += 1;

            let header = format!("[{}] {}\n", form.form_id, page.url);
            out.push_str(&header.bold().to_string());

            if !form.found {
                let msg = "  form not found in HTML\n";
                out.push_str(&msg.red().to_string());
                total_mismatch += 1;
                out.push('\n');
                continue;
            }

            // Field results
            for field in &form.field_results {
                out.push_str(&format!("  {}\n", field.field_name));
                for check in &field.checks {
                    if check.ok {
                        total_ok += 1;
                        let line = format!("    ✓ {} = {}\n", check.constraint, check.expected);
                        out.push_str(&line.green().to_string());
                    } else {
                        total_mismatch += 1;
                        let actual_str = check
                            .actual
                            .as_deref()
                            .map(|a| format!("actual {a}"))
                            .unwrap_or_else(|| "not present in HTML".to_string());
                        let line = format!(
                            "    ✗ {}: expected {}, {}\n",
                            check.constraint, check.expected, actual_str
                        );
                        out.push_str(&line.red().to_string());
                    }
                }
            }

            // Missing fields
            if !form.missing_fields.is_empty() {
                total_mismatch += form.missing_fields.len();
                let line = format!("\n  Missing fields: {}\n", form.missing_fields.join(", "));
                out.push_str(&line.red().to_string());
            }

            // Unexpected fields (warnings)
            if !form.unexpected_fields.is_empty() {
                total_warnings += form.unexpected_fields.len();
                let line = format!(
                    "  Unexpected fields: {}\n",
                    form.unexpected_fields.join(", ")
                );
                out.push_str(&line.yellow().to_string());
            }

            out.push('\n');
        }
    }

    // Summary
    let summary = if total_mismatch == 0 {
        let mut s =
            format!("{total_pages} page(s), {total_forms} form(s), all {total_ok} checks passed");
        if total_warnings > 0 {
            s.push_str(&format!(", {total_warnings} warning(s)"));
        }
        s.green().bold().to_string()
    } else {
        let mut s =
            format!("{total_pages} page(s), {total_forms} form(s), {total_mismatch} mismatch(es)");
        if total_warnings > 0 {
            s.push_str(&format!(", {total_warnings} warning(s)"));
        }
        s.red().bold().to_string()
    };
    out.push_str(&summary);
    out.push('\n');

    out
}

fn render_validation_json(report: &ValidationReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}
