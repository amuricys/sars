use graph::distance_between_nodes;
use graph::types::Graph;
use pathfinding::directed::dijkstra::dijkstra;
use pathfinding::num_traits::Zero;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use stitcher::types;
use stitcher::types::Stitching;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RowCol(usize, usize);

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct MyFloat(f64);

impl Add for MyFloat {
    type Output = MyFloat;

    fn add(self, rhs: Self) -> Self::Output {
        MyFloat(self.0 + rhs.0)
    }
}

impl Zero for MyFloat {
    fn zero() -> Self {
        MyFloat(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}
impl Eq for MyFloat {}
impl Hash for MyFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{:?}", self.0).hash(state);
    }
}
impl Ord for MyFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (MyFloat(x), MyFloat(y)) => x.partial_cmp(y).unwrap(),
        }
    }
}

impl RowCol {
    fn successors(&self, mat: &Vec<Vec<(MyFloat, MyFloat)>>) -> Vec<(RowCol, MyFloat)> {
        match self {
            RowCol(row, col) if *row < mat.len() - 1 && *col < mat[0].len() - 1 => {
                vec![(RowCol(row + 1, *col), mat[*row][*col].0), (RowCol(*row, col + 1), mat[*row][*col].1)]
            }
            RowCol(row, col) if *col < mat[0].len() - 1 => vec![(RowCol(*row, col + 1), mat[*row][*col].1)],
            RowCol(row, col) if *row < mat.len() - 1 => vec![(RowCol(row + 1, *col), mat[*row][*col].0)],
            _ => vec![],
        }
    }
    fn success(&self, mat: &Vec<Vec<(MyFloat, MyFloat)>>) -> bool {
        self.0 == mat.len() - 1 && self.1 == mat[0].len() - 1
    }
}

/*
   Returns a matrix of dimensions innerN x outerN where each element is (CostOfMovingDown, CostOfMovingRight)
   in the matrix. Corresponds to distances between graphs. For example:

   matrix[i][j] = (
     distance from inner graph's ith node to outer graph's j+1th node,
     distance from outer graph's jth node to inner graph's i+1th node
   )
*/

fn make_matrix(outer: &Graph, inner: &Graph) -> Vec<Vec<(MyFloat, MyFloat)>> {
    let rows_amt = inner.nodes.len();
    let cols_amt = outer.nodes.len();
    let mut ret = vec![vec![(MyFloat(0.0), MyFloat(0.0)); cols_amt]; rows_amt];
    for row_i in 0..rows_amt {
        for col_i in 0..cols_amt {
            // Cost of moving down, right
            ret[row_i][col_i] = (
                MyFloat(distance_between_nodes(&outer.nodes[col_i], inner.next(row_i))),
                MyFloat(distance_between_nodes(&inner.nodes[row_i], outer.next(col_i))),
            );
        }
    }
    ret
}

fn give_me_a_path(outer: &Graph, inner: &Graph) -> Vec<RowCol> {
    let mat = make_matrix(outer, inner);
    let result = dijkstra(&RowCol(0, 0), |p| p.successors(&mat), |p| p.success(&mat));
    result.unwrap().0
}

pub fn stitch(outer: &Graph, inner: &Graph) -> types::Stitching {
    let shortest_path = give_me_a_path(outer, inner);
    let mut ret = Stitching::new();
    for RowCol(row, col) in shortest_path {
        ret.put(
            (row, inner.nodes[row].x, inner.nodes[row].y),
            (col, outer.nodes[col].x, outer.nodes[col].y),
        );
    }
    ret
}
