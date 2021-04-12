use crate::*;

impl StructuredMesh {
    /// meshing the bspline surface
    /// # Arguments
    /// * `bspsurface` - bspline surface to meshed
    /// * `tol` - standard tolerance for meshing
    pub fn from_surface<S>(bspsurface: &S, tol: f64) -> StructuredMesh
    where S: ParametricSurface<Point = Point3, Vector = Vector3> + ParameterDivision2D {
        let (div0, div1) = bspsurface.parameter_division(tol);
        create_mesh(bspsurface, div0, div1)
    }
}

fn create_mesh<S>(bspsurface: &S, div0: Vec<f64>, div1: Vec<f64>) -> StructuredMesh
where S: ParametricSurface<Point = Point3, Vector = Vector3> {
    let mut positions = vec![Vec::with_capacity(div1.len()); div0.len()];
    let mut normals = vec![Vec::with_capacity(div1.len()); div0.len()];
    div0.iter()
        .zip(positions.iter_mut().zip(normals.iter_mut()))
        .for_each(|(u, (prow, nrow))| {
            div1.iter().for_each(move|v| {
                prow.push(bspsurface.subs(*u, *v));
                nrow.push(bspsurface.normal(*u, *v));
            })
        });
    StructuredMesh {
        positions: positions,
        uv_division: Some((div0, div1)),
        normals: Some(normals),
    }
}
