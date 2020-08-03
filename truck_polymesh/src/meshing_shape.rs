use crate::*;
type BSplineSurface = geometry::BSplineSurface<[f64; 4]>;

impl StructuredMesh {
    /// meshing the bspline surface
    /// # Arguments
    /// * `bspsurface` - bspline surface to meshed
    /// * `tol` - standard tolerance for meshing
    pub fn from_surface(bspsurface: &BSplineSurface, tol: f64) -> StructuredMesh {
        let (knot_vec0, knot_vec1) = bspsurface.knot_vecs();
        let u0 = knot_vec0[0];
        let u1 = knot_vec0[knot_vec0.len() - 1];
        let mut div0 = vec![u0, u1];
        let v0 = knot_vec1[0];
        let v1 = knot_vec1[knot_vec1.len() - 1];
        let mut div1 = vec![v0, v1];

        create_space_division(bspsurface, tol, &mut div0, &mut div1);
        create_mesh(bspsurface, div0, div1)
    }
}

fn is_far(bspsurface: &BSplineSurface, u0: f64, u1: f64, v0: f64, v1: f64, tol: f64) -> bool {
    let (mut degree0, mut degree1) = bspsurface.degrees();
    let bspsurface = bspsurface.get_closure();
    degree0 *= 2;
    degree1 *= 2;
    let pt00 = bspsurface(u0, v0);
    let pt01 = bspsurface(u0, v1);
    let pt10 = bspsurface(u1, v0);
    let pt11 = bspsurface(u1, v1);
    for i in 0..=degree0 {
        for j in 0..=degree1 {
            let p = (i as f64) / (degree0 as f64);
            let q = (j as f64) / (degree1 as f64);
            let u = u0 * p + u1 * (1.0 - p);
            let v = v0 * q + v1 * (1.0 - q);
            let val_mid = bspsurface(u, v);
            let par_mid = &pt00 * p * q
                + &pt01 * p * (1.0 - q)
                + &pt10 * (1.0 - p) * q
                + &pt11 * (1.0 - p) * (1.0 - q);
            let res = val_mid.rational_projection() - par_mid.rational_projection();
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
)
{
    let (mut degree0, mut degree1) = bspsurface.degrees();
    degree0 *= 2;
    degree1 *= 2;

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

fn create_mesh(bspsurface: &BSplineSurface, div0: Vec<f64>, div1: Vec<f64>) -> StructuredMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for u in &div0 {
        let prow = div1
            .iter()
            .map(|v| {
                let pt = bspsurface.subs(*u, *v).rational_projection();
                vector!(pt[0], pt[1], pt[2])
            })
            .collect();
        let nrow = bspsurface
            .rational_normal_vectors(div1.iter().map(|v| (*u, *v)));
        positions.push(prow);
        normals.push(nrow);
    }
    StructuredMesh {
        positions: positions,
        uv_division: (div0, div1),
        normals: normals,
    }
}
