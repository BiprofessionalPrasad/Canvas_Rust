use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
    Text,
}

impl ShapeType {
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ShapeType::Rectangle => "Rectangle",
            ShapeType::Circle => "Circle",
            ShapeType::Line => "Line",
            ShapeType::Text => "Text",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub z_order: u32,
    pub outline: bool,
}

impl Shape {
    #[must_use]
    pub fn new_rectangle(x: f64, y: f64, width: f64, height: f64, color: String, z_order: u32) -> Self {
        Self {
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width,
            height,
            x2: 0.0,
            y2: 0.0,
            color,
            text: String::new(),
            font_size: crate::constants::DEFAULT_FONT_SIZE,
            outline: false,
            z_order,
        }
    }

    #[must_use]
    pub fn new_circle(x: f64, y: f64, width: f64, height: f64, color: String, z_order: u32) -> Self {
        Self {
            shape_type: ShapeType::Circle,
            x,
            y,
            width,
            height,
            x2: 0.0,
            y2: 0.0,
            color,
            text: String::new(),
            font_size: crate::constants::DEFAULT_FONT_SIZE,
            outline: false,
            z_order,
        }
    }

    #[must_use]
    pub fn new_line(x1: f64, y1: f64, x2: f64, y2: f64, color: String, z_order: u32) -> Self {
        Self {
            shape_type: ShapeType::Line,
            x: x1,
            y: y1,
            width: 0.0,
            height: 0.0,
            x2,
            y2,
            color,
            text: String::new(),
            font_size: crate::constants::DEFAULT_FONT_SIZE,
            outline: false,
            z_order,
        }
    }

    #[must_use]
    pub fn new_text(x: f64, y: f64, text: String, color: String, font_size: f64, z_order: u32) -> Self {
        Self {
            shape_type: ShapeType::Text,
            x,
            y,
            width: crate::constants::DEFAULT_TEXT_WIDTH,
            height: crate::constants::DEFAULT_TEXT_HEIGHT,
            x2: 0.0,
            y2: 0.0,
            color,
            text,
            font_size,
            outline: false,
            z_order,
        }
    }

    pub fn draw(&self, context: &CanvasRenderingContext2d) -> Result<(), crate::errors::AppError> {
        context.set_fill_style_str(&self.color);
        context.set_stroke_style_str(&self.color);
        context.set_line_width(2.0);

        match self.shape_type {
            ShapeType::Rectangle => {
                if !self.outline {
                    context.fill_rect(self.x, self.y, self.width, self.height);
                }
                context.stroke_rect(self.x, self.y, self.width, self.height);
            }
            ShapeType::Circle => {
                context.begin_path();
                let center_x = self.x + self.width / 2.0;
                let center_y = self.y + self.height / 2.0;
                let radius = (self.width / 2.0).max(self.height / 2.0);
                context.arc(center_x, center_y, radius, 0.0, std::f64::consts::PI * 2.0)
                    .map_err(|e| crate::errors::AppError::DomOperationFailed {
                        operation: format!("arc: {:?}", e),
                    })?;
                if !self.outline {
                    context.fill();
                }
                context.stroke();
            }
            ShapeType::Line => {
                context.begin_path();
                context.move_to(self.x, self.y);
                context.line_to(self.x2, self.y2);
                context.stroke();
            }
            ShapeType::Text => {
                context.set_font(&format!("{}px sans-serif", self.font_size));
                context
                    .fill_text(&self.text, self.x, self.y + self.font_size)
                    .map_err(|e| crate::errors::AppError::DomOperationFailed {
                        operation: format!("fill_text: {:?}", e),
                    })?;
            }
        }

        Ok(())
    }

    pub fn draw_selection(&self, context: &CanvasRenderingContext2d) -> Result<(), crate::errors::AppError> {
        use crate::constants::*;

        context.set_stroke_style_str(SELECTION_COLOR);
        context.set_line_width(2.0);
        context
            .set_line_dash(&JsValue::from(js_sys::Array::of2(
                &JsValue::from_f64(DASH_PATTERN[0]),
                &JsValue::from_f64(DASH_PATTERN[1]),
            )))
            .map_err(|e| crate::errors::AppError::DomOperationFailed {
                operation: format!("set_line_dash: {:?}", e),
            })?;

        if self.shape_type == ShapeType::Line {
            context.stroke_rect(
                self.x.min(self.x2) - SELECTION_MARGIN,
                self.y.min(self.y2) - SELECTION_MARGIN,
                (self.x - self.x2).abs() + 2.0 * SELECTION_MARGIN,
                (self.y - self.y2).abs() + 2.0 * SELECTION_MARGIN,
            );
        } else {
            context.stroke_rect(
                self.x - SELECTION_MARGIN,
                self.y - SELECTION_MARGIN,
                self.width + 2.0 * SELECTION_MARGIN,
                self.height + 2.0 * SELECTION_MARGIN,
            );
        }

        context
            .set_line_dash(&JsValue::from(js_sys::Array::new()))
            .map_err(|e| crate::errors::AppError::DomOperationFailed {
                operation: format!("set_line_dash: {:?}", e),
            })?;

        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        match self.shape_type {
            ShapeType::Line => {
                let dist = crate::interaction::dist_to_segment(x, y, self.x, self.y, self.x2, self.y2);
                dist < crate::constants::LINE_PICK_DISTANCE
            }
            ShapeType::Circle => {
                let center_x = self.x + self.width / 2.0;
                let center_y = self.y + self.height / 2.0;
                let radius = (self.width / 2.0).max(self.height / 2.0);
                let dist = ((x - center_x).powi(2) + (y - center_y).powi(2)).sqrt();
                dist <= radius
            }
            _ => {
                x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
            }
        }
    }
}