use graph::effects::apply_changes;
use graph::effects::helpers;
use graph::types::{Graph, Node, NodeChange, NodeChangeMap, ThickSurface, INNER, OUTER};
use graph::{closest_node_to_some_point, distance_between_nodes, distance_between_points};
use piston::input::keyboard::Key::Out;
use stitcher::types::Stitching;

fn outer_changes_to_blob(n: usize, inn: &Node, outer_changes: &NodeChangeMap, g: &Graph) {
    let sei_la = helpers::n_closest_outers(n, inn, outer_changes, g);
    let avg_ball_of_change = sei_la.iter().fold(
        NodeChange {
            id: 0,
            cur_x: 0.0,
            cur_y: 0.0,
            delta_x: 0.0,
            delta_y: 0.0,
        },
        |acc, x| NodeChange {
            id: 0,
            cur_x: acc.cur_x + x.cur_x / sei_la.len() as f64,
            cur_y: acc.cur_y + x.cur_y / sei_la.len() as f64,
            delta_x: acc.delta_x + x.delta_x / sei_la.len() as f64,
            delta_y: acc.delta_y + x.delta_y / sei_la.len() as f64,
        },
    );
    let cur_dist = distance_between_points(avg_ball_of_change.cur_x, avg_ball_of_change.cur_y, inn.x, inn.y);
}
