use crate::PolygonMesh;
use geometry::BSplineSurface;
use shape::Geometry;

impl PolygonMesh {
    /// meshing the bspline surface
    /// # Arguments
    /// * `bspsurface` - bspline surface to meshed
    /// * `tol` - standard tolerance for meshing
    pub fn from_surface(bspsurface: &mut BSplineSurface, tol: f64) -> PolygonMesh {
        let (knot_vec0, knot_vec1) = bspsurface.knot_vecs();
        let u0 = knot_vec0[0];
        let u1 = knot_vec0[knot_vec0.len() - 1];
        let mut div0 = vec![u0, u1];
        let v0 = knot_vec1[0];
        let v1 = knot_vec1[knot_vec1.len() - 1];
        let mut div1 = vec![v0, v1];

        create_space_division(bspsurface, tol, &mut div0, &mut div1);
        create_mesh(bspsurface, &div0, &div1)
    }

    pub fn from_shape(geometry: &mut Geometry, tol: f64) -> PolygonMesh {
        let mut mesh = PolygonMesh::default();
        for surface in geometry.surfaces_mut() {
            let counter = mesh.vertices.len();
            let mut tmp = PolygonMesh::from_surface(surface, tol);
            mesh.vertices.append(&mut tmp.vertices);
            mesh.uv_coords.append(&mut tmp.uv_coords);
            mesh.normals.append(&mut tmp.normals);
            for face in tmp.quad_faces.iter_mut() {
                for vert in face.iter_mut() {
                    vert[0] += counter;
                    vert[2] += counter;
                }
                mesh.quad_faces.push(face.clone());
            }
        }
        mesh
    }
}

fn is_far(bspsurface: &BSplineSurface, u0: f64, u1: f64, v0: f64, v1: f64, tol: f64) -> bool {
    let (mut degree0, mut degree1) = bspsurface.degrees();
    let bspsurface = bspsurface.get_closure();
    degree0 *= 2;
    degree1 *= 2;
    for i in 0..=degree0 {
        for j in 0..=degree1 {
            let p = (i as f64) / (degree0 as f64);
            let q = (j as f64) / (degree1 as f64);
            let u = u0 * p + u1 * (1.0 - p);
            let v = v0 * q + v1 * (1.0 - q);
            let val_mid = bspsurface(u, v);
            let par_mid = bspsurface(u0, v0) * p * q 
                + bspsurface(u0, v1) * p * (1.0 - q) 
                + bspsurface(u1, v0) * (1.0 - p) * q
                + bspsurface(u1, v1) * (1.0 - p) * (1.0 - q);
            let res = val_mid.projection() - par_mid.projection();
            if res.norm2() > tol * tol {
                return true;
            }
        }
    }
    false
}

fn create_space_division(
    bspsurface: &BSplineSurface,
    tol: f64,
    mut div0: &mut Vec<f64>,
    mut div1: &mut Vec<f64>,
) {
    let (degree0, degree1) = bspsurface.degrees();

    let mut divide_flag0 = vec![false; div0.len() - 1];
    let mut divide_flag1 = vec![false; div1.len() - 1];

    for i in 1..div0.len() {
        for j in 1..div1.len() {
            let far = is_far(bspsurface, div0[i - 1], div0[i], div1[j - 1], div1[j], tol);
            divide_flag0[i - 1] = divide_flag0[i - 1] || far;
            divide_flag1[j - 1] = divide_flag1[j - 1] || far;
        }
    }

    let mut new_div0 = vec![div0[0]];
    for i in 1..div0.len() {
        if divide_flag0[i - 1] {
            for j in 1..=degree0 {
                let p = (j as f64) / (degree0 as f64);
                new_div0.push(div0[i - 1] * (1.0 - p) + div0[i] * p);
            }
        } else {
            new_div0.push(div0[i]);
        }
    }

    let mut new_div1 = vec![div1[0]];
    for i in 1..div1.len() {
        if divide_flag1[i - 1] {
            for j in 1..=degree1 {
                let p = (j as f64) / (degree1 as f64);
                new_div1.push(div1[i - 1] * (1.0 - p) + div1[i] * p);
            }
        } else {
            new_div1.push(div1[i]);
        }
    }

    if div0.len() != new_div0.len() || div1.len() != new_div1.len() {
        *div0 = new_div0;
        *div1 = new_div1;
        create_space_division(bspsurface, tol, &mut div0, &mut div1);
    }
}

fn create_mesh(bspsurface: &mut BSplineSurface, div0: &Vec<f64>, div1: &Vec<f64>) -> PolygonMesh {
    let mut meshdata = PolygonMesh::default();
    for u in div0 {
        for v in div1 {
            let vertex = bspsurface.subs(*u, *v).projection();
            meshdata.vertices.push([vertex[0], vertex[1], vertex[2]]);
            meshdata.uv_coords.push([*u, *v]);
            let normal = bspsurface.normal_vector(*u, *v).projection();
            meshdata.normals.push([normal[0], normal[1], normal[2]]);
        }
    }
    for i in 1..div0.len() {
        for j in 1..div1.len() {
            let i0 = div1.len() * (i - 1) + (j - 1);
            let i1 = div1.len() * i + (j - 1);
            let i2 = div1.len() * i + j;
            let i3 = div1.len() * (i - 1) + j;
            meshdata.quad_faces.push([[i0, i0, i0], [i1, i1, i1], [i2, i2, i2], [i3, i3, i3]]);
        }
    }
    meshdata
}
