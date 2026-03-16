//! Compare expected constraints (from config) against actual HTML form constraints.

use crate::config::{FieldExpectation, FormConfig};
use crate::model::{FormField, FormInfo, InputType};
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashMap;

/// Full validation report across all pages.
#[derive(Debug, Serialize)]
pub struct ValidationReport {
    /// Results per page.
    pub pages: Vec<PageResult>,
}

/// Validation results for a single page.
#[derive(Debug, Serialize)]
pub struct PageResult {
    /// The URL or file path of the page.
    pub url: String,
    /// Results per form on this page.
    pub form_results: Vec<FormComparisonResult>,
}

/// Comparison results for a single form.
#[derive(Debug, Serialize)]
pub struct FormComparisonResult {
    /// The form identifier from config.
    pub form_id: String,
    /// Whether the form was found in the HTML.
    pub found: bool,
    /// Fields expected in config but missing from HTML.
    pub missing_fields: Vec<String>,
    /// Fields present in HTML but not in config (warnings).
    pub unexpected_fields: Vec<String>,
    /// Per-field constraint comparison results.
    pub field_results: Vec<FieldComparisonResult>,
}

/// Constraint comparison results for a single field.
#[derive(Debug, Serialize)]
pub struct FieldComparisonResult {
    /// The field name.
    pub field_name: String,
    /// Individual constraint check results.
    pub checks: Vec<ConstraintCheck>,
}

/// The result of checking a single constraint.
#[derive(Debug, Serialize)]
pub struct ConstraintCheck {
    /// The constraint being checked (e.g., "type", "required", "maxlength").
    pub constraint: String,
    /// The expected value from config.
    pub expected: String,
    /// The actual value from HTML (if present).
    pub actual: Option<String>,
    /// Whether the check passed.
    pub ok: bool,
}

/// Compare expected forms against actual parsed forms from HTML.
///
/// The `html` parameter is needed for CSS selector matching.
pub fn compare_forms(
    expected_forms: &[FormConfig],
    actual_forms: &[FormInfo],
    html: &str,
) -> Vec<FormComparisonResult> {
    let document = Html::parse_document(html);

    expected_forms
        .iter()
        .map(|expected| {
            let label = expected.display_label();
            if let Some(actual) = find_matching_form(expected, actual_forms, &document) {
                compare_single_form(expected, actual, &label)
            } else {
                FormComparisonResult {
                    form_id: label,
                    found: false,
                    missing_fields: Vec::new(),
                    unexpected_fields: Vec::new(),
                    field_results: Vec::new(),
                }
            }
        })
        .collect()
}

/// Find the form matching the config, using priority: selector → index → id.
fn find_matching_form<'a>(
    config: &FormConfig,
    forms: &'a [FormInfo],
    document: &Html,
) -> Option<&'a FormInfo> {
    if let Some(ref css) = config.selector {
        // Match by CSS selector: find the position of the first matching <form>,
        // then return the FormInfo at that position.
        let form_selector = Selector::parse("form").ok()?;
        let target_selector = Selector::parse(css).ok()?;
        let target_el = document.select(&target_selector).next()?;

        // Find which <form> index this element corresponds to
        for (i, form_el) in document.select(&form_selector).enumerate() {
            if form_el.id() == target_el.id() {
                return forms.get(i);
            }
        }
        None
    } else if let Some(index) = config.index {
        forms.get(index)
    } else if let Some(ref id) = config.id {
        forms.iter().find(|f| f.identifier == *id)
    } else {
        None
    }
}

fn compare_single_form(
    expected: &FormConfig,
    actual: &FormInfo,
    label: &str,
) -> FormComparisonResult {
    let actual_fields: HashMap<&str, &FormField> =
        actual.fields.iter().map(|f| (f.name.as_str(), f)).collect();

    let mut missing_fields = Vec::new();
    let mut field_results = Vec::new();

    // Check each expected field
    for (field_name, expectation) in &expected.fields {
        if let Some(actual_field) = actual_fields.get(field_name.as_str()) {
            let checks = compare_field(expectation, actual_field);
            field_results.push(FieldComparisonResult {
                field_name: field_name.clone(),
                checks,
            });
        } else {
            missing_fields.push(field_name.clone());
        }
    }

    // Find unexpected fields (in HTML but not in config)
    let expected_names: std::collections::HashSet<&str> =
        expected.fields.keys().map(|s| s.as_str()).collect();
    let unexpected_fields: Vec<String> = actual
        .fields
        .iter()
        .filter(|f| !expected_names.contains(f.name.as_str()))
        .map(|f| f.name.clone())
        .collect();

    FormComparisonResult {
        form_id: label.to_string(),
        found: true,
        missing_fields,
        unexpected_fields,
        field_results,
    }
}

