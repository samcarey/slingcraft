#![warn(clippy::all, rust_2018_idioms)]

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
pub use app::TemplateApp;
