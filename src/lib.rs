use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent, KeyboardEvent};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

mod errors;
pub use errors::{AppError, update_status};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Select,
    Rectangle,
    Circle,
    Line,
    Text,
    Delete,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
    Text,
}

pub struct Shape {
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: String,
    pub text: String,
    pub font_size: f64,
}

pub struct AppState {
    pub shapes: Vec<Shape>,
    pub current_tool: Tool,
    pub selected_index: Option<usize>,
    pub is_interacting: bool,
    pub start_x: f64,
    pub start_y: f64,
    pub current_x: f64,
    pub current_y: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            current_tool: Tool::Select,
            selected_index: None,
            is_interacting: false,
            start_x: 0.0,
            start_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
            canvas_width: 800.0,
            canvas_height: 600.0,
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            self.shapes.remove(idx);
            self.selected_index = None;
        }
    }
}

// Input validation functions
fn validate_canvas_dimensions(width: f64, height: f64) -> Result<(), AppError> {
    const MIN_DIMENSION: f64 = 100.0;
    const MAX_DIMENSION: f64 = 10000.0;

    if width < MIN_DIMENSION || height < MIN_DIMENSION || width.is_nan() || height.is_nan() {
        return Err(AppError::InvalidDimensions { width, height });
    }

    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(AppError::InvalidDimensions { width, height });
    }

    Ok(())
}

fn validate_font_size(size: f64) -> Result<(), AppError> {
    const MIN_FONT_SIZE: f64 = 6.0;
    const MAX_FONT_SIZE: f64 = 72.0;

    if size < MIN_FONT_SIZE || size > MAX_FONT_SIZE || size.is_nan() {
        return Err(AppError::InvalidFontSize {
            size,
            min: MIN_FONT_SIZE,
            max: MAX_FONT_SIZE
        });
    }

    Ok(())
}

fn validate_mouse_position(x: f64, y: f64, canvas_width: f64, canvas_height: f64) -> Result<(), AppError> {
    if x < 0.0 || x > canvas_width || y < 0.0 || y > canvas_height || x.is_nan() || y.is_nan() {
        return Err(AppError::MouseOutOfBounds { x, y });
    }
    Ok(())
}

fn validate_color(color: &str) -> Result<(), AppError> {
    // Basic hex color validation
    if !color.starts_with('#') || color.len() != 7 {
        return Err(AppError::InvalidColor { color: color.to_string() });
    }

    // Check if hex characters are valid
    for c in color.chars().skip(1) {
        if !c.is_ascii_hexdigit() {
            return Err(AppError::InvalidColor { color: color.to_string() });
        }
    }

    Ok(())
}

fn validate_text(text: &str) -> Result<(), AppError> {
    if text.len() > 1000 {
        return Err(AppError::InvalidText { reason: "Text too long (max 1000 characters)".to_string() });
    }

    // Check for control characters (except common whitespace)
    for c in text.chars() {
        if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
            return Err(AppError::InvalidText { reason: "Text contains invalid characters".to_string() });
        }
    }

    Ok(())
}

// Global thread-safe state
static APP_STATE: Lazy<Arc<Mutex<AppState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(AppState::new()))
});

const TOOLBAR_HEIGHT: f64 = 50.0;
const BUTTON_WIDTH: f64 = 80.0;

#[wasm_bindgen]
pub fn set_selected_color(color: String) -> Result<(), JsValue> {
    validate_color(&color)?;

    if let Ok(mut s) = APP_STATE.lock() {
        if let Some(idx) = s.selected_index {
            s.shapes[idx].color = color;
        }
    }
    Ok(())
}

#[wasm_bindgen]
pub fn set_selected_font_size(size: f64) -> Result<(), JsValue> {
    validate_font_size(size)?;

    if let Ok(mut s) = APP_STATE.lock() {
        if let Some(idx) = s.selected_index {
            s.shapes[idx].font_size = size;
        }
    }
    Ok(())
}

#[wasm_bindgen]
pub fn resize_canvas(width: f64, height: f64) -> Result<(), JsValue> {
    validate_canvas_dimensions(width, height)?;

    if let Ok(mut s) = APP_STATE.lock() {
        s.canvas_width = width;
        s.canvas_height = height;
    }
    Ok(())
}

