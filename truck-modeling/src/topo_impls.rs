use rustc_hash::FxHashMap as HashMap;
use truck_topology::*;

pub(super) fn create_edge<P: Clone, C: Clone, CP: Fn(&P, &P) -> C>(
    v0: &Vertex<P>,
    v1: &Vertex<P>,
    connect_points: CP,
) -> C {
    connect_points(&v0.point(), &v1.point())
}

pub(super) fn connect_vertices<P: Clone, C: Clone, CP: Fn(&P, &P) -> C>(
    v0: &Vertex<P>,
    v1: &Vertex<P>,
    connect_points: CP,
) -> Edge<P, C> {
    Edge::debug_new(v0, v1, create_edge(v0, v1, connect_points))
}

pub(super) fn create_surface<P: Clone, C: Clone, S: Clone, CC: Fn(&C, &C) -> S>(
    edge0: &Edge<P, C>,
    edge1: &Edge<P, C>,
    connect_curves: CC,
) -> S {
    connect_curves(&edge0.curve(), &edge1.curve())
}

pub(super) fn connect_edges<
    P: Clone,
    C: Clone,
    S: Clone,
    CP: Fn(&P, &P) -> C,
    CC: Fn(&C, &C) -> S,
>(
    edge0: &Edge<P, C>,
    edge1: &Edge<P, C>,
    connect_points: CP,
    connect_curves: CC,
) -> Face<P, C, S> {
    let edge2 = connect_vertices(edge0.front(), edge1.front(), &connect_points);
    let edge3 = connect_vertices(edge0.back(), edge1.back(), connect_points);
    let surface = create_surface(edge0, edge1, connect_curves);
    let wire: Wire<P, C> = match edge0.orientation() {
        true => vec![edge0.clone(), edge3, edge1.inverse(), edge2.inverse()].into(),
        false => vec![edge0.inverse(), edge2, edge1.clone(), edge3.inverse()].into(),
    };
    let mut face = Face::debug_new(vec![wire], surface);
    if !edge0.orientation() {
        face.invert();
    }
    face
}

fn sub_connect_wires<P: Clone, C: Clone, S: Clone, CP: Fn(&P, &P) -> C, CC: Fn(&C, &C) -> S>(
    edge0: &Edge<P, C>,
    edge1: &Edge<P, C>,
    connect_points: CP,
    connect_curves: CC,
    vemap: &mut HashMap<VertexID<P>, Edge<P, C>>,
) -> Face<P, C, S> {
    let edge2 = vemap
        .entry(edge0.front().id())
        .or_insert_with(|| connect_vertices(edge0.front(), edge1.front(), &connect_points))
        .clone();
    let edge3 = vemap
        .entry(edge0.back().id())
        .or_insert_with(|| connect_vertices(edge0.back(), edge1.back(), connect_points))
        .clone();
    let ori = edge0.orientation();
    let wire = match ori {
        true => Wire::from(vec![edge0.clone(), edge3, edge1.inverse(), edge2.inverse()]),
        false => Wire::from(vec![edge2, edge1.clone(), edge3.inverse(), edge0.inverse()]),
    };
    let surface = create_surface(edge0, edge1, connect_curves);
    let mut face = Face::debug_new(vec![wire], surface);
    if !ori {
        face.invert();
    }
    face
}

pub(super) fn connect_wires<
    'a,
    P: 'a + Clone,
    C: 'a + Clone,
    S: 'a + Clone,
    CP: 'a + Fn(&P, &P) -> C,
    CC: 'a + Fn(&C, &C) -> S,
    I: IntoIterator<Item = &'a Edge<P, C>> + 'a,
>(
    wire0: I,
    wire1: I,
    connect_points: CP,
    connect_curves: CC,
) -> impl Iterator<Item = Face<P, C, S>> + 'a {
    let mut vemap = HashMap::<VertexID<P>, Edge<P, C>>::default();
    wire0.into_iter().zip(wire1).map(move |(edge0, edge1)| {
        sub_connect_wires(edge0, edge1, &connect_points, &connect_curves, &mut vemap)
    })
}

pub(super) fn connect_raw_wires<
    'a,
    P: 'a + Clone,
    C: 'a + Clone,
    S: 'a + Clone,
    CP: 'a + Fn(&P, &P) -> C,
    CC: 'a + Fn(&C, &C) -> S,
    I: IntoIterator<Item = Edge<P, C>> + 'a,
>(
    wire0: I,
    wire1: I,
    connect_points: CP,
    connect_curves: CC,
) -> impl Iterator<Item = Face<P, C, S>> + 'a {
    let mut vemap = HashMap::<VertexID<P>, Edge<P, C>>::default();
    wire0.into_iter().zip(wire1).map(move |(edge0, edge1)| {
        sub_connect_wires(&edge0, &edge1, &connect_points, &connect_curves, &mut vemap)
    })
}
