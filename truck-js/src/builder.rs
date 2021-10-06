use crate::{wasm_bindgen, AbstractShape, Edge, Face, IntoWasm, Vertex, Wire};
use truck_modeling::*;

macro_rules! intopt {
    ($type: ty, $slice: ident) => {
        assert!(
            $slice.len() == 3,
            "{} is not a 3-dimentional!",
            stringify!($slice)
        );
        let $slice = <$type>::new($slice[0], $slice[1], $slice[2]);
    };
    ($type: ty, $slice: ident, $($a: ty, $b: ident),*) => {
        intopt!($type, $slice);
        intopt!($($a, $b),*);
    }
}

/// Creates and returns a vertex by a three dimensional point.
#[wasm_bindgen]
pub fn vertex(x: f64, y: f64, z: f64) -> Vertex { builder::vertex(Point3::new(x, y, z)).into() }
/// Returns a line from `vertex0` to `vertex1`.
#[wasm_bindgen]
pub fn line(vertex0: &Vertex, vertex1: &Vertex) -> Edge {
	builder::line(&*vertex0, &*vertex1).into()
}
/// Returns a circle arc from `vertex0` to `vertex1` via `transit`.
#[wasm_bindgen]
pub fn circle_arc(vertex0: &Vertex, vertex1: &Vertex, transit: &[f64]) -> Edge {
	intopt!(Point3, transit);
	builder::circle_arc(&*vertex0, &*vertex1, transit).into()
}
/// Returns a Bezier curve from `vertex0` to `vertex1` with inter control points `inter_points`.
#[wasm_bindgen]
pub fn bezier(vertex0: &Vertex, vertex1: &Vertex, inter_points: &[f64]) -> Edge {
	assert!(
		inter_points.len() % 3 == 0,
		"inter_points cannot convert to 3-dimentional points!"
	);
	let inter_points = inter_points
		.chunks(3)
		.map(|p| Point3::new(p[0], p[1], p[2]))
		.collect();
	builder::bezier(&*vertex0, &*vertex1, inter_points).into()
}
/// Returns a homotopic face from `edge0` to `edge1`.
#[wasm_bindgen]
pub fn homotopy(edge0: &Edge, edge1: &Edge) -> Face { builder::homotopy(&*edge0, &*edge1).into() }
/// Try attatiching a plane whose boundary is `wire`.
#[wasm_bindgen]
pub fn try_attach_plane(wire: &Wire) -> Option<Face> {
	builder::try_attach_plane(&[wire.0.clone()])
		.map(|face| face.into())
		.map_err(|e| eprintln!("{}", e))
		.ok()
}

macro_rules! transform_if_chain {
    ($shape: expr, $function: expr, ($($arg: expr),*), $exception: expr, $member: ident) => {
        if let Some(entity) = &$shape.$member {
            $function(&entity.0, $($arg),*).into_wasm().upcast()
        } else {
            $exception
        }
    };
    ($shape: expr, $function: expr, ($($arg: expr),*), $exception: expr, $member: ident, $($a: ident),*) => {
        if let Some(entity) = &$shape.$member {
            $function(&entity.0, $($arg),*).into_wasm().upcast()
        } else {
            transform_if_chain!($shape, $function, ($($arg),*), $exception, $($a),*)
        }
    }
}

macro_rules! derive_all_shape {
    ($shape: expr, $function: expr, ($($arg: expr),*)) => {
        transform_if_chain!(
            $shape,
            $function,
            ($($arg),*),
            unreachable!(),
            vertex,
            edge,
            wire,
            face,
            shell,
            solid
        )
    };
}

/// Returns a translated vertex, edge, wire, face, shell or solid.
#[wasm_bindgen]
pub fn translated(shape: &AbstractShape, vector: &[f64]) -> AbstractShape {
	intopt!(Vector3, vector);
	derive_all_shape!(shape, builder::translated, (vector))
}

/// Returns a rotated vertex, edge, wire, face, shell or solid.
#[wasm_bindgen]
pub fn rotated(shape: &AbstractShape, origin: &[f64], axis: &[f64], angle: f64) -> AbstractShape {
	intopt!(Point3, origin, Vector3, axis);
	derive_all_shape!(shape, builder::rotated, (origin, axis, Rad(angle)))
}

/// Returns a scaled vertex, edge, wire, face, shell or solid.
#[wasm_bindgen]
pub fn scaled(shape: &AbstractShape, origin: &[f64], scalars: &[f64]) -> AbstractShape {
	intopt!(Point3, origin);
	if scalars.len() == 1 {
		let s = Vector3::new(scalars[0], scalars[0], scalars[0]);
		derive_all_shape!(shape, builder::scaled, (origin, s))
	} else if scalars.len() == 3 {
		let s = Vector3::new(scalars[0], scalars[1], scalars[2]);
		derive_all_shape!(shape, builder::scaled, (origin, s))
	} else {
		panic!("The length of scalars is not 1 or 3.");
	}
}

macro_rules! derive_all_sweepable{
    ($shape: expr, $function: expr, ($($arg: expr),*)) => {
        transform_if_chain!(
            $shape,
            $function,
            ($($arg),*),
            panic!("sweep is only implemented to Vertex, Edge, Wire and Face."),
            vertex,
            edge,
            wire,
            face
        )
    };
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by a vector.
#[wasm_bindgen]
pub fn tsweep(shape: &AbstractShape, vector: &[f64]) -> AbstractShape {
	intopt!(Vector3, vector);
	derive_all_sweepable!(shape, builder::tsweep, (vector))
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by the rotation.
#[wasm_bindgen]
pub fn rsweep(shape: &AbstractShape, origin: &[f64], axis: &[f64], angle: f64) -> AbstractShape {
	intopt!(Point3, origin, Vector3, axis);
	derive_all_sweepable!(shape, builder::rsweep, (origin, axis, Rad(angle)))
}