#[wasm_bindgen]
pub fn set_selected_text(text: String) -> Result<(), JsValue> {
    validate_text(&text)?;

    if let Ok(mut s) = APP_STATE.lock() {
        if let Some(idx) = s.selected_index {
            s.shapes[idx].text = text;
        }
    }
    Ok(())
}

#[wasm_bindgen]
pub fn get_selected_color() -> String {
    if let Ok(s) = APP_STATE.lock() {
        if let Some(idx) = s.selected_index {
            return s.shapes[idx].color.clone();
        }
    }
    "#E0E0E0".to_string()
}

#[wasm_bindgen]
pub fn get_selected_text() -> String {
    if let Ok(s) = APP_STATE.lock() {
        if let Some(idx) = s.selected_index {
            return s.shapes[idx].text.clone();
        }
    }
    "".to_string()
}

#[wasm_bindgen]
pub fn get_selected_font_size() -> f64 {
    if let Ok(s) = APP_STATE.lock() {
        if let Some(idx) = s.selected_index {
            return s.shapes[idx].font_size;
        }
    }
    20.0
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
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
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let state = APP_STATE.clone();

    // --- Helper Function to Render the Canvas ---
    let render = {
        let context = context.clone();
        let canvas = canvas.clone();
        let state = state.clone();
        
        move || {
            let s = state.lock().map_err(|_| AppError::StateLockPoisoned)
                .map_err(|e| JsValue::from(e))
                .unwrap();
            
            // Sync canvas size if changed
            if canvas.width() as f64 != s.canvas_width || canvas.height() as f64 != s.canvas_height {
                canvas.set_width(s.canvas_width as u32);
                canvas.set_height(s.canvas_height as u32);
            }

            // Clear the canvas
            context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

            // Draw Shapes
            for (idx, shape) in s.shapes.iter().enumerate() {
                draw_shape(&context, shape);
                
                // Highlight Selection
                if s.selected_index == Some(idx) {
                    context.set_stroke_style(&JsValue::from_str("#18A0FB"));
                    context.set_line_width(2.0);
                    context.set_line_dash(&JsValue::from(js_sys::Array::of2(&JsValue::from_f64(5.0), &JsValue::from_f64(5.0)))).unwrap();
                    
                    if shape.shape_type == ShapeType::Line {
                        context.stroke_rect(
                            shape.x.min(shape.x2) - 5.0,
                            shape.y.min(shape.y2) - 5.0,
                            (shape.x - shape.x2).abs() + 10.0,
                            (shape.y - shape.y2).abs() + 10.0
                        );
                    } else {
                        context.stroke_rect(shape.x - 5.0, shape.y - 5.0, shape.width + 10.0, shape.height + 10.0);
                    }
                    context.set_line_dash(&JsValue::from(js_sys::Array::new())).unwrap();
                }
            }

            // Draw Interaction Preview
            if s.is_interacting && s.current_tool != Tool::Select && s.current_tool != Tool::Delete {
                context.set_stroke_style(&JsValue::from_str("rgba(24, 160, 251, 0.5)"));
                context.set_line_width(1.0);
                let preview_shape = create_shape_from_interaction(&s);
                draw_shape(&context, &preview_shape);
            }

            // Draw Toolbar
            draw_toolbar(&context, &s);
        }
    };

    // --- Animation Loop ---
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let render_clone = render.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        render_clone();
        web_sys::window().unwrap().request_animation_frame(
            f.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap();
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;

    fn draw_shape(context: &CanvasRenderingContext2d, shape: &Shape) {
        context.set_fill_style(&JsValue::from_str(&shape.color));
        context.set_stroke_style(&JsValue::from_str(&shape.color));
        context.set_line_width(2.0);

        match shape.shape_type {
            ShapeType::Rectangle => {
                context.fill_rect(shape.x, shape.y, shape.width, shape.height);
                context.stroke_rect(shape.x, shape.y, shape.width, shape.height);
            }
            ShapeType::Circle => {
                context.begin_path();
                let center_x = shape.x + shape.width / 2.0;
                let center_y = shape.y + shape.height / 2.0;
                let radius = (shape.width / 2.0).max(shape.height / 2.0);
                context.arc(center_x, center_y, radius, 0.0, std::f64::consts::PI * 2.0).unwrap();
                context.fill();
                context.stroke();
            }
            ShapeType::Line => {
                context.begin_path();
                context.move_to(shape.x, shape.y);
                context.line_to(shape.x2, shape.y2);
                context.stroke();
            }
            ShapeType::Text => {
                context.set_font(&format!("{}px sans-serif", shape.font_size));
                context.fill_text(&shape.text, shape.x, shape.y + shape.font_size).unwrap();
            }
        }
    }

    fn draw_toolbar(context: &CanvasRenderingContext2d, state: &AppState) {
        context.set_fill_style(&JsValue::from_str("#333333"));
        context.fill_rect(0.0, 0.0, state.canvas_width, TOOLBAR_HEIGHT);

        let tools = [Tool::Select, Tool::Rectangle, Tool::Circle, Tool::Line, Tool::Text, Tool::Delete];
        let labels = ["Select", "Rect", "Circle", "Line", "Text", "DELETE"];

        for (i, tool) in tools.iter().enumerate() {
            let x = i as f64 * BUTTON_WIDTH;
            if state.current_tool == *tool {
                context.set_fill_style(&JsValue::from_str("#18A0FB"));
            } else if *tool == Tool::Delete {
                context.set_fill_style(&JsValue::from_str("#F44336"));
            } else {
                context.set_fill_style(&JsValue::from_str("#444444"));
            }
            context.fill_rect(x + 5.0, 5.0, BUTTON_WIDTH - 10.0, TOOLBAR_HEIGHT - 10.0);
            
            context.set_fill_style(&JsValue::from_str("#FFFFFF"));
            context.set_font("14px sans-serif");
            context.fill_text(labels[i], x + 10.0, 30.0).unwrap();
        }
    }

    fn create_shape_from_interaction(state: &AppState) -> Shape {
        let x = state.start_x.min(state.current_x);
        let y = state.start_y.min(state.current_y);
        let width = (state.start_x - state.current_x).abs();
        let height = (state.start_y - state.current_y).abs();

        match state.current_tool {
            Tool::Rectangle => Shape {
                shape_type: ShapeType::Rectangle,
                x, y, width, height,
                x2: 0.0, y2: 0.0,
                color: "#E0E0E0".to_string(),
                text: "".to_string(),
                font_size: 20.0,
            },
            Tool::Circle => Shape {
                shape_type: ShapeType::Circle,
                x, y, width, height,
                x2: 0.0, y2: 0.0,
                color: "#E0E0E0".to_string(),
                text: "".to_string(),
                font_size: 20.0,
            },
            Tool::Line => Shape {
                shape_type: ShapeType::Line,
                x: state.start_x, y: state.start_y,
                width: 0.0, height: 0.0,
                x2: state.current_x, y2: state.current_y,
                color: "#E0E0E0".to_string(),
                text: "".to_string(),
                font_size: 20.0,
            },
            Tool::Text => Shape {
                shape_type: ShapeType::Text,
                x: state.start_x, y: state.start_y,
                width: 100.0, height: 30.0,
                x2: 0.0, y2: 0.0,
                color: "#FFFFFF".to_string(),
                text: "Text".to_string(),
                font_size: 20.0,
            },
            _ => unreachable!(),
        }
    }

    // Initial draw
    render();

    // --- Event Listeners ---
    
    // 1. Mouse Down
    {
        let state = state.clone();
        let render = render.clone();
        
        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mouse_x = event.offset_x() as f64;
            let mouse_y = event.offset_y() as f64;

            {
                let mut s = state.lock().map_err(|_| AppError::StateLockPoisoned)
                    .map_err(|e| JsValue::from(e))
                    .unwrap();

                // Validate mouse position
                if let Err(e) = validate_mouse_position(mouse_x, mouse_y, s.canvas_width, s.canvas_height) {
                    web_sys::console::log_1(&format!("Warning: {:?}", e).into());
                    update_status(&e.to_string());
                    return;
                }

                // Toolbar Hit Test
                if mouse_y < TOOLBAR_HEIGHT {
                    let tool_idx = (mouse_x / BUTTON_WIDTH) as usize;
                    match tool_idx {
                        0 => s.current_tool = Tool::Select,
                        1 => s.current_tool = Tool::Rectangle,
                        2 => s.current_tool = Tool::Circle,
                        3 => s.current_tool = Tool::Line,
                        4 => s.current_tool = Tool::Text,
                        5 => {
                            s.delete_selected();
                        },
                        _ => {}
                    }
                    web_sys::console::log_1(&JsValue::from_str(&format!("Tool selected: {:?}", s.current_tool)));
                    if tool_idx != 5 {
                        s.selected_index = None;
                    }
                } else {
                    s.start_x = mouse_x;
                    s.start_y = mouse_y;
                    s.current_x = mouse_x;
                    s.current_y = mouse_y;
                    s.is_interacting = true;

                    if s.current_tool == Tool::Select {
                        s.selected_index = None;
                        for (idx, shape) in s.shapes.iter().enumerate().rev() {
                            if is_point_in_shape(mouse_x, mouse_y, shape) {
                                s.selected_index = Some(idx);
                                break;
                            }
                        }
                    }
                }
            }
            render();
        });
        
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 2. Mouse Move
    {
        let state = state.clone();
        let render = render.clone();

        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mouse_x = event.offset_x() as f64;
            let mouse_y = event.offset_y() as f64;
            let mut should_render = false;

            {
                let mut s = state.lock().map_err(|_| AppError::StateLockPoisoned)
                    .map_err(|e| JsValue::from(e))
                    .unwrap();
                if s.is_interacting {
                    let dx = mouse_x - s.current_x;
                    let dy = mouse_y - s.current_y;
                    
                    s.current_x = mouse_x;
                    s.current_y = mouse_y;

                    if s.current_tool == Tool::Select {
                        if let Some(idx) = s.selected_index {
                            let shape = &mut s.shapes[idx];
                            shape.x += dx;
                            shape.y += dy;
                            shape.x2 += dx;
                            shape.y2 += dy;
                        }
                    }
                    should_render = true;
                }
            }
            if should_render {
                render();
            }
        });

        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 3. Mouse Up
    {
        let state = state.clone();
        let render = render.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |_event: MouseEvent| {
            let mut should_render = false;
            {
                let mut s = state.lock().map_err(|_| AppError::StateLockPoisoned)
                    .map_err(|e| JsValue::from(e))
                    .unwrap();
                if s.is_interacting {
                    if s.current_tool != Tool::Select && s.current_tool != Tool::Delete {
                        let new_shape = create_shape_from_interaction(&s);
                        s.shapes.push(new_shape);
                        s.selected_index = Some(s.shapes.len() - 1);
                    }
                    s.is_interacting = false;
                    should_render = true;
                }
            }
            if should_render {
                render();
            }
        });
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 4. Keyboard Listener (Delete/Backspace)
    {
        let state = state.clone();
        let render = render.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let key = event.key();
            if key == "Delete" || key == "Backspace" {
                if let Ok(mut s) = state.lock() {
                    s.delete_selected();
                    render();
                }
            }
        });
        window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    fn is_point_in_shape(x: f64, y: f64, shape: &Shape) -> bool {
        match shape.shape_type {
            ShapeType::Line => {
                let dist = dist_to_segment(x, y, shape.x, shape.y, shape.x2, shape.y2);
                dist < 5.0
            }
            _ => {
                x >= shape.x && x <= shape.x + shape.width &&
                y >= shape.y && y <= shape.y + shape.height
            }
        }
    }

    fn dist_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let l2 = (x1 - x2).powi(2) + (y1 - y2).powi(2);
        if l2 == 0.0 { return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt(); }
        let t = ((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / l2;
        let t = t.max(0.0).min(1.0);
        ((px - (x1 + t * (x2 - x1))).powi(2) + (py - (y1 + t * (y2 - y1))).powi(2)).sqrt()
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(validate_font_size(6.0).is_ok());
        assert!(validate_font_size(20.0).is_ok());
        assert!(validate_font_size(72.0).is_ok());

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

        // Invalid formats
        assert!(validate_color("ff0000").is_err()); // Missing #
        assert!(validate_color("#ff00").is_err()); // Too short
        assert!(validate_color("#ff00000").is_err()); // Too long
        assert!(validate_color("#gg0000").is_err()); // Invalid hex character
        assert!(validate_color("red").is_err()); // Named color not supported
    }

    #[test]
    fn test_validate_text() {
        // Valid text
        assert!(validate_text("Hello World").is_ok());
        assert!(validate_text("Text with tabs\t").is_ok());
        assert!(validate_text("Text\nwith\nlines").is_ok());

        // Text too long
        let long_text = "a".repeat(1001);
        assert!(validate_text(&long_text).is_err());

        // Control characters (except common whitespace)
        assert!(validate_text("Text\x00with\x01control").is_err());
    }

    #[test]
    fn test_validate_mouse_position() {
        // Valid positions
        assert!(validate_mouse_position(400.0, 300.0, 800.0, 600.0).is_ok());
        assert!(validate_mouse_position(0.0, 0.0, 800.0, 600.0).is_ok());
        assert!(validate_mouse_position(800.0, 600.0, 800.0, 600.0).is_ok());

        // Out of bounds
        assert!(validate_mouse_position(-10.0, 300.0, 800.0, 600.0).is_err());
        assert!(validate_mouse_position(400.0, -10.0, 800.0, 600.0).is_err());
        assert!(validate_mouse_position(400.0, 650.0, 800.0, 600.0).is_err());

        // NaN values
        assert!(validate_mouse_position(f64::NAN, 300.0, 800.0, 600.0).is_err());
        assert!(validate_mouse_position(400.0, f64::NAN, 800.0, 600.0).is_err());
    }

    #[test]
    fn test_shape_creation() {
        // Test rectangle creation
        let rect = Shape {
            shape_type: ShapeType::Rectangle,
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
            x2: 0.0,
            y2: 0.0,
            color: "#ff0000".to_string(),
            text: "".to_string(),
            font_size: 20.0,
        };
        assert_eq!(rect.shape_type, ShapeType::Rectangle);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 10.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
        assert_eq!(rect.color, "#ff0000");

        // Test circle creation
        let circle = Shape {
            shape_type: ShapeType::Circle,
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 80.0,
            x2: 0.0,
            y2: 0.0,
            color: "#00ff00".to_string(),
            text: "".to_string(),
            font_size: 20.0,
        };
        assert_eq!(circle.shape_type, ShapeType::Circle);

        // Test line creation
        let line = Shape {
            shape_type: ShapeType::Line,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: "#0000ff".to_string(),
            text: "".to_string(),
            font_size: 20.0,
        };
        assert_eq!(line.shape_type, ShapeType::Line);
        assert_eq!(line.x2, 100.0);
        assert_eq!(line.y2, 100.0);
    }

    #[test]
    fn test_app_state() {
        let mut state = AppState::new();

        // Test initial state
        assert_eq!(state.shapes.len(), 0);
        assert_eq!(state.current_tool, Tool::Select);
        assert_eq!(state.selected_index, None);
        assert_eq!(state.is_interacting, false);
        assert_eq!(state.canvas_width, 800.0);
        assert_eq!(state.canvas_height, 600.0);

        // Test delete_selected with no selection
        state.delete_selected();
        assert_eq!(state.selected_index, None);

        // Test delete_selected with selection
        state.shapes.push(Shape {
            shape_type: ShapeType::Rectangle,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            x2: 0.0,
            y2: 0.0,
            color: "#ff0000".to_string(),
            text: "".to_string(),
            font_size: 20.0,
        });
        state.selected_index = Some(0);
        state.delete_selected();
        assert_eq!(state.shapes.len(), 0);
        assert_eq!(state.selected_index, None);
    }
}