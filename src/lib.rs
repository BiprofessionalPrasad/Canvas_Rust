use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use std::rc::Rc;
use std::cell::RefCell;

mod constants;
mod errors;
mod state;
mod shapes;
mod tools;
mod validation;
mod interaction;
mod rendering;
mod events;

pub use errors::{AppError, update_status};
pub use shapes::{Shape, ShapeType};
pub use state::{AppState, App};
pub use tools::Tool;
pub use validation::*;

thread_local! {
    static APP_INSTANCE: RefCell<Option<Rc<App>>> = RefCell::new(None);
}

fn get_app() -> Rc<App> {
    APP_INSTANCE.with(|app_cell| {
        if app_cell.borrow().is_none() {
            let app = Rc::new(App::new());
            *app_cell.borrow_mut() = Some(app.clone());
        }
        app_cell.borrow().as_ref().unwrap().clone()
    })
}

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

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
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

    let app = get_app();

    // Create the renderer
    let renderer = Rc::new(RefCell::new(rendering::Renderer::new(canvas.clone(), context.clone())));

    // Create render function that combines dirty check + render + clear in one with_state call
    let app_render = app.clone();
    let renderer_render = renderer.clone();
    let render_fn = Box::new(move || -> Result<bool, AppError> {
        app_render.with_state(|state| {
            if !state.is_dirty() {
                return Ok(false);
            }
            renderer_render.borrow().render(state)?;
            state.clear_dirty();
            Ok(true)
        })
    });

    // Create the animation loop
    let animation_loop = Rc::new(rendering::AnimationLoop::new(render_fn));

    // Initial render
    let _ = app.with_state(|state| {
        renderer.borrow().render(state)?;
        state.clear_dirty();
        Ok::<(), AppError>(())
    });

    // Set up event handlers with animation loop
    let animation_loop_events = animation_loop.clone();
    let app_events = app.clone();

    // Wrap animation loop start in a callback for event handlers
    let request_render = Rc::new(RefCell::new(Box::new(move || {
        animation_loop_events.start();
    }) as Box<dyn FnMut()>));

    let app_setup = app_events.clone();
    let request_render_setup = request_render.clone();
    let render_callback = Rc::new(RefCell::new(Box::new(move || {
        request_render_setup.borrow_mut()();
    }) as Box<dyn FnMut()>));

    let event_handlers = events::EventHandlers::new();
    event_handlers.setup_all(app_setup, &canvas, render_callback.clone())?;

    // Initial render triggers animation loop
    request_render.borrow_mut()();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_validate_canvas_dimensions() {
        assert!(validate_canvas_dimensions(800.0, 600.0).is_ok());
        assert!(validate_canvas_dimensions(50.0, 600.0).is_err());
        assert!(validate_canvas_dimensions(800.0, 50.0).is_err());
        assert!(validate_canvas_dimensions(800.0, 20000.0).is_err());
        assert!(validate_canvas_dimensions(20000.0, 600.0).is_err());
        assert!(validate_canvas_dimensions(f64::NAN, 600.0).is_err());
        assert!(validate_canvas_dimensions(800.0, f64::NAN).is_err());
    }

    #[test]
    fn test_validate_font_size() {
        assert!(validate_font_size(MIN_FONT_SIZE).is_ok());
        assert!(validate_font_size(20.0).is_ok());
        assert!(validate_font_size(MAX_FONT_SIZE).is_ok());
        assert!(validate_font_size(4.0).is_err());
        assert!(validate_font_size(100.0).is_err());
        assert!(validate_font_size(f64::NAN).is_err());
    }

    #[test]
    fn test_validate_color() {
        assert!(validate_color("#ff0000").is_ok());
        assert!(validate_color("#00FF00").is_ok());
        assert!(validate_color("#0000FF").is_ok());
        assert!(validate_color("#123ABC").is_ok());
        assert!(validate_color("rgb(255, 0, 0)").is_ok());
        assert!(validate_color("rgb(128, 128, 128)").is_ok());
        assert!(validate_color("red").is_ok());
        assert!(validate_color("blue").is_ok());
        assert!(validate_color("green").is_ok());
        assert!(validate_color("ff0000").is_err());
        assert!(validate_color("#ff00").is_err());
        assert!(validate_color("#ff00000").is_err());
        assert!(validate_color("#gg0000").is_err());
        assert!(validate_color("rgb(300, 0, 0)").is_err());
    }

    #[test]
    fn test_validate_text() {
        assert!(validate_text("Hello World").is_ok());
        assert!(validate_text("Text with tabs\t").is_ok());
        assert!(validate_text("Text\nwith\nlines").is_ok());
        let long_text = "a".repeat(MAX_TEXT_LENGTH + 1);
        assert!(validate_text(&long_text).is_err());
        assert!(validate_text("Text\x00with\x01control").is_err());
    }

    #[test]
    fn test_app_state() {
        let mut state = AppState::new();
        assert_eq!(state.shapes.len(), 0);
        assert_eq!(state.current_tool, Tool::Select);
        assert_eq!(state.selected_index, None);
        assert_eq!(state.is_interacting, false);
        assert_eq!(state.canvas_width, DEFAULT_CANVAS_WIDTH);
        assert_eq!(state.canvas_height, DEFAULT_CANVAS_HEIGHT);
        assert!(state.is_dirty());

        state.clear_dirty();
        assert!(!state.is_dirty());
        state.mark_dirty();
        assert!(state.is_dirty());

        assert!(state.delete_selected().is_err());

        let shape = Shape::new_rectangle(0.0, 0.0, 50.0, 50.0, "#ff0000".to_string(), 0);
        assert!(state.add_shape(shape).is_ok());
        state.selected_index = Some(0);
        assert!(state.delete_selected().is_ok());
        assert_eq!(state.shapes.len(), 0);
        assert_eq!(state.selected_index, None);
    }

    #[test]
    fn test_shape_creation() {
        let rect = Shape::new_rectangle(10.0, 10.0, 100.0, 50.0, "#ff0000".to_string(), 0);
        assert_eq!(rect.shape_type, ShapeType::Rectangle);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 10.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
        assert_eq!(rect.color, "#ff0000");

        let circle = Shape::new_circle(20.0, 20.0, 80.0, 80.0, "#00ff00".to_string(), 1);
        assert_eq!(circle.shape_type, ShapeType::Circle);

        let line = Shape::new_line(0.0, 0.0, 100.0, 100.0, "#0000ff".to_string(), 2);
        assert_eq!(line.shape_type, ShapeType::Line);
        assert_eq!(line.x2, 100.0);
        assert_eq!(line.y2, 100.0);
    }

    #[test]
    fn test_shape_contains_point() {
        let rect = Shape::new_rectangle(10.0, 10.0, 100.0, 50.0, "#ff0000".to_string(), 0);
        assert!(rect.contains_point(50.0, 30.0));
        assert!(!rect.contains_point(5.0, 30.0));
        assert!(!rect.contains_point(150.0, 30.0));
        assert!(!rect.contains_point(50.0, 5.0));
        assert!(!rect.contains_point(50.0, 80.0));
    }

    #[test]
    fn test_app_wrapper() {
        let app = App::new();
        let result = app.with_state(|state| {
            assert_eq!(state.shapes.len(), 0);
            Ok(42)
        });
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_max_shapes_limit() {
        let mut state = AppState::new();
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
        let extra_shape = Shape::new_rectangle(0.0, 0.0, 10.0, 10.0, "#ff0000".to_string(), MAX_SHAPES as u32);
        assert!(state.add_shape(extra_shape).is_err());
    }
}
