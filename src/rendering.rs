use crate::shapes::Shape;
use crate::state::AppState;
use crate::tools::Tool;
use crate::constants::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub struct Renderer {
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<CanvasRenderingContext2d>,
    last_rendered_state: Option<usize>, // Simple hash to detect changes
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement, context: CanvasRenderingContext2d) -> Self {
        Self {
            canvas: Rc::new(canvas),
            context: Rc::new(context),
            last_rendered_state: None,
        }
    }

    pub fn render(&mut self, state: &AppState) -> Result<(), crate::errors::AppError> {
        // Only render if state is dirty or if this is the first render
        if !state.is_dirty() && self.last_rendered_state.is_some() {
            return Ok(());
        }

        // Sync canvas size if changed
        if self.canvas.width() as f64 != state.canvas_width || self.canvas.height() as f64 != state.canvas_height {
            self.canvas.set_width(state.canvas_width as u32);
            self.canvas.set_height(state.canvas_height as u32);
        }

        // Clear the canvas
        self.context.clear_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64);

        // Draw Shapes - sort by z_order to ensure proper layering
        let mut indexed_shapes: Vec<(usize, &Shape)> = state.shapes.iter().enumerate().collect();
        indexed_shapes.sort_by_key(|(_, shape)| shape.z_order);

        for (original_idx, shape) in indexed_shapes {
            // Draw the shape
            shape.draw(&self.context)?;

            // Highlight Selection
            if state.selected_index == Some(original_idx) {
                shape.draw_selection(&self.context)?;
            }
        }

        // Draw Interaction Preview
        if state.is_interacting && state.current_tool.needs_interaction() {
            self.context.set_stroke_style(&JsValue::from_str(PREVIEW_COLOR));
            self.context.set_line_width(1.0);
            let preview_shape = crate::interaction::create_shape_from_interaction(state, state.next_z_order);
            preview_shape.draw(&self.context)?;
        }

        // Draw Toolbar
        self.draw_toolbar(state)?;

        // Update last rendered state hash
        self.last_rendered_state = Some(self.calculate_state_hash(state));

        Ok(())
    }

    fn draw_toolbar(&self, state: &AppState) -> Result<(), crate::errors::AppError> {
        self.context.set_fill_style(&JsValue::from_str("#333333"));
        self.context.fill_rect(0.0, 0.0, state.canvas_width, TOOLBAR_HEIGHT);

        let tools = [Tool::Select, Tool::Rectangle, Tool::Circle, Tool::Line, Tool::Text, Tool::Delete];
        let labels = ["Select", "Rect", "Circle", "Line", "Text", "DELETE"];

        for (i, tool) in tools.iter().enumerate() {
            let x = i as f64 * BUTTON_WIDTH;

            // Set button color based on state
            let color = if state.current_tool == *tool {
                "#18A0FB"
            } else if *tool == Tool::Delete {
                "#F44336"
            } else {
                "#444444"
            };

            self.context.set_fill_style(&JsValue::from_str(color));
            self.context.fill_rect(x + 5.0, 5.0, BUTTON_WIDTH - 10.0, TOOLBAR_HEIGHT - 10.0);

            self.context.set_fill_style(&JsValue::from_str("#FFFFFF"));
            self.context.set_font("14px sans-serif");
            self.context
                .fill_text(labels[i], x + 10.0, 30.0)
                .map_err(|e| crate::errors::AppError::DomOperationFailed {
                    operation: format!("fill_text toolbar: {:?}", e),
                })?;
        }

        Ok(())
    }

    fn calculate_state_hash(&self, state: &AppState) -> usize {
        // Simple hash to detect state changes
        // In production, use a proper hashing algorithm
        let mut hash: usize = 0;
        hash = hash.wrapping_add(state.shapes.len());
        hash = hash.wrapping_add(state.selected_index.unwrap_or(0));
        hash = hash.wrapping_add(state.current_tool as usize);
        hash = hash.wrapping_add(state.is_interacting as usize);
        hash = hash.wrapping_add(state.next_z_order as usize);
        // Add position of selected shape if any
        if let Some(idx) = state.selected_index {
            if let Some(shape) = state.shapes.get(idx) {
                hash = hash.wrapping_add((shape.x as u32) as usize);
                hash = hash.wrapping_add((shape.y as u32) as usize);
            }
        }
        hash
    }
}

// Animation loop with dirty flag optimization
pub fn start_animation_loop<F>(_renderer: Rc<RefCell<Renderer>>, render_fn: F) -> Result<(), JsValue>
where
    F: Fn() -> Result<(), crate::errors::AppError> + 'static,
{
    let window = web_sys::window().unwrap();

    // Use a closure that can be called recursively
    let animation_loop = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let animation_loop_clone = animation_loop.clone();
    let window_clone = window.clone();

    *animation_loop_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Execute the render function
        if let Err(e) = render_fn() {
            web_sys::console::error_1(&format!("Render error: {:?}", e).into());
        }

        // Only request next frame if needed
        // In this implementation, we check the dirty flag in the render function
        window_clone.request_animation_frame(
            animation_loop.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap();
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(
        animation_loop_clone.borrow().as_ref().unwrap().as_ref().unchecked_ref()
    )?;

    // Keep the closure from being garbage collected
    animation_loop_clone.borrow_mut().take().unwrap().forget();

    Ok(())
}