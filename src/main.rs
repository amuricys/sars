mod simulated_annealing;
mod vector_2d_helpers;
mod renderer;
mod graph_change;
mod types;
mod graph;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use types::NodeChange;
use simulated_annealing::step;

fn main() {
    let mut my_graph = graph::thick_surface(0.8, 0.2,  100);
    let my_change = NodeChange {id: 0, cur_x: my_graph.outer.nodes[0].x, cur_y: my_graph.outer.nodes[0].y, new_x: my_graph.outer.nodes[0].x + 0.2, new_y: my_graph.outer.nodes[0].y};
    let changeset = graph_change::smooth_change_out2(&my_graph.outer, my_change, 10);
    let inner_changeset = graph_change::changes_in_other_graph(&my_graph.outer, &changeset, &my_graph.inner);
    graph_change::apply_changes(&mut my_graph.outer, &changeset);
    graph_change::apply_changes(&mut my_graph.inner, &inner_changeset);

    step(&mut my_graph, 1.0);

    let (mut window, mut renderer) = renderer::setup_renderer(Vec::from([&my_graph.outer, &my_graph.inner]));

    renderer::event_loop(&mut renderer, &mut window);
}
