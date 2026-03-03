use crate::*;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

/// Minimum U-row count to enable parallel surface evaluation.
#[cfg(not(target_arch = "wasm32"))]
const PAR_THRESHOLD: usize = 8;

impl<P> PolylineCurve<P> {
    /// Meshes the curve.
    pub fn from_curve<C>(curve: C, range: (f64, f64), tol: f64) -> Self
    where C: ParameterDivision1D<Point = P> {
        PolylineCurve(curve.parameter_division(range, tol).1)
    }
}

fn eval_row(
    surface: &impl ParametricSurface3D,
    u: f64,
    div1: &[f64],
) -> (Vec<Point3>, Vec<Vector3>) {
    let positions = div1.iter().map(|v| surface.subs(u, *v)).collect();
    let normals = div1.iter().map(|v| surface.normal(u, *v)).collect();
    (positions, normals)
}

impl StructuredMesh {
    /// Meshes the surface.
    /// # Arguments
    /// * `surface` - surface to be meshed.
    /// * `range` - parameter range.
    /// * `tol` - standard tolerance for meshing.
    pub fn from_surface<S>(
        surface: &S,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> StructuredMesh
    where
        S: ParametricSurface3D + ParameterDivision2D,
    {
        let (div0, div1) = surface.parameter_division(range, tol);
        let (positions, normals): (Vec<_>, Vec<_>) =
            div0.iter().map(|u| eval_row(surface, *u, &div1)).unzip();
        StructuredMesh {
            positions,
            uv_division: Some((div0, div1)),
            normals: Some(normals),
        }
    }

    /// Meshes the surface with parallel U-row evaluation on non-WASM.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_surface_par<S>(
        surface: &S,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> StructuredMesh
    where
        S: ParametricSurface3D + ParameterDivision2D + Sync,
    {
        let (div0, div1) = surface.parameter_division(range, tol);
        let (positions, normals): (Vec<_>, Vec<_>) = if div0.len() >= PAR_THRESHOLD {
            div0.par_iter()
                .map(|u| eval_row(surface, *u, &div1))
                .unzip()
        } else {
            div0.iter().map(|u| eval_row(surface, *u, &div1)).unzip()
        };
        StructuredMesh {
            positions,
            uv_division: Some((div0, div1)),
            normals: Some(normals),
        }
    }
}
