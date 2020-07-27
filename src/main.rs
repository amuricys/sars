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


fn main() {
    let mut my_graph = graph::thick_surface(1.0, 0.2,  100);
    let my_change = NodeChange {id: 0, cur_x: my_graph.outer.nodes[0].x, cur_y: my_graph.outer.nodes[0].y, new_x: my_graph.outer.nodes[0].x - 0.2, new_y: my_graph.outer.nodes[0].y};
    let changeset = graph_change::smooth_change_out(&my_graph.outer, my_change, 3.5);
    graph_change::apply_changes(&mut my_graph.outer, changeset);

    let (mut window, mut renderer) = renderer::setup_renderer(Vec::from([my_graph.outer, my_graph.inner]));

    renderer::event_loop(&mut renderer, &mut window);
}
