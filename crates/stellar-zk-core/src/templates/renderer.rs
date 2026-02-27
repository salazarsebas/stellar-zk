//! Handlebars-based template renderer for project scaffolding.
//!
//! Wraps the [`handlebars::Handlebars`] engine with **strict mode** enabled by default.
//! Strict mode ensures that any `{{variable}}` referenced in a template must be present
//! in the data context — otherwise rendering returns an error. This is critical because
//! templates produce Rust source code; a silently missing variable would generate code
//! that fails to compile with confusing errors far from the actual cause.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::templates::{embedded, renderer::TemplateRenderer};
//!
//! let renderer = TemplateRenderer::new();
//! let data = serde_json::json!({ "contract_name": "groth16_verifier" });
//! let output = renderer.render(embedded::GROTH16_CONTRACT_LIB, &data)?;
//! ```

use handlebars::Handlebars;
use serde_json::Value;

use crate::error::{Result, StellarZkError};

/// Template renderer using Handlebars for generating project files.
///
/// Uses strict mode so that any template variable not present in the data context
/// causes an error rather than silently rendering as empty. This catches missing
/// variables early (at `init` time) rather than producing broken contract code.
pub struct TemplateRenderer {
    hbs: Handlebars<'static>,
}

impl TemplateRenderer {
    /// Create a new renderer with strict mode enabled.
    ///
    /// Strict mode means `{{missing_var}}` in a template will return an error
    /// instead of an empty string. This is important because templates generate
    /// Rust source code — a silently missing variable would produce code that
    /// fails to compile with confusing errors.
    pub fn new() -> Self {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        Self { hbs }
    }

    /// Render a template string with the given data context.
    pub fn render(&self, template: &str, data: &Value) -> Result<String> {
        self.hbs
            .render_template(template, data)
            .map_err(|e| StellarZkError::TemplateRender(e.to_string()))
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}
