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
    #[inline]
    #[must_use]
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

    #[inline]
    #[must_use]
    pub fn creates_shape(&self) -> bool {
        matches!(self, Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Text)
    }

    #[inline]
    #[must_use]
    pub fn needs_interaction(&self) -> bool {
        !matches!(self, Tool::Select | Tool::Delete)
    }
}