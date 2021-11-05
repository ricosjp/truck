use crate::*;
use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_topology::*;

/// Only solids consisting of faces whose surface is implemented this trait can be used for set operations.
pub trait ShapeOpsSurface:
	ParametricSurface3D
	+ ParameterDivision2D
	+ SearchParameter<Point = Point3, Parameter = (f64, f64)>
	+ SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
	+ Invertible {
}
impl<S> ShapeOpsSurface for S where S: ParametricSurface3D
		+ ParameterDivision2D
		+ SearchParameter<Point = Point3, Parameter = (f64, f64)>
		+ SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
		+ Invertible
{
}

/// Only solids consisting of edges whose curve is implemented this trait can be used for set operations.
pub trait ShapeOpsCurve<S: ShapeOpsSurface>:
	ParametricCurve3D
	+ ParameterDivision1D<Point = Point3>
	+ Cut
	+ Invertible
	+ From<IntersectionCurve<PolylineCurve<Point3>, S>>
	+ SearchParameter<Point = Point3, Parameter = f64>
	+ SearchNearestParameter<Point = Point3, Parameter = f64> {
}
impl<C, S: ShapeOpsSurface> ShapeOpsCurve<S> for C where C: ParametricCurve3D
		+ ParameterDivision1D<Point = Point3>
		+ Cut
		+ Invertible
		+ From<IntersectionCurve<PolylineCurve<Point3>, S>>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>
{
}

fn process_one_pair_of_shells<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
	shell0: &Shell<Point3, C, S>,
	shell1: &Shell<Point3, C, S>,
	tol: f64,
) -> Option<[Shell<Point3, C, S>; 2]> {
	nonpositive_tolerance!(tol);
	let poly_shell0 = shell0.triangulation(tol)?;
	let poly_shell1 = shell1.triangulation(tol)?;
	let loops_store::LoopsStoreQuadruple {
		geom_loops_store0: loops_store0,
		geom_loops_store1: loops_store1,
		..
	} = loops_store::create_loops_stores(shell0, &poly_shell0, shell1, &poly_shell1, tol)?;
	let mut cls0 = divide_face::divide_faces(shell0, &loops_store0, tol)?;
	cls0.integrate_by_component();
	let mut cls1 = divide_face::divide_faces(shell1, &loops_store1, tol)?;
	cls1.integrate_by_component();
	let [mut and0, mut or0, unknown0] = cls0.and_or_unknown();
	unknown0.into_iter().for_each(|face| {
		let pt = face.boundaries()[0]
			.vertex_iter()
			.next()
			.unwrap()
			.get_point();
		let mut dir = random_normal_3d();
		if dir.so_small() {
			dir = random_normal_3d();
		}
		dir = dir.normalize();
		let count = poly_shell1.iter().fold(0, |count, face| {
			let poly = face.get_surface();
			count + poly.signed_crossing_faces(pt, dir)
		});
		if count >= 1 {
			and0.push(face);
		} else {
			or0.push(face);
		}
	});
	let [mut and1, mut or1, unknown1] = cls1.and_or_unknown();
	unknown1.into_iter().for_each(|face| {
		let pt = face.boundaries()[0]
			.vertex_iter()
			.next()
			.unwrap()
			.get_point();
		let mut dir = random_normal_3d();
		if dir.so_small() {
			dir = random_normal_3d();
		}
		dir = dir.normalize();
		let count = poly_shell0.iter().fold(0, |count, face| {
			let poly = face.get_surface();
			count + poly.signed_crossing_faces(pt, dir)
		});
		if count >= 1 {
			and1.push(face);
		} else {
			or1.push(face);
		}
	});
	and0.append(&mut and1);
	or0.append(&mut or1);
	Some([and0, or0])
}

/// AND operation between two solids.
pub fn and<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
	solid0: &Solid<Point3, C, S>,
	solid1: &Solid<Point3, C, S>,
	tol: f64,
) -> Option<Solid<Point3, C, S>> {
	let mut iter0 = solid0.boundaries().iter();
	let mut iter1 = solid1.boundaries().iter();
	let shell0 = iter0.next().unwrap();
	let shell1 = iter1.next().unwrap();
	let [mut and_shell, _] = process_one_pair_of_shells(shell0, shell1, tol)?;
	for shell in iter0 {
		let [res, _] = process_one_pair_of_shells(&and_shell, shell, tol)?;
		and_shell = res;
	}
	for shell in iter1 {
		let [res, _] = process_one_pair_of_shells(&and_shell, shell, tol)?;
		and_shell = res;
	}
	let boundaries = and_shell.connected_components();
	Some(Solid::new(boundaries))
}

/// OR operation between two solids.
pub fn or<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
	solid0: &Solid<Point3, C, S>,
	solid1: &Solid<Point3, C, S>,
	tol: f64,
) -> Option<Solid<Point3, C, S>> {
	let mut iter0 = solid0.boundaries().iter();
	let mut iter1 = solid1.boundaries().iter();
	let shell0 = iter0.next().unwrap();
	let shell1 = iter1.next().unwrap();
	let [_, mut or_shell] = process_one_pair_of_shells(shell0, shell1, tol)?;
	for shell in iter0 {
		let [_, res] = process_one_pair_of_shells(&or_shell, shell, tol)?;
		or_shell = res;
	}
	for shell in iter1 {
		let [_, res] = process_one_pair_of_shells(&or_shell, shell, tol)?;
		or_shell = res;
	}
	let boundaries = or_shell.connected_components();
	Some(Solid::new(boundaries))
}

fn random_normal_3d() -> Vector3 {
	let mut x = rand::random::<f64>();
	if x.so_small() {
		x = rand::random::<f64>();
	}
	let mut y = rand::random::<f64>();
	if y.so_small() {
		y = rand::random::<f64>();
	}
	let mut z = rand::random::<f64>();
	if z.so_small() {
		z = rand::random::<f64>();
	}
	let mut w = rand::random::<f64>();
	if w.so_small() {
		w = rand::random::<f64>();
	}
	Vector3::new(
		f64::sqrt(-2.0 * f64::ln(x)) * f64::cos(2.0 * PI * y),
		f64::sqrt(-2.0 * f64::ln(x)) * f64::sin(2.0 * PI * y),
		f64::sqrt(-2.0 * f64::ln(z)) * f64::cos(2.0 * PI * w),
	)
}

#[cfg(test)]
mod tests;
