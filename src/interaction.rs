use crate::shapes::Shape;
use crate::state::AppState;
use crate::tools::Tool;
use crate::constants::*;

pub fn is_point_in_shape(x: f64, y: f64, shape: &Shape) -> bool {
    shape.contains_point(x, y)
}

pub fn dist_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let l2 = (x1 - x2).powi(2) + (y1 - y2).powi(2);
    if l2 == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = ((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / l2;
    let t = t.max(0.0).min(1.0);
    ((px - (x1 + t * (x2 - x1))).powi(2) + (py - (y1 + t * (y2 - y1))).powi(2)).sqrt()
}

pub fn create_shape_from_interaction(state: &AppState, z_order: u32) -> Shape {
    let x = state.start_x.min(state.current_x);
    let y = state.start_y.min(state.current_y);
    let width = (state.start_x - state.current_x).abs();
    let height = (state.start_y - state.current_y).abs();

    match state.current_tool {
        Tool::Rectangle => Shape::new_rectangle(x, y, width, height, DEFAULT_SHAPE_COLOR.to_string(), z_order),
        Tool::Circle => Shape::new_circle(x, y, width, height, DEFAULT_SHAPE_COLOR.to_string(), z_order),
        Tool::Line => Shape::new_line(state.start_x, state.start_y, state.current_x, state.current_y, DEFAULT_SHAPE_COLOR.to_string(), z_order),
        Tool::Text => Shape::new_text(state.start_x, state.start_y, TEXT_DEFAULT.to_string(), DEFAULT_TEXT_COLOR.to_string(), DEFAULT_FONT_SIZE, z_order),
        _ => unreachable!(),
    }
}

pub fn handle_toolbar_click(x: f64, state: &mut AppState) -> bool {
    if x < 0.0 || x >= state.canvas_width {
        return false;
    }

    let tool_idx = (x / BUTTON_WIDTH) as usize;
    match tool_idx {
        0 => {
            state.current_tool = Tool::Select;
            state.selected_index = None;
        }
        1 => state.current_tool = Tool::Rectangle,
        2 => state.current_tool = Tool::Circle,
        3 => state.current_tool = Tool::Line,
        4 => state.current_tool = Tool::Text,
        5 => {
            if let Err(e) = state.delete_selected() {
                web_sys::console::error_1(&format!("Failed to delete selected shape: {:?}", e).into());
            }
        }
        _ => return false,
    }

    web_sys::console::log_1(&format!("Tool selected: {:?}", state.current_tool).into());
    state.mark_dirty();
    true
}

pub fn handle_mouse_down(mouse_x: f64, mouse_y: f64, state: &mut AppState) -> Result<(), crate::errors::AppError> {
    // Toolbar interaction
    if mouse_y < TOOLBAR_HEIGHT {
        handle_toolbar_click(mouse_x, state);
        return Ok(());
    }

    // Canvas interaction
    state.start_x = mouse_x;
    state.start_y = mouse_y;
    state.current_x = mouse_x;
    state.current_y = mouse_y;
    state.is_interacting = true;

    if state.current_tool == Tool::Select {
        state.select_shape_at_position(mouse_x, mouse_y);
    }

    Ok(())
}

pub fn handle_mouse_move(mouse_x: f64, mouse_y: f64, state: &mut AppState) -> Result<bool, crate::errors::AppError> {
    if !state.is_interacting {
        return Ok(false);
    }

    let dx = mouse_x - state.current_x;
    let dy = mouse_y - state.current_y;

    state.current_x = mouse_x;
    state.current_y = mouse_y;

    if state.current_tool == Tool::Select {
        state.move_selected_shape(dx, dy)?;
    }

    Ok(true)
}

pub fn handle_mouse_up(state: &mut AppState) -> Result<bool, crate::errors::AppError> {
    if !state.is_interacting {
        return Ok(false);
    }

    if state.current_tool.creates_shape() {
        let z_order = state.next_z_order;
        state.next_z_order += 1;
        let new_shape = create_shape_from_interaction(state, z_order);
        state.add_shape(new_shape)?;
        state.selected_index = Some(state.shapes.len() - 1);
    }

    state.is_interacting = false;
    Ok(true)
}

pub fn handle_keyboard_navigation(key: &str, state: &mut AppState) -> Result<bool, crate::errors::AppError> {
    match key {
        "ArrowLeft" | "ArrowUp" => {
            let changed = state.select_previous_shape();
            if changed {
                // Announce to screen readers
                if let Some(shape) = state.get_selected_shape() {
                    announce_to_screen_reader(&format!("Selected {} at ({}, {})",
                        shape.shape_type.as_str(), shape.x, shape.y));
                }
            }
            Ok(changed)
        }
        "ArrowRight" | "ArrowDown" => {
            let changed = state.select_next_shape();
            if changed {
                // Announce to screen readers
                if let Some(shape) = state.get_selected_shape() {
                    announce_to_screen_reader(&format!("Selected {} at ({}, {})",
                        shape.shape_type.as_str(), shape.x, shape.y));
                }
            }
            Ok(changed)
        }
        "Delete" | "Backspace" => {
            if let Err(e) = state.delete_selected() {
                announce_to_screen_reader(&format!("Error deleting shape: {}", e));
            } else {
                announce_to_screen_reader("Shape deleted");
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn announce_to_screen_reader(message: &str) {
    // Create or update ARIA live region for screen reader announcements
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let live_region = match document.get_element_by_id("sr-announcements") {
                Some(element) => element,
                None => {
                    if let Ok(element) = document.create_element("div") {
                        let _ = element.set_id("sr-announcements");
                        let _ = element.set_attribute("aria-live", "polite");
                        let _ = element.set_attribute("aria-atomic", "true");
                        let _ = element.set_attribute("style", "position: absolute; left: -10000px; width: 1px; height: 1px; overflow: hidden;");
                        if let Some(body) = document.body() {
                            let _ = body.append_child(&element);
                        }
                        element
                    } else {
                        return;
                    }
                }
            };

            live_region.set_text_content(Some(message));
        }
    }
}