use types::ThickSurface;
use graph::{cyclic_graph_from_coords};
use vec1::Vec1;

pub fn diagonal_ts() -> ThickSurface {
    let mut coords_outer = Vec1::new((-1.0, -1.0));
    coords_outer.push((-0.8, 0.85));
    coords_outer.push((-0.4, 0.45));
    coords_outer.push((-0.0, 0.05));
    coords_outer.push((0.4, -0.35));
    coords_outer.push((0.8, -0.75));

    let mut coords_inner = Vec1::new((-1.0, -1.0));
    coords_inner.push((-0.8, 0.75));
    coords_inner.push((-0.4, 0.35));
    coords_inner.push((0.0, -0.05));
    coords_inner.push((0.4, -0.45));
    coords_inner.push((0.8, -0.85));

    let mut outer_graph = cyclic_graph_from_coords(&coords_outer);
    let mut inner_graph = cyclic_graph_from_coords(&coords_inner);

    ThickSurface {layers: vec![outer_graph, inner_graph]}
}
