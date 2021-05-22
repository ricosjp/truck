use crate::*;

impl StructuredMesh {
    /// meshing the bspline surface
    /// # Arguments
    /// * `bspsurface` - bspline surface to meshed
    /// * `tol` - standard tolerance for meshing
    pub fn from_surface<S>(surface: &S, range: ((f64, f64), (f64, f64)), tol: f64) -> StructuredMesh
    where S: ParametricSurface3D + ParameterDivision2D {
        let (div0, div1) = surface.parameter_division(range, tol);
        create_mesh(surface, div0, div1)
    }
}

fn create_mesh<S>(surface: &S, div0: Vec<f64>, div1: Vec<f64>) -> StructuredMesh
where S: ParametricSurface3D {
    let mut positions = vec![Vec::with_capacity(div1.len()); div0.len()];
    let mut normals = vec![Vec::with_capacity(div1.len()); div0.len()];
    div0.iter()
        .zip(positions.iter_mut().zip(normals.iter_mut()))
        .for_each(|(u, (prow, nrow))| {
            div1.iter().for_each(move|v| {
                prow.push(surface.subs(*u, *v));
                nrow.push(surface.normal(*u, *v));
            })
        });
    StructuredMesh {
        positions: positions,
        uv_division: Some((div0, div1)),
        normals: Some(normals),
    }
}
