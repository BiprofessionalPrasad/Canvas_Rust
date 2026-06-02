use crate::constants::*;
use crate::errors::AppError;

pub fn validate_canvas_dimensions(width: f64, height: f64) -> Result<(), AppError> {
    if width < MIN_DIMENSION || height < MIN_DIMENSION || width.is_nan() || height.is_nan() {
        return Err(AppError::InvalidDimensions { width, height });
    }

    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(AppError::InvalidDimensions { width, height });
    }

    Ok(())
}

pub fn validate_font_size(size: f64) -> Result<(), AppError> {
    if size < MIN_FONT_SIZE || size > MAX_FONT_SIZE || size.is_nan() {
        return Err(AppError::InvalidFontSize {
            size,
            min: MIN_FONT_SIZE,
            max: MAX_FONT_SIZE,
        });
    }

    Ok(())
}

pub fn validate_mouse_position(x: f64, y: f64, canvas_width: f64, canvas_height: f64) -> Result<(), AppError> {
    if x < 0.0 || x > canvas_width || y < 0.0 || y > canvas_height || x.is_nan() || y.is_nan() {
        return Err(AppError::MouseOutOfBounds { x, y });
    }
    Ok(())
}

pub fn validate_color(color: &str) -> Result<(), AppError> {
    // Accept hex colors
    if color.starts_with('#') {
        if color.len() != 7 {
            return Err(AppError::InvalidColor {
                color: color.to_string(),
            });
        }

        // Check if hex characters are valid
        for c in color.chars().skip(1) {
            if !c.is_ascii_hexdigit() {
                return Err(AppError::InvalidColor {
                    color: color.to_string(),
                });
            }
        }
        return Ok(());
    }

    // Accept RGB format
    if color.starts_with("rgb(") && color.ends_with(')') {
        let parts = color[4..color.len() - 1].split(',');
        if parts.clone().count() == 3 {
            for part in parts {
                if let Ok(num) = part.trim().parse::<f64>() {
                    if num < 0.0 || num > 255.0 {
                        return Err(AppError::InvalidColor {
                            color: color.to_string(),
                        });
                    }
                } else {
                    return Err(AppError::InvalidColor {
                        color: color.to_string(),
                    });
                }
            }
            return Ok(());
        }
    }

    // Accept some basic CSS color names
    match color.to_lowercase().as_str() {
        "red" | "green" | "blue" | "yellow" | "purple" | "orange" | "pink" | "brown"
        | "black" | "white" | "gray" | "grey" | "cyan" | "magenta" | "lime" | "navy"
        | "teal" | "olive" | "maroon" | "aqua" | "silver" | "fuchsia" => return Ok(()),
        _ => {}
    }

    Err(AppError::InvalidColor {
        color: color.to_string(),
    })
}

pub fn validate_text(text: &str) -> Result<(), AppError> {
    if text.len() > MAX_TEXT_LENGTH {
        return Err(AppError::InvalidText {
            reason: format!("Text too long (max {} characters)", MAX_TEXT_LENGTH),
        });
    }

    // Check for control characters (except common whitespace)
    for c in text.chars() {
        if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
            return Err(AppError::InvalidText {
                reason: "Text contains invalid characters".to_string(),
            });
        }
    }

    // Sanitize text for potential XSS if ever rendered as HTML
    if text.contains('<') || text.contains('>') || text.contains('&') {
        // Basic HTML escaping warning
        web_sys::console::warn_1(&"Text contains HTML special characters".into());
    }

    Ok(())
}