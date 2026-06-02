use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use std::rc::Rc;
use std::cell::RefCell;

// Module declarations
mod constants;
mod errors;
mod state;
mod shapes;
mod tools;
mod validation;
mod interaction;
mod rendering;
mod events;

// Re-exports for wasm-bindgen
pub use errors::{AppError, update_status};
pub use shapes::{Shape, ShapeType};
pub use state::{AppState, App};
pub use tools::Tool;
pub use validation::*;

// Global app instance (avoiding the previous global state anti-pattern)
thread_local! {
    static APP_INSTANCE: RefCell<Option<Rc<App>>> = RefCell::new(None);
}

// Get or create the app instance
fn get_app() -> Rc<App> {
    APP_INSTANCE.with(|app_cell| {
        if app_cell.borrow().is_none() {
            let app = Rc::new(App::new());
            *app_cell.borrow_mut() = Some(app.clone());
        }
        app_cell.borrow().as_ref().unwrap().clone()
    })
}

// WASM exported functions
#[wasm_bindgen]
pub fn set_selected_color(color: String) -> Result<(), JsValue> {
    validate_color(&color)?;

    get_app().with_state(|state| {
        if let Some(idx) = state.selected_index {
            if let Some(shape) = state.shapes.get_mut(idx) {
                shape.color = color;
                state.mark_dirty();
            }
        }
        Ok(())
    })?;

    Ok(())
}

#[wasm_bindgen]
pub fn set_selected_font_size(size: f64) -> Result<(), JsValue> {
    validate_font_size(size)?;

    get_app().with_state(|state| {
        if let Some(idx) = state.selected_index {
            if let Some(shape) = state.shapes.get_mut(idx) {
                shape.font_size = size;
                state.mark_dirty();
            }
        }
        Ok(())
    })?;

    Ok(())
}

#[wasm_bindgen]
pub fn resize_canvas(width: f64, height: f64) -> Result<(), JsValue> {
    validate_canvas_dimensions(width, height)?;

    get_app().with_state(|state| {
        state.canvas_width = width;
        state.canvas_height = height;
        state.mark_dirty();
        Ok(())
    })?;

    Ok(())
}

#[wasm_bindgen]
pub fn set_selected_text(text: String) -> Result<(), JsValue> {
    validate_text(&text)?;

    get_app().with_state(|state| {
        if let Some(idx) = state.selected_index {
            if let Some(shape) = state.shapes.get_mut(idx) {
                shape.text = text;
                state.mark_dirty();
            }
        }
        Ok(())
    })?;

    Ok(())
}

#[wasm_bindgen]
pub fn get_selected_color() -> String {
    get_app().with_state(|state| {
        Ok(state.get_selected_shape()
            .map(|s| s.color.clone())
            .unwrap_or_else(|| "#E0E0E0".to_string()))
    }).unwrap_or_else(|_| "#E0E0E0".to_string())
}

#[wasm_bindgen]
pub fn get_selected_text() -> String {
    get_app().with_state(|state| {
        Ok(state.get_selected_shape()
            .map(|s| s.text.clone())
            .unwrap_or_else(|| "".to_string()))
    }).unwrap_or_else(|_| "".to_string())
}

#[wasm_bindgen]
pub fn get_selected_font_size() -> f64 {
    get_app().with_state(|state| {
        Ok(state.get_selected_shape()
            .map(|s| s.font_size)
            .unwrap_or(constants::DEFAULT_FONT_SIZE))
    }).unwrap_or(constants::DEFAULT_FONT_SIZE)
}