fn compare_field(expected: &FieldExpectation, actual: &FormField) -> Vec<ConstraintCheck> {
    let mut checks = Vec::new();

    // type
    if let Some(ref expected_type) = expected.input_type {
        let actual_type = actual.input_type.to_string();
        checks.push(ConstraintCheck {
            constraint: "type".to_string(),
            expected: expected_type.clone(),
            actual: Some(actual_type.clone()),
            ok: InputType::from_attr(expected_type) == actual.input_type,
        });
    }

    // required
    if let Some(expected_required) = expected.required {
        checks.push(ConstraintCheck {
            constraint: "required".to_string(),
            expected: expected_required.to_string(),
            actual: Some(actual.required.to_string()),
            ok: expected_required == actual.required,
        });
    }

    // min
    check_optional_f64(&mut checks, "min", expected.min, actual.min);

    // max
    check_optional_f64(&mut checks, "max", expected.max, actual.max);

    // minlength
    check_optional_usize(
        &mut checks,
        "minlength",
        expected.minlength,
        actual.minlength,
    );

    // maxlength
    check_optional_usize(
        &mut checks,
        "maxlength",
        expected.maxlength,
        actual.maxlength,
    );

    // pattern
    if let Some(ref expected_pattern) = expected.pattern {
        match &actual.pattern {
            Some(actual_pattern) => {
                checks.push(ConstraintCheck {
                    constraint: "pattern".to_string(),
                    expected: expected_pattern.clone(),
                    actual: Some(actual_pattern.clone()),
                    ok: expected_pattern == actual_pattern,
                });
            }
            None => {
                checks.push(ConstraintCheck {
                    constraint: "pattern".to_string(),
                    expected: expected_pattern.clone(),
                    actual: None,
                    ok: false,
                });
            }
        }
    }

    // step
    check_optional_f64(&mut checks, "step", expected.step, actual.step);

    checks
}

fn check_optional_f64(
    checks: &mut Vec<ConstraintCheck>,
    name: &str,
    expected: Option<f64>,
    actual: Option<f64>,
) {
    if let Some(exp) = expected {
        match actual {
            Some(act) => {
                checks.push(ConstraintCheck {
                    constraint: name.to_string(),
                    expected: format_number(exp),
                    actual: Some(format_number(act)),
                    ok: (exp - act).abs() < 1e-9,
                });
            }
            None => {
                checks.push(ConstraintCheck {
                    constraint: name.to_string(),
                    expected: format_number(exp),
                    actual: None,
                    ok: false,
                });
            }
        }
    }
}

fn check_optional_usize(
    checks: &mut Vec<ConstraintCheck>,
    name: &str,
    expected: Option<usize>,
    actual: Option<usize>,
) {
    if let Some(exp) = expected {
        match actual {
            Some(act) => {
                checks.push(ConstraintCheck {
                    constraint: name.to_string(),
                    expected: exp.to_string(),
                    actual: Some(act.to_string()),
                    ok: exp == act,
                });
            }
            None => {
                checks.push(ConstraintCheck {
                    constraint: name.to_string(),
                    expected: exp.to_string(),
                    actual: None,
                    ok: false,
                });
            }
        }
    }
}

