use graph::types::{Graph};
use graph::cyclic_graph_from_coords;

// -----------------------------------
// Esqueleto: As duas funcoes abaixo não compilam ainda
// -----------------------------------
fn convex_hull_from_graph(g: &Graph) -> Graph {
    let intermediate_representation: geo::types::LineString = graph_to_line_string(g);
    let convex_hull_points: geo::types::Polygon = intermediate_representation.convex_hull();
    let vec_of_points: Vec<(f64, f64)> = line_string_to_vec_of_points(convex_hull_points.exterior);
    cyclic_graph_from_coords(&vec_of_points)
}

pub fn convex_hull (g: &Graph) -> Graph {
    let polygon = g.convex_hull();
    let vec_of_points: Vec<(f64, f64)> = line_string_to_vec_of_points(polygon.exterior);
    cyclic_graph_from_coords(&vec_of_points)
}