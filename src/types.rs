pub struct Params {
    pub initial_thickness: f64,
    pub initial_radius: f64,
    pub initial_num_points: usize,
    pub initial_temperature: f64,
    pub initial_gray_matter_area: f64,
    pub compression_factor: f64,
    pub softness_factor: f64, // <- how much should closeness of nodes in different surfaces impact pushes?
    pub how_smooth: usize,
    pub node_addition_threshold: f64,
    pub node_deletion_threshold: f64,
    pub low_high: (f64, f64),
    pub recorders: Vec<String>,
}
