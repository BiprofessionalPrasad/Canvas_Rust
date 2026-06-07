use crate::shapes::Shape;
use crate::state::AppState;
use crate::tools::Tool;
use crate::constants::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

type AnimationClosure = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

pub struct Renderer {
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<CanvasRenderingContext2d>,
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement, context: CanvasRenderingContext2d) -> Self {
        Self {
            canvas: Rc::new(canvas),
            context: Rc::new(context),
        }
    }

    pub fn render(&self, state: &AppState) -> Result<(), crate::errors::AppError> {
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
            shape.draw(&self.context)?;

            if state.selected_index == Some(original_idx) {
                shape.draw_selection(&self.context)?;
            }
        }

        // Draw Interaction Preview
        if state.is_interacting && state.current_tool.needs_interaction() {
            self.context.set_stroke_style_str(PREVIEW_COLOR);
            self.context.set_line_width(1.0);
            let preview_shape = crate::interaction::create_shape_from_interaction(state, state.next_z_order);
            preview_shape.draw(&self.context)?;
        }

        // Draw Toolbar
        self.draw_toolbar(state)?;

        Ok(())
    }

    fn draw_toolbar(&self, state: &AppState) -> Result<(), crate::errors::AppError> {
        self.context.set_fill_style_str("#333333");
        self.context.fill_rect(0.0, 0.0, state.canvas_width, TOOLBAR_HEIGHT);

        let tools = [Tool::Select, Tool::Rectangle, Tool::Circle, Tool::Line, Tool::Text, Tool::Delete];
        let labels = ["Select", "Rect", "Circle", "Line", "Text", "DELETE"];

        for (i, tool) in tools.iter().enumerate() {
            let x = i as f64 * BUTTON_WIDTH;

            let color = if state.current_tool == *tool {
                "#18A0FB"
            } else if *tool == Tool::Delete {
                "#F44336"
            } else {
                "#444444"
            };

            self.context.set_fill_style_str(color);
            self.context.fill_rect(x + 5.0, 5.0, BUTTON_WIDTH - 10.0, TOOLBAR_HEIGHT - 10.0);

            self.context.set_fill_style_str("#FFFFFF");
            self.context.set_font("14px sans-serif");
            self.context
                .fill_text(labels[i], x + 10.0, 30.0)
                .map_err(|e| crate::errors::AppError::DomOperationFailed {
                    operation: format!("fill_text toolbar: {:?}", e),
                })?;
        }

        Ok(())
    }
}

// Animation loop that stops when idle, supports re-entrant start
pub struct AnimationLoop {
    active: Rc<std::cell::Cell<bool>>,
    closure: AnimationClosure,
}

impl AnimationLoop {
    pub fn new<F>(render_fn: F) -> Self
    where
        F: Fn() -> Result<bool, crate::errors::AppError> + 'static,
    {
        let active = Rc::new(std::cell::Cell::new(false));
        let closure: AnimationClosure = Rc::new(RefCell::new(None));

        let closure_weak = Rc::downgrade(&closure);
        let active_clone = active.clone();

        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if !active_clone.get() {
                return;
            }

            match render_fn() {
                Ok(true) => {
                    // Rendered successfully, continue loop
                    if active_clone.get() {
                        if let Some(closure_rc) = closure_weak.upgrade() {
                            if let Some(ref c) = *closure_rc.borrow() {
                                let _ = web_sys::window()
                                    .and_then(|w| {
                                        w.request_animation_frame(c.as_ref().unchecked_ref()).ok()
                                    });
                            }
                        }
                    }
                }
                Ok(false) => {
                    // Nothing to render, stop the loop
                    active_clone.set(false);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Render error: {:?}", e).into());
                    active_clone.set(false);
                }
            }
        }) as Box<dyn FnMut()>));

        Self { active, closure }
    }

    pub fn start(&self) {
        // Set active before scheduling to avoid race
        self.active.set(true);
        if let Some(ref c) = *self.closure.borrow() {
            if let Some(window) = web_sys::window() {
                let _ = window.request_animation_frame(c.as_ref().unchecked_ref());
            }
        }
    }

}
