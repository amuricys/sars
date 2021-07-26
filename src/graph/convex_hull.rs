use graph::types::{Graph};
use graph::cyclic_graph_from_coords;
use geo::algorithm::convex_hull::ConvexHull;

// -----------------------------------
// Esqueleto: As duas funcoes abaixo não compilam ainda
// -----------------------------------
fn convex_hull_from_graph(g: &Graph) -> Graph {
    let intermediate_representation_points_vec: Vec<(f64, f64)> = g.to_vec_of_points();
    let line_string_representation: geo::LineString<f64> = vec_of_points_to_line_string(&intermediate_representation_points_vec);
    let convex_hull_points: geo::Polygon<f64> = line_string_representation.convex_hull();
    let vec_of_points: Vec<(f64, f64)> = line_string_to_vec_of_points(&convex_hull_points.exterior());
    cyclic_graph_from_coords(&vec_of_points)
}

fn line_string_to_vec_of_points (l: &geo::LineString<f64>) -> Vec<(f64, f64)> {
    let mut coord = vec![];
    for i in 0..l.0.len() {
        let vec_inside_linestring = &l.0;
        let coordinate = vec_inside_linestring[i];
        coord.push((coordinate.x, coordinate.y));
    }
    coord
}

fn vec_of_points_to_line_string (l: &Vec<(f64, f64)>) -> geo::LineString<f64> {
    geo::LineString::from( l.clone() )
}