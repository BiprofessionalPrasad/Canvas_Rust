use crate::constants::*;
use crate::shapes::Shape;
use crate::tools::Tool;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
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
    pub next_z_order: u32,
    pub dirty_flag: bool,
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
            canvas_width: DEFAULT_CANVAS_WIDTH,
            canvas_height: DEFAULT_CANVAS_HEIGHT,
            next_z_order: 0,
            dirty_flag: true,
        }
    }

    pub fn delete_selected(&mut self) -> Result<(), crate::errors::AppError> {
        if let Some(idx) = self.selected_index {
            if idx < self.shapes.len() {
                self.shapes.remove(idx);
                self.selected_index = None;
                self.mark_dirty();
                Ok(())
            } else {
                Err(crate::errors::AppError::InvalidOperation {
                    operation: "delete_selected".to_string(),
                    reason: "Selected index out of bounds".to_string(),
                })
            }
        } else {
            Err(crate::errors::AppError::InvalidOperation {
                operation: "delete_selected".to_string(),
                reason: "No shape selected".to_string(),
            })
        }
    }

    pub fn add_shape(&mut self, shape: Shape) -> Result<(), crate::errors::AppError> {
        if self.shapes.len() >= MAX_SHAPES {
            return Err(crate::errors::AppError::InvalidOperation {
                operation: "add_shape".to_string(),
                reason: format!("Maximum number of shapes ({}) reached", MAX_SHAPES),
            });
        }
        self.shapes.push(shape);
        self.mark_dirty();
        Ok(())
    }

    pub fn update_shape(&mut self, index: usize, shape: Shape) -> Result<(), crate::errors::AppError> {
        if index >= self.shapes.len() {
            return Err(crate::errors::AppError::InvalidOperation {
                operation: "update_shape".to_string(),
                reason: "Index out of bounds".to_string(),
            });
        }
        self.shapes[index] = shape;
        self.mark_dirty();
        Ok(())
    }

    pub fn mark_dirty(&mut self) {
        self.dirty_flag = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty_flag = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_flag
    }

    pub fn get_selected_shape_mut(&mut self) -> Option<&mut Shape> {
        self.selected_index.and_then(|idx| self.shapes.get_mut(idx))
    }

    pub fn get_selected_shape(&self) -> Option<&Shape> {
        self.selected_index.and_then(|idx| self.shapes.get(idx))
    }

    pub fn select_next_shape(&mut self) -> bool {
        if self.shapes.is_empty() {
            return false;
        }

        let next_idx = match self.selected_index {
            None => 0,
            Some(idx) => (idx + 1) % self.shapes.len(),
        };

        self.selected_index = Some(next_idx);
        self.mark_dirty();
        true
    }

    pub fn select_previous_shape(&mut self) -> bool {
        if self.shapes.is_empty() {
            return false;
        }

        let prev_idx = match self.selected_index {
            None => self.shapes.len() - 1,
            Some(idx) => {
                if idx == 0 {
                    self.shapes.len() - 1
                } else {
                    idx - 1
                }
            }
        };

        self.selected_index = Some(prev_idx);
        self.mark_dirty();
        true
    }

    pub fn select_shape_at_position(&mut self, x: f64, y: f64) -> bool {
        for (idx, shape) in self.shapes.iter().enumerate().rev() {
            if crate::interaction::is_point_in_shape(x, y, shape) {
                self.selected_index = Some(idx);
                self.mark_dirty();
                return true;
            }
        }

        self.selected_index = None;
        self.mark_dirty();
        false
    }

    pub fn move_selected_shape(&mut self, dx: f64, dy: f64) -> Result<(), crate::errors::AppError> {
        if let Some(shape) = self.get_selected_shape_mut() {
            shape.x += dx;
            shape.y += dy;
            if shape.shape_type == crate::shapes::ShapeType::Line {
                shape.x2 += dx;
                shape.y2 += dy;
            }
            self.mark_dirty();
            Ok(())
        } else {
            Err(crate::errors::AppError::InvalidOperation {
                operation: "move_selected_shape".to_string(),
                reason: "No shape selected".to_string(),
            })
        }
    }
}

pub struct App {
    state: Rc<RefCell<AppState>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(AppState::new())),
        }
    }

    pub fn with_state<F, R>(&self, f: F) -> Result<R, crate::errors::AppError>
    where
        F: FnOnce(&mut AppState) -> Result<R, crate::errors::AppError>,
    {
        match self.state.try_borrow_mut() {
            Ok(mut guard) => f(&mut *guard),
            Err(_) => Err(crate::errors::AppError::StateAlreadyBorrowed),
        }
    }
}
