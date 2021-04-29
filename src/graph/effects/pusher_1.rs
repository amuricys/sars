use graph::types::{ThickSurface, NodeChangeMap, OUTER};
use stitcher::types::Stitching;
use graph::effects::apply_changes;
use stitcher::{stitch_choice, stitch_default};

pub fn push_inners (ts: &ThickSurface, outer_changes: &NodeChangeMap,  s: &Stitching) -> NodeChangeMap {
    let mut ts_clone = ts.clone();
    apply_changes(&mut ts_clone.layers[OUTER], outer_changes);
    let intermediary_stitching = stitch_default(&ts_clone);
    /*
    We need some things to be preserved here.


    The important thing is not NUMBER of nodes pushed but the overall accumulated DISTANCE that one change covers.
    If the outer change has traversed a quarter of the surface
    1. Distance between pushed nodes has to be preserved.
    2.

    */
}