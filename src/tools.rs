use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Tool {
    Select,
    Rectangle,
    Circle,
    Line,
    Text,
    Delete,
}

impl Tool {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Rectangle => "Rectangle",
            Tool::Circle => "Circle",
            Tool::Line => "Line",
            Tool::Text => "Text",
            Tool::Delete => "Delete",
        }
    }

    pub fn creates_shape(&self) -> bool {
        matches!(self, Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Text)
    }

    pub fn needs_interaction(&self) -> bool {
        !matches!(self, Tool::Select | Tool::Delete)
    }
}