/// Format a number, removing trailing `.0` for integers.
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Returns true if the entire validation report has no errors.
/// Unexpected fields are warnings and do not count as errors.
pub fn report_has_errors(report: &ValidationReport) -> bool {
    for page in &report.pages {
        for form in &page.form_results {
            if !form.found {
                return true;
            }
            if !form.missing_fields.is_empty() {
                return true;
            }
            for field in &form.field_results {
                if field.checks.iter().any(|c| !c.ok) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FieldExpectation;
    use indexmap::IndexMap;

    const STUB_HTML: &str = r#"<html><body>
        <form name="f"><input type="email" name="email" required /></form>
    </body></html>"#;

    fn id_config(id: &str, fields: IndexMap<String, FieldExpectation>) -> FormConfig {
        FormConfig {
            id: Some(id.to_string()),
            selector: None,
            index: None,
            fields,
        }
    }

    fn make_actual_field(
        name: &str,
        input_type: InputType,
        required: bool,
        maxlength: Option<usize>,
    ) -> FormField {
        FormField {
            name: name.to_string(),
            input_type,
            required,
            min: None,
            max: None,
            minlength: None,
            maxlength,
            pattern: None,
            step: None,
        }
    }

    fn make_form(id: &str, fields: Vec<FormField>) -> FormInfo {
        FormInfo {
            identifier: id.to_string(),
            index: 0,
            fields,
        }
    }

    #[test]
    fn test_all_constraints_match() {
        let mut fields = IndexMap::new();
        fields.insert(
            "email".to_string(),
            FieldExpectation {
                input_type: Some("email".to_string()),
                required: Some(true),
                ..Default::default()
            },
        );
        let expected = id_config("f", fields);
        let actual = make_form(
            "f",
            vec![make_actual_field("email", InputType::Email, true, None)],
        );

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        assert!(results[0].found);
        assert!(results[0].missing_fields.is_empty());
        assert!(results[0].field_results[0].checks.iter().all(|c| c.ok));
    }

    #[test]
    fn test_constraint_mismatch() {
        let mut fields = IndexMap::new();
        fields.insert(
            "username".to_string(),
            FieldExpectation {
                maxlength: Some(50),
                ..Default::default()
            },
        );
        let expected = id_config("f", fields);
        let actual = make_form(
            "f",
            vec![make_actual_field(
                "username",
                InputType::Text,
                false,
                Some(100),
            )],
        );

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        let check = &results[0].field_results[0].checks[0];
        assert!(!check.ok);
        assert_eq!(check.expected, "50");
        assert_eq!(check.actual.as_deref(), Some("100"));
    }

    #[test]
    fn test_missing_constraint_in_html() {
        let mut fields = IndexMap::new();
        fields.insert(
            "password".to_string(),
            FieldExpectation {
                minlength: Some(10),
                ..Default::default()
            },
        );
        let expected = id_config("f", fields);
        let actual = make_form(
            "f",
            vec![make_actual_field(
                "password",
                InputType::Password,
                false,
                None,
            )],
        );

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        let check = &results[0].field_results[0].checks[0];
        assert!(!check.ok);
        assert_eq!(check.constraint, "minlength");
        assert!(check.actual.is_none());
    }

    #[test]
    fn test_missing_field_in_html() {
        let mut fields = IndexMap::new();
        fields.insert(
            "nonexistent".to_string(),
            FieldExpectation {
                required: Some(true),
                ..Default::default()
            },
        );
        let expected = id_config("f", fields);
        let actual = make_form("f", vec![]);

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        assert_eq!(results[0].missing_fields, vec!["nonexistent"]);
    }

    #[test]
    fn test_unexpected_field_in_html() {
        let expected = id_config("f", IndexMap::new());
        let actual = make_form(
            "f",
            vec![make_actual_field("surprise", InputType::Text, false, None)],
        );

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        assert_eq!(results[0].unexpected_fields, vec!["surprise"]);
    }

    #[test]
    fn test_form_not_found() {
        let expected = id_config("missing_form", IndexMap::new());
        let results = compare_forms(&[expected], &[], STUB_HTML);
        assert!(!results[0].found);
    }

    #[test]
    fn test_required_false_mismatch() {
        let mut fields = IndexMap::new();
        fields.insert(
            "website".to_string(),
            FieldExpectation {
                required: Some(false),
                ..Default::default()
            },
        );
        let expected = id_config("f", fields);
        let actual = make_form(
            "f",
            vec![make_actual_field("website", InputType::Url, true, None)],
        );

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        let check = &results[0].field_results[0].checks[0];
        assert!(!check.ok);
        assert_eq!(check.expected, "false");
        assert_eq!(check.actual.as_deref(), Some("true"));
    }

    #[test]
    fn test_match_by_index() {
        let mut fields = IndexMap::new();
        fields.insert(
            "email".to_string(),
            FieldExpectation {
                input_type: Some("email".to_string()),
                ..Default::default()
            },
        );
        let expected = FormConfig {
            id: None,
            selector: None,
            index: Some(0),
            fields,
        };
        let actual = make_form(
            "unnamed",
            vec![make_actual_field("email", InputType::Email, true, None)],
        );

        let results = compare_forms(&[expected], &[actual], STUB_HTML);
        assert!(results[0].found);
        assert!(results[0].field_results[0].checks.iter().all(|c| c.ok));
    }

    #[test]
    fn test_match_by_selector() {
        let html = r#"<html><body>
            <form class="login-form"><input type="email" name="email" required /></form>
        </body></html>"#;
        let forms = crate::parser::parse_forms(html);

        let mut fields = IndexMap::new();
        fields.insert(
            "email".to_string(),
            FieldExpectation {
                input_type: Some("email".to_string()),
                required: Some(true),
                ..Default::default()
            },
        );
        let expected = FormConfig {
            id: None,
            selector: Some("form.login-form".to_string()),
            index: None,
            fields,
        };

        let results = compare_forms(&[expected], &forms, html);
        assert!(results[0].found);
        assert!(results[0].field_results[0].checks.iter().all(|c| c.ok));
    }
}
