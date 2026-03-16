//! Parse HTML to extract form fields and their constraint attributes.

use crate::model::{FormField, FormInfo, InputType};
use scraper::{Html, Selector};

/// Parse HTML and extract all forms with their fields and constraints.
pub fn parse_forms(html: &str) -> Vec<FormInfo> {
    let document = Html::parse_document(html);
    let form_selector = Selector::parse("form").expect("valid selector");

    let mut forms = Vec::new();

    for (index, form_el) in document.select(&form_selector).enumerate() {
        let identifier = form_el
            .value()
            .attr("name")
            .or_else(|| form_el.value().attr("id"))
            .or_else(|| form_el.value().attr("action"))
            .unwrap_or("unnamed")
            .to_string();

        let fields = extract_fields_from_element(&form_el);
        forms.push(FormInfo {
            identifier,
            index,
            fields,
        });
    }

    forms
}

/// Extract form fields from a parent element (a `<form>`).
fn extract_fields_from_element(element: &scraper::ElementRef) -> Vec<FormField> {
    let mut fields = Vec::new();

    // Extract <input> elements
    let input_selector = Selector::parse("input").expect("valid selector");
    for input in element.select(&input_selector) {
        let el = input.value();
        let input_type = InputType::from_attr(el.attr("type").unwrap_or("text"));

        // Skip hidden, checkbox, radio, and submit-like inputs
        match input_type {
            InputType::Hidden | InputType::Checkbox | InputType::Radio => continue,
            InputType::Other(ref t)
                if t == "submit" || t == "button" || t == "reset" || t == "image" =>
            {
                continue;
            }
            _ => {}
        }

        let name = resolve_field_name(el);
        if name.is_empty() {
            continue;
        }

        fields.push(build_field(name, input_type, el));
    }

    // Extract <textarea> elements
    let textarea_selector = Selector::parse("textarea").expect("valid selector");
    for textarea in element.select(&textarea_selector) {
        let el = textarea.value();
        let name = resolve_field_name(el);
        if name.is_empty() {
            continue;
        }
        fields.push(build_field(name, InputType::Textarea, el));
    }

    // Extract <select> elements
    let select_selector = Selector::parse("select").expect("valid selector");
    for select in element.select(&select_selector) {
        let el = select.value();
        let name = resolve_field_name(el);
        if name.is_empty() {
            continue;
        }
        fields.push(FormField {
            name,
            input_type: InputType::Select,
            required: el.attr("required").is_some(),
            min: None,
            max: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            step: None,
        });
    }

    fields
}

/// Resolve the field name from `name` attribute, falling back to `id`.
///
/// React/state-managed forms often omit `name` but set `id` on inputs.
/// This fallback ensures those fields are still visible to boundform.
fn resolve_field_name(el: &scraper::node::Element) -> String {
    el.attr("name")
        .or_else(|| el.attr("id"))
        .unwrap_or("")
        .to_string()
}

/// Build a `FormField` from an HTML element's attributes.
fn build_field(name: String, input_type: InputType, el: &scraper::node::Element) -> FormField {
    let required = el.attr("required").is_some();
    let min = el.attr("min").and_then(|v| v.parse::<f64>().ok());
    let max = el.attr("max").and_then(|v| v.parse::<f64>().ok());
    let minlength = el
        .attr("minlength")
        .or_else(|| el.attr("minLength"))
        .and_then(|v| v.parse::<usize>().ok());
    let maxlength = el
        .attr("maxlength")
        .or_else(|| el.attr("maxLength"))
        .and_then(|v| v.parse::<usize>().ok());
    let pattern = el.attr("pattern").map(|v| v.to_string());
    let step = el.attr("step").and_then(|v| v.parse::<f64>().ok());

    FormField {
        name,
        input_type,
        required,
        min,
        max,
        minlength,
        maxlength,
        pattern,
        step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_form() {
        let html = r#"
        <html><body>
        <form name="login">
            <input type="email" name="email" required />
            <input type="password" name="password" required minlength="8" />
            <input type="submit" value="Login" />
        </form>
        </body></html>
        "#;

        let forms = parse_forms(html);
        assert_eq!(forms.len(), 1);
        assert_eq!(forms[0].identifier, "login");
        assert_eq!(forms[0].fields.len(), 2);

        assert_eq!(forms[0].fields[0].name, "email");
        assert_eq!(forms[0].fields[0].input_type, InputType::Email);
        assert!(forms[0].fields[0].required);

        assert_eq!(forms[0].fields[1].name, "password");
        assert_eq!(forms[0].fields[1].input_type, InputType::Password);
        assert!(forms[0].fields[1].required);
        assert_eq!(forms[0].fields[1].minlength, Some(8));
    }

    #[test]
    fn test_parse_form_with_number_constraints() {
        let html = r#"
        <form id="settings">
            <input type="number" name="age" min="0" max="150" step="1" required />
            <input type="text" name="username" maxlength="50" pattern="[a-zA-Z0-9]+" />
        </form>
        "#;

        let forms = parse_forms(html);
        assert_eq!(forms.len(), 1);
        assert_eq!(forms[0].identifier, "settings");

        let age = &forms[0].fields[0];
        assert_eq!(age.name, "age");
        assert_eq!(age.min, Some(0.0));
        assert_eq!(age.max, Some(150.0));
        assert_eq!(age.step, Some(1.0));
        assert!(age.required);

        let username = &forms[0].fields[1];
        assert_eq!(username.name, "username");
        assert_eq!(username.maxlength, Some(50));
        assert_eq!(username.pattern, Some("[a-zA-Z0-9]+".to_string()));
    }

    #[test]
    fn test_skip_hidden_and_submit_inputs() {
        let html = r#"
        <form name="f">
            <input type="hidden" name="csrf" value="token123" />
            <input type="text" name="visible" />
            <input type="submit" value="Go" />
            <input type="button" value="Cancel" />
        </form>
        "#;

        let forms = parse_forms(html);
        assert_eq!(forms[0].fields.len(), 1);
        assert_eq!(forms[0].fields[0].name, "visible");
    }

    #[test]
    fn test_parse_textarea_and_select() {
        let html = r#"
        <form name="feedback">
            <textarea name="message" required minlength="10" maxlength="500"></textarea>
            <select name="category" required>
                <option value="">Choose...</option>
                <option value="bug">Bug</option>
            </select>
        </form>
        "#;

        let forms = parse_forms(html);
        assert_eq!(forms[0].fields.len(), 2);
        assert_eq!(forms[0].fields[0].input_type, InputType::Textarea);
        assert_eq!(forms[0].fields[0].minlength, Some(10));
        assert_eq!(forms[0].fields[0].maxlength, Some(500));
        assert_eq!(forms[0].fields[1].input_type, InputType::Select);
        assert!(forms[0].fields[1].required);
    }
}
