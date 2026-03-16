//! YAML configuration for expected form constraints.

use crate::error::{BoundformError, Result};
use indexmap::IndexMap;
use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Pages to validate.
    pub pages: Vec<PageConfig>,
}

/// A page (URL or file path) with its expected forms.
#[derive(Debug, Deserialize)]
pub struct PageConfig {
    /// URL or local file path to fetch.
    pub url: String,
    /// Expected forms on this page.
    pub forms: Vec<FormConfig>,
}

/// Expected form definition.
///
/// Forms can be identified by one of three methods (checked in this order):
/// 1. `selector` — CSS selector matching a `<form>` element
/// 2. `index` — 0-based position among all `<form>` elements on the page
/// 3. `id` — matches the resolved identifier (name → id → action)
///
/// At least one must be specified.
#[derive(Debug, Deserialize)]
pub struct FormConfig {
    /// Match by resolved identifier (name/id/action). Backward compatible.
    pub id: Option<String>,
    /// Match by CSS selector (first matching `<form>`).
    pub selector: Option<String>,
    /// Match by 0-based index among all `<form>` elements on the page.
    pub index: Option<usize>,
    /// Expected fields keyed by field name. Insertion order is preserved.
    pub fields: IndexMap<String, FieldExpectation>,
}

impl FormConfig {
    /// Returns a display label for this form config (for reporting).
    pub fn display_label(&self) -> String {
        if let Some(ref selector) = self.selector {
            format!("selector={selector}")
        } else if let Some(index) = self.index {
            format!("index={index}")
        } else if let Some(ref id) = self.id {
            id.clone()
        } else {
            "unspecified".to_string()
        }
    }
}

/// Expected constraints for a single field.
/// Only specified (non-None) constraints are checked.
#[derive(Debug, Deserialize, Default)]
pub struct FieldExpectation {
    /// Expected input type (e.g., "text", "email", "number").
    #[serde(rename = "type")]
    pub input_type: Option<String>,
    /// Whether the field is expected to be required.
    pub required: Option<bool>,
    /// Expected `min` attribute value.
    pub min: Option<f64>,
    /// Expected `max` attribute value.
    pub max: Option<f64>,
    /// Expected `minlength` attribute value.
    pub minlength: Option<usize>,
    /// Expected `maxlength` attribute value.
    pub maxlength: Option<usize>,
    /// Expected `pattern` attribute value.
    pub pattern: Option<String>,
    /// Expected `step` attribute value.
    pub step: Option<f64>,
}

/// Maximum config file size (1 MB).
const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

/// Load and parse a YAML config file.
pub fn load_config(path: &str) -> Result<Config> {
    // Check file size before reading to prevent YAML bombs
    let metadata = std::fs::metadata(path).map_err(BoundformError::IoError)?;
    if metadata.len() > MAX_CONFIG_SIZE {
        return Err(BoundformError::ConfigError(format!(
            "config file too large: {} bytes (max: {} bytes)",
            metadata.len(),
            MAX_CONFIG_SIZE
        )));
    }

    let content = std::fs::read_to_string(path).map_err(BoundformError::IoError)?;
    let config: Config =
        serde_yml::from_str(&content).map_err(|e| BoundformError::ConfigError(e.to_string()))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let yaml = r#"
pages:
  - url: "http://localhost:3000/register"
    forms:
      - id: "register"
        fields:
          email:
            type: email
            required: true
          username:
            type: text
            required: true
            maxlength: 50
          password:
            type: password
            required: true
            minlength: 10
"#;

        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.pages.len(), 1);
        assert_eq!(config.pages[0].url, "http://localhost:3000/register");
        assert_eq!(config.pages[0].forms.len(), 1);

        let form = &config.pages[0].forms[0];
        assert_eq!(form.id.as_deref(), Some("register"));
        assert_eq!(form.fields.len(), 3);

        let email = &form.fields["email"];
        assert_eq!(email.input_type.as_deref(), Some("email"));
        assert_eq!(email.required, Some(true));
        assert_eq!(email.maxlength, None);

        let username = &form.fields["username"];
        assert_eq!(username.maxlength, Some(50));

        let password = &form.fields["password"];
        assert_eq!(password.minlength, Some(10));
    }

    #[test]
    fn test_parse_config_with_number_constraints() {
        let yaml = r#"
pages:
  - url: "./form.html"
    forms:
      - id: "settings"
        fields:
          age:
            type: number
            min: 0
            max: 150
            step: 1
            required: true
"#;

        let config: Config = serde_yml::from_str(yaml).unwrap();
        let age = &config.pages[0].forms[0].fields["age"];
        assert_eq!(age.min, Some(0.0));
        assert_eq!(age.max, Some(150.0));
        assert_eq!(age.step, Some(1.0));
        assert_eq!(age.required, Some(true));
    }

    #[test]
    fn test_parse_config_with_index() {
        let yaml = r#"
pages:
  - url: "http://localhost:3000/login"
    forms:
      - index: 0
        fields:
          email:
            type: email
            required: true
"#;

        let config: Config = serde_yml::from_str(yaml).unwrap();
        let form = &config.pages[0].forms[0];
        assert_eq!(form.index, Some(0));
        assert!(form.id.is_none());
        assert!(form.selector.is_none());
    }

    #[test]
    fn test_parse_config_with_selector() {
        let yaml = r#"
pages:
  - url: "http://localhost:3000/register"
    forms:
      - selector: "form.signup"
        fields:
          username:
            type: text
"#;

        let config: Config = serde_yml::from_str(yaml).unwrap();
        let form = &config.pages[0].forms[0];
        assert_eq!(form.selector.as_deref(), Some("form.signup"));
        assert!(form.id.is_none());
        assert!(form.index.is_none());
    }

    #[test]
    fn test_required_false_is_meaningful() {
        let yaml = r#"
pages:
  - url: "./form.html"
    forms:
      - id: "f"
        fields:
          website:
            type: url
            required: false
"#;

        let config: Config = serde_yml::from_str(yaml).unwrap();
        let website = &config.pages[0].forms[0].fields["website"];
        assert_eq!(website.required, Some(false));
    }
}
