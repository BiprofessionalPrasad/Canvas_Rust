use crate::App;
use crate::validation::validate_mouse_position;
use crate::interaction::{handle_mouse_down, handle_mouse_move, handle_mouse_up, handle_keyboard_navigation};
use web_sys::{HtmlCanvasElement, MouseEvent, KeyboardEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;

pub struct EventHandlers;

impl EventHandlers {
    pub fn new() -> Self {
        Self
    }

    pub fn setup_all(
        &self,
        app: Rc<App>,
        canvas: &HtmlCanvasElement,
        render_callback: Rc<RefCell<Box<dyn FnMut()>>>,
    ) -> Result<(), JsValue> {
        Self::setup_mouse_events(app.clone(), canvas, render_callback.clone())?;
        Self::setup_keyboard_events(app.clone(), render_callback.clone())?;
        Ok(())
    }

    fn setup_mouse_events(
        app: Rc<App>,
        canvas: &HtmlCanvasElement,
        render_callback: Rc<RefCell<Box<dyn FnMut()>>>,
    ) -> Result<(), JsValue> {
        // Mouse down event
        {
            let app_clone = app.clone();
            let render_callback_clone = render_callback.clone();

            let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                let mouse_x = event.offset_x() as f64;
                let mouse_y = event.offset_y() as f64;

                let should_render = app_clone.with_state(|state| {
                    // Validate mouse position
                    if let Err(e) = validate_mouse_position(mouse_x, mouse_y, state.canvas_width, state.canvas_height) {
                        web_sys::console::log_1(&format!("Warning: {:?}", e).into());
                        crate::errors::update_status(&e.to_string());
                        return Ok(false);
                    }

                    handle_mouse_down(mouse_x, mouse_y, state)?;
                    Ok(state.is_dirty())
                }).unwrap_or_else(|e| {
                    web_sys::console::error_1(&format!("Error in mouse down: {:?}", e).into());
                    false
                });

                if should_render {
                    (*render_callback_clone.borrow_mut())();
                }
            });

            canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            // Leak the closure to keep it alive for the duration of the page
            closure.forget();
        }

        // Mouse move event
        {
            let app_clone = app.clone();
            let render_callback_clone = render_callback.clone();

            let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                let mouse_x = event.offset_x() as f64;
                let mouse_y = event.offset_y() as f64;

                let should_render = app_clone.with_state(|state| {
                    handle_mouse_move(mouse_x, mouse_y, state)
                }).unwrap_or_else(|e| {
                    web_sys::console::error_1(&format!("Error in mouse move: {:?}", e).into());
                    false
                });

                if should_render {
                    (*render_callback_clone.borrow_mut())();
                }
            });

            canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Mouse up event
        {
            let app_clone = app.clone();
            let render_callback_clone = render_callback.clone();

            let closure = Closure::<dyn FnMut(_)>::new(move |_event: MouseEvent| {
                let should_render = app_clone.with_state(|state| {
                    handle_mouse_up(state)
                }).unwrap_or_else(|e| {
                    web_sys::console::error_1(&format!("Error in mouse up: {:?}", e).into());
                    false
                });

                if should_render {
                    (*render_callback_clone.borrow_mut())();
                }
            });

            canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }

    fn setup_keyboard_events(
        app: Rc<App>,
        render_callback: Rc<RefCell<Box<dyn FnMut()>>>,
    ) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();

        let closure = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let key = event.key();

            // Handle arrow keys for navigation and delete/backspace for deletion
            match key.as_str() {
                "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Delete" | "Backspace" => {
                    let should_render = app.with_state(|state| {
                        handle_keyboard_navigation(&key, state)
                    }).unwrap_or_else(|e| {
                        web_sys::console::error_1(&format!("Error in keyboard handler: {:?}", e).into());
                        false
                    });

                    if should_render {
                        (*render_callback.borrow_mut())();

                        // Prevent default browser behavior for arrow keys and delete
                        event.prevent_default();
                    }
                }
                _ => {}
            }
        });

        window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }
}