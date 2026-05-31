use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Canvas element not found")]
    CanvasNotFound,

    #[error("Invalid canvas dimensions: {width}x{height}")]
    InvalidDimensions { width: f64, height: f64 },

    #[error("Invalid color format: {color}")]
    InvalidColor { color: String },

    #[error("Invalid font size: {size}. Must be between {min} and {max}")]
    InvalidFontSize { size: f64, min: f64, max: f64 },

    #[error("State lock poisoned")]
    StateLockPoisoned,

    #[error("DOM operation failed: {operation}")]
    DomOperationFailed { operation: String },

    #[error("Mouse coordinates out of bounds: ({x}, {y})")]
    MouseOutOfBounds { x: f64, y: f64 },

    #[error("Invalid text input: {reason}")]
    InvalidText { reason: String },
}

impl From<AppError> for JsValue {
    fn from(err: AppError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

pub fn update_status(message: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(status_bar) = document.get_element_by_id("status-bar") {
                status_bar.set_text_content(Some(message));
            } else {
                // Fallback to console if status bar doesn't exist yet
                web_sys::console::log_1(&JsValue::from_str(message));
            }
        }
    }
}