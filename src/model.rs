//! Data structures for form fields and constraints.

use serde::Serialize;
use std::fmt;

/// Supported HTML input types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Text,
    Number,
    Email,
    Url,
    Password,
    Tel,
    Search,
    Date,
    Hidden,
    Checkbox,
    Radio,
    Select,
    Textarea,
    Other(String),
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputType::Other(s) => write!(f, "{s}"),
            _ => {
                let s = serde_json::to_string(self).unwrap_or_default();
                // Remove quotes from JSON string
                write!(f, "{}", s.trim_matches('"'))
            }
        }
    }
}

impl InputType {
    /// Parse an HTML `type` attribute value into an `InputType`.
    pub fn from_attr(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "text" => InputType::Text,
            "number" => InputType::Number,
            "email" => InputType::Email,
            "url" => InputType::Url,
            "password" => InputType::Password,
            "tel" => InputType::Tel,
            "search" => InputType::Search,
            "date" => InputType::Date,
            "hidden" => InputType::Hidden,
            "checkbox" => InputType::Checkbox,
            "radio" => InputType::Radio,
            "select" => InputType::Select,
            "textarea" => InputType::Textarea,
            other => InputType::Other(other.to_string()),
        }
    }
}

/// Represents a single form field with its HTML5 constraint attributes.
#[derive(Debug, Clone, Serialize)]
pub struct FormField {
    /// The `name` (or `id` fallback) of the field.
    pub name: String,
    /// The input type (text, email, number, etc.).
    pub input_type: InputType,
    /// Whether the `required` attribute is present.
    pub required: bool,
    /// The `min` attribute value (for number/date inputs).
    pub min: Option<f64>,
    /// The `max` attribute value (for number/date inputs).
    pub max: Option<f64>,
    /// The `minlength` attribute value.
    pub minlength: Option<usize>,
    /// The `maxlength` attribute value.
    pub maxlength: Option<usize>,
    /// The `pattern` attribute value (regex).
    pub pattern: Option<String>,
    /// The `step` attribute value (for number inputs).
    pub step: Option<f64>,
}

/// Represents a parsed form with its fields.
#[derive(Debug, Clone, Serialize)]
pub struct FormInfo {
    /// The form's `name`, `id`, or `action` attribute (for identification).
    pub identifier: String,
    /// 0-based index of this form among all `<form>` elements on the page.
    pub index: usize,
    /// The fields within this form.
    pub fields: Vec<FormField>,
}
