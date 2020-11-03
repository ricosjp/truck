use crate::*;
type BSplineSurface = geometry::BSplineSurface<Vector4>;

impl StructuredMesh {
    /// meshing the bspline surface
    /// # Arguments
    /// * `bspsurface` - bspline surface to meshed
    /// * `tol` - standard tolerance for meshing
    pub fn from_surface(bspsurface: &BSplineSurface, tol: f64) -> StructuredMesh {
        let (div0, div1) = bspsurface.rational_parameter_division(tol);
        create_mesh(bspsurface, div0, div1)
    }
}

fn create_mesh(bspsurface: &BSplineSurface, div0: Vec<f64>, div1: Vec<f64>) -> StructuredMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for u in &div0 {
        let prow = div1
            .iter()
            .map(|v| Point3::from_homogeneous(bspsurface.subs(*u, *v)))
            .collect();
        let nrow = bspsurface.rational_normal_vectors(div1.iter().map(|v| (*u, *v)));
        positions.push(prow);
        normals.push(nrow);
    }
    StructuredMesh {
        positions: positions,
        uv_division: (div0, div1),
        normals: normals,
    }
}