// Main initialization function
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    let window = web_sys::window()
        .ok_or(AppError::DomOperationFailed { operation: "get window".to_string() })?;
    let document = window.document()
        .ok_or(AppError::DomOperationFailed { operation: "get document".to_string() })?;

    let canvas = document.get_element_by_id("canvas")
        .ok_or(AppError::CanvasNotFound)?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|e| AppError::DomOperationFailed { operation: format!("convert to HtmlCanvasElement: {:?}", e) })?;

    let context = canvas
        .get_context("2d")?
        .ok_or(AppError::DomOperationFailed { operation: "get 2d context".to_string() })?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|e| AppError::DomOperationFailed { operation: format!("convert to CanvasRenderingContext2d: {:?}", e) })?;

    // Create the app instance
    let app = get_app();

    // Set up the renderer
    let renderer = rendering::Renderer::new(canvas.clone(), context.clone());
    let renderer_rc = Rc::new(RefCell::new(renderer));

    // Create render callback that checks dirty flag
    let render_callback = {
        let app_clone = app.clone();
        let renderer_clone = renderer_rc.clone();

        Rc::new(RefCell::new(Box::new(move || {
            app_clone.with_state(|state| {
                renderer_clone.borrow_mut().render(state)
            }).unwrap_or_else(|e| {
                web_sys::console::error_1(&format!("Render error: {:?}", e).into());
            });
        }) as Box<dyn FnMut()>))
    };

    // Initial render
    (*render_callback.borrow_mut())();

    // Set up event handlers
    let event_handlers = events::EventHandlers::new();
    event_handlers.setup_all(app.clone(), &canvas, render_callback.clone())?;

    // Start the animation loop (will only render when dirty flag is set)
    rendering::start_animation_loop(renderer_rc, {
        let render_cb = render_callback.clone();
        Box::new(move || {
            // Check if we need to render
            if let Ok(needs_render) = get_app().with_state(|state| {
                Ok(state.is_dirty())
            }) {
                if needs_render {
                    (*render_cb.borrow_mut())();
                    // Clear dirty flag after rendering
                    let _ = get_app().with_state(|state| {
                        state.clear_dirty();
                        Ok(())
                    });
                }
            }
            Ok(())
        })
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_validate_canvas_dimensions() {
        // Valid dimensions
        assert!(validate_canvas_dimensions(800.0, 600.0).is_ok());

        // Too small
        assert!(validate_canvas_dimensions(50.0, 600.0).is_err());
        assert!(validate_canvas_dimensions(800.0, 50.0).is_err());

        // Too large
        assert!(validate_canvas_dimensions(800.0, 20000.0).is_err());
        assert!(validate_canvas_dimensions(20000.0, 600.0).is_err());

        // NaN values
        assert!(validate_canvas_dimensions(f64::NAN, 600.0).is_err());
        assert!(validate_canvas_dimensions(800.0, f64::NAN).is_err());
    }

    #[test]
    fn test_validate_font_size() {
        // Valid sizes
        assert!(validate_font_size(MIN_FONT_SIZE).is_ok());
        assert!(validate_font_size(20.0).is_ok());
        assert!(validate_font_size(MAX_FONT_SIZE).is_ok());

        // Too small
        assert!(validate_font_size(4.0).is_err());

        // Too large
        assert!(validate_font_size(100.0).is_err());

        // NaN value
        assert!(validate_font_size(f64::NAN).is_err());
    }

    #[test]
    fn test_validate_color() {
        // Valid hex colors
        assert!(validate_color("#ff0000").is_ok());
        assert!(validate_color("#00FF00").is_ok());
        assert!(validate_color("#0000FF").is_ok());
        assert!(validate_color("#123ABC").is_ok());

        // Valid RGB colors
        assert!(validate_color("rgb(255, 0, 0)").is_ok());
        assert!(validate_color("rgb(128, 128, 128)").is_ok());

        // Valid named colors
        assert!(validate_color("red").is_ok());
        assert!(validate_color("blue").is_ok());
        assert!(validate_color("green").is_ok());

        // Invalid formats
        assert!(validate_color("ff0000").is_err()); // Missing #
        assert!(validate_color("#ff00").is_err()); // Too short
        assert!(validate_color("#ff00000").is_err()); // Too long
        assert!(validate_color("#gg0000").is_err()); // Invalid hex character
        assert!(validate_color("rgb(300, 0, 0)").is_err()); // Invalid RGB values
    }

    #[test]
    fn test_validate_text() {
        // Valid text
        assert!(validate_text("Hello World").is_ok());
        assert!(validate_text("Text with tabs\t").is_ok());
        assert!(validate_text("Text\nwith\nlines").is_ok());

        // Text too long
        let long_text = "a".repeat(MAX_TEXT_LENGTH + 1);
        assert!(validate_text(&long_text).is_err());

        // Control characters (except common whitespace)
        assert!(validate_text("Text\x00with\x01control").is_err());
    }

    #[test]
    fn test_app_state() {
        let mut state = AppState::new();

        // Test initial state
        assert_eq!(state.shapes.len(), 0);
        assert_eq!(state.current_tool, Tool::Select);
        assert_eq!(state.selected_index, None);
        assert_eq!(state.is_interacting, false);
        assert_eq!(state.canvas_width, DEFAULT_CANVAS_WIDTH);
        assert_eq!(state.canvas_height, DEFAULT_CANVAS_HEIGHT);
        assert!(state.is_dirty());

        // Test dirty flag
        state.clear_dirty();
        assert!(!state.is_dirty());
        state.mark_dirty();
        assert!(state.is_dirty());

        // Test delete_selected with no selection
        assert!(state.delete_selected().is_err());

        // Test delete_selected with selection
        let shape = Shape::new_rectangle(0.0, 0.0, 50.0, 50.0, "#ff0000".to_string(), 0);
        assert!(state.add_shape(shape).is_ok());
        state.selected_index = Some(0);
        assert!(state.delete_selected().is_ok());
        assert_eq!(state.shapes.len(), 0);
        assert_eq!(state.selected_index, None);
    }

    #[test]
    fn test_shape_creation() {
        // Test rectangle creation
        let rect = Shape::new_rectangle(10.0, 10.0, 100.0, 50.0, "#ff0000".to_string(), 0);
        assert_eq!(rect.shape_type, ShapeType::Rectangle);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 10.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
        assert_eq!(rect.color, "#ff0000");

        // Test circle creation
        let circle = Shape::new_circle(20.0, 20.0, 80.0, 80.0, "#00ff00".to_string(), 1);
        assert_eq!(circle.shape_type, ShapeType::Circle);

        // Test line creation
        let line = Shape::new_line(0.0, 0.0, 100.0, 100.0, "#0000ff".to_string(), 2);
        assert_eq!(line.shape_type, ShapeType::Line);
        assert_eq!(line.x2, 100.0);
        assert_eq!(line.y2, 100.0);
    }

    #[test]
    fn test_shape_contains_point() {
        let rect = Shape::new_rectangle(10.0, 10.0, 100.0, 50.0, "#ff0000".to_string(), 0);

        // Point inside
        assert!(rect.contains_point(50.0, 30.0));

        // Point outside
        assert!(!rect.contains_point(5.0, 30.0));
        assert!(!rect.contains_point(150.0, 30.0));
        assert!(!rect.contains_point(50.0, 5.0));
        assert!(!rect.contains_point(50.0, 80.0));
    }

    #[test]
    fn test_app_wrapper() {
        let app = App::new();

        // Test with_state works
        let result = app.with_state(|state| {
            assert_eq!(state.shapes.len(), 0);
            Ok(42)
        });
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_max_shapes_limit() {
        let mut state = AppState::new();

        // Add shapes up to the limit
        for i in 0..MAX_SHAPES {
            let shape = Shape::new_rectangle(
                i as f64 * 10.0,
                0.0,
                10.0,
                10.0,
                "#ff0000".to_string(),
                i as u32,
            );
            assert!(state.add_shape(shape).is_ok());
        }

        // Try to add one more - should fail
        let extra_shape = Shape::new_rectangle(0.0, 0.0, 10.0, 10.0, "#ff0000".to_string(), MAX_SHAPES as u32);
        assert!(state.add_shape(extra_shape).is_err());
    }
}