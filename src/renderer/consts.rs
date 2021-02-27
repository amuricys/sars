use renderer::types::Color;

pub const BLACK: Color = [0.0, 0.0, 0.0, 0.0];
pub const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
pub const PURPLE: Color = [0.8, 0.0, 0.8, 1.0];
pub const PINK: Color = [1.0, 0.4, 1.0, 1.0];
pub const BLUE: Color = [0.2, 0.2, 1.0, 1.0];
pub const TURQUOISE: Color = [0.5, 0.6, 1.0, 1.0];
pub const GREEN: Color = [0.2, 1.0, 0.2, 1.0];
pub const RED: Color = [1.0, 0.0, 0.2, 1.0];
pub const _COLORS: [Color; 6] = [BLACK, WHITE, PURPLE, PINK, BLUE, GREEN];

pub const WINDOW_SIZE: (f64, f64) = (800.0, 800.0);
