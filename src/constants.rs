// Constants for the canvas application
pub const TOOLBAR_HEIGHT: f64 = 50.0;
pub const BUTTON_WIDTH: f64 = 80.0;
pub const DEFAULT_FONT_SIZE: f64 = 20.0;
pub const SELECTION_MARGIN: f64 = 5.0;
pub const DASH_PATTERN: [f64; 2] = [5.0, 5.0];
pub const LINE_PICK_DISTANCE: f64 = 5.0;
pub const PREVIEW_COLOR: &str = "rgba(24, 160, 251, 0.5)";
pub const SELECTION_COLOR: &str = "#18A0FB";
pub const DEFAULT_SHAPE_COLOR: &str = "#E0E0E0";
pub const TEXT_DEFAULT: &str = "Text";
pub const DEFAULT_TEXT_WIDTH: f64 = 100.0;
pub const DEFAULT_TEXT_HEIGHT: f64 = 30.0;

// Canvas dimension limits
pub const MIN_DIMENSION: f64 = 100.0;
pub const MAX_DIMENSION: f64 = 10000.0;

// Font size limits
pub const MIN_FONT_SIZE: f64 = 6.0;
pub const MAX_FONT_SIZE: f64 = 72.0;

// Text limits
pub const MAX_TEXT_LENGTH: usize = 1000;

// Default canvas size
pub const DEFAULT_CANVAS_WIDTH: f64 = 800.0;
pub const DEFAULT_CANVAS_HEIGHT: f64 = 600.0;

// Maximum shapes to prevent memory exhaustion
pub const MAX_SHAPES: usize = 1000;

// Minimum drag distance to create a shape (prevents zero-size shapes on click)
pub const MIN_SHAPE_SIZE: f64 = 2.0;