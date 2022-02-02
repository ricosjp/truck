use super::*;
use rustc_hash::FxHashMap as HashMap;
use std::f64::consts::PI;

/// subdivision surface algorithms
pub trait Subdivision {
    /// Extended Loop method
    ///
    /// # Remarks
    /// Confirm:
    /// - All faces are triangles.
    /// - `self.shell_condition()` is `Oriented` or `Closed` before use.
    /// This method does NOT check these conditions.
    fn loop_subdivision(&mut self) -> &mut Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Edge(usize, usize);
#[derive(Clone, Copy, Debug)]
struct EdgeInfo {
    idx: usize,
    first_wing: usize,
    second_wing: Option<usize>,
}

impl Edge {
    #[inline]
    fn new(v0: usize, v1: usize) -> Self { Edge(usize::min(v0, v1), usize::max(v0, v1)) }
}

impl EdgeInfo {
    #[inline]
    fn new(idx: usize, first_wing: usize) -> Self {
        Self {
            idx,
            first_wing,
            second_wing: None,
        }
    }
    #[inline]
    fn add_second_wing(&mut self, second_wing: usize) { self.second_wing = Some(second_wing); }
}

impl Subdivision for PolygonMesh {
    fn loop_subdivision(&mut self) -> &mut Self {
        let mut edges = HashMap::default();
        let mut vertex_adjacency = vec![Vec::new(); self.positions().len()];
        self.tri_faces().iter().for_each(|face| {
            let face = [face[0].pos, face[1].pos, face[2].pos];
            add_vertex_edge(&mut edges, &mut vertex_adjacency, face[1], face[2], face[0]);
            add_vertex_edge(&mut edges, &mut vertex_adjacency, face[2], face[0], face[1]);
            add_vertex_edge(&mut edges, &mut vertex_adjacency, face[0], face[1], face[2]);
        });
        let positions = vertex_adjacency
            .iter()
            .enumerate()
            .map(|t| calc_new_position(t.0, t.1, &edges, self.positions()))
            .chain(edge_positions(&edges, &vertex_adjacency, self.positions()))
            .collect::<Vec<_>>();
        let tri_faces = self
            .tri_faces()
            .iter()
            .flat_map(|v| {
                let len = self.positions().len();
                let e: [StandardVertex; 3] = [
                    (edges.get(&Edge::new(v[1].pos, v[2].pos)).unwrap().idx + len).into(),
                    (edges.get(&Edge::new(v[2].pos, v[0].pos)).unwrap().idx + len).into(),
                    (edges.get(&Edge::new(v[0].pos, v[1].pos)).unwrap().idx + len).into(),
                ];
                vec![
                    [v[0], e[2], e[1]],
                    [e[0], v[2], e[1]],
                    [e[0], e[2], v[1]],
                    [e[0], e[1], e[2]],
                ]
            })
            .collect::<Vec<_>>();
        {
            let editor = self.debug_editor();
            editor.attributes.positions = positions;
            *editor.faces = Faces::from_tri_and_quad_faces(tri_faces, Vec::new());
        }
        self
    }
}

fn add_vertex_edge(
    edges: &mut HashMap<Edge, EdgeInfo>,
    vertex_adjacency: &mut Vec<Vec<usize>>,
    v0: usize,
    v1: usize,
    v2: usize,
) {
    let idx = edges.len();
    edges
        .entry(Edge::new(v0, v1))
        .and_modify(|tuple| tuple.add_second_wing(v2))
        .or_insert_with(|| {
            vertex_adjacency[v0].push(v1);
            vertex_adjacency[v1].push(v0);
            EdgeInfo::new(idx, v2)
        });
}

#[derive(Clone, Copy, Debug)]
enum VertexBoundaryCondition {
    Corner(usize, usize),
    Boundary(usize, usize),
    Inner,
}

impl VertexBoundaryCondition {
    fn new(v: usize, adjacency: &[usize], edges: &HashMap<Edge, EdgeInfo>) -> Self {
        let binfo = adjacency.iter().copied().fold((None, None), |binfo, w| {
            let edge = edges.get(&Edge::new(v, w)).unwrap();
            match (edge.second_wing, binfo) {
                (None, (None, _)) => (Some(w), None),
                (None, (Some(x), None)) => (Some(x), Some(w)),
                _ => binfo,
            }
        });
        if let (Some(x), Some(y)) = binfo {
            if adjacency.len() == 2 {
                Self::Corner(x, y)
            } else {
                Self::Boundary(x, y)
            }
        } else {
            Self::Inner
        }
    }
}

fn calc_new_position(
    v: usize,
    adjacency: &[usize],
    edges: &HashMap<Edge, EdgeInfo>,
    positions: &[Point3],
) -> Point3 {
    use VertexBoundaryCondition::*;
    let point = positions[v];
    match VertexBoundaryCondition::new(v, adjacency, edges) {
        Corner(_, _) => point,
        Boundary(w0, w1) => point + ((positions[w0] - point) + (positions[w1] - point)) / 8.0,
        Inner => {
            let alpha = 3.0 / 8.0 + f64::cos(2.0 * PI / adjacency.len() as f64) / 4.0;
            let alpha = (5.0 / 8.0 - alpha * alpha) / adjacency.len() as f64;
            point
                + adjacency
                    .iter()
                    .copied()
                    .map(|w| positions[w] - point)
                    .sum::<Vector3>()
                    * alpha
        }
    }
}

fn edge_positions(
    edges: &HashMap<Edge, EdgeInfo>,
    vertex_adjacency: &[Vec<usize>],
    positions: &[Point3],
) -> Vec<Point3> {
    use VertexBoundaryCondition::*;
    let mut res = vec![Point3::origin(); edges.len()];
    edges.iter().for_each(|tuple| {
        let Edge(v0, v1) = *tuple.0;
        let EdgeInfo {
            idx,
            first_wing,
            second_wing,
        } = *tuple.1;
        res[idx] = match second_wing {
            Some(second_wing) => {
                let gamma = match VertexBoundaryCondition::new(v0, &vertex_adjacency[v0], edges) {
                    Boundary(_, _) => {
                        5.0 / 8.0 - f64::cos(PI / vertex_adjacency[v0].len() as f64) / 4.0
                    }
                    Inner => match VertexBoundaryCondition::new(v1, &vertex_adjacency[v1], edges) {
                        Boundary(_, _) => {
                            3.0 / 8.0 + f64::cos(PI / vertex_adjacency[v1].len() as f64) / 4.0
                        }
                        Inner => 0.5,
                        Corner(_, _) => unreachable!(),
                    },
                    Corner(_, _) => unreachable!(),
                };
                positions[v0]
                    + (positions[v1] - positions[v0]) * gamma
                    + (positions[first_wing] - positions[v0]) / 8.0
                    + (positions[second_wing] - positions[v1]) / 8.0
            }
            None => positions[v0].midpoint(positions[v1]),
        };
    });
    res
}
