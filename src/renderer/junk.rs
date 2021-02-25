/* These two are NOT INVERSES */
pub(crate) fn from_minus1_1_to_window(x: f64, y: f64, window_size_x: f64, window_size_y: f64) -> (f64, f64) {
    (x * window_size_x / 2.0, y * (-window_size_y / 2.0))
}

pub(crate) fn from_window_to_minus1_1(x: f64, y: f64, window_size_x: f64, window_size_y: f64) -> (f64, f64) {
    (2.0 * x / window_size_x - 1.0, (-2.0 * y / window_size_y) + 1.0)
}