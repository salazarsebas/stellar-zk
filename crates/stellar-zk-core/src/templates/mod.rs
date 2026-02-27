//! Template system for stellar-zk project scaffolding.
//!
//! Templates are embedded into the binary at compile-time via [`include_str!`] in the
//! [`embedded`] module, then rendered at runtime with [Handlebars](https://handlebarsjs.com/)
//! via the [`renderer::TemplateRenderer`].
//!
//! ## Template variables
//!
//! Templates use Handlebars syntax. Common variables:
//! - `{{contract_name}}` — Soroban contract name (e.g., `groth16_verifier`)
//! - `{{project_name}}` — project directory name
//! - `{{backend}}` — backend identifier (`groth16`, `ultrahonk`, `risc0`)
//!
//! ## Adding a new template
//!
//! 1. Create the `.tmpl` file under `templates/`
//! 2. Add a `pub const` with `include_str!` in [`embedded`]
//! 3. Run `cargo build` to verify the path resolves
//!
//! **Warning**: Template files in `templates/` and constants in [`embedded`] must stay in sync.
//! The `include_str!` paths are relative to this file and checked at compile-time.

pub mod embedded;
pub mod renderer;
