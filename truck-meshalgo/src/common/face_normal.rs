use super::*;

#[derive(Clone, Copy, Debug)]
pub struct FaceNormal {
    pub face_id: usize,
    pub normal: Vector3,
}

impl FaceNormal {
    pub fn new(positions: &[Point3], face: &[Vertex], face_id: usize) -> FaceNormal {
        let center = face
            .iter()
            .fold(Vector3::zero(), |sum, v| sum + positions[v.pos].to_vec())
            / face.len() as f64;
        let normal = face
            .windows(2)
            .chain(std::iter::once([face[face.len() - 1], face[0]].as_ref()))
            .fold(Vector3::zero(), |sum, v| {
                let vec0 = positions[v[0].pos].to_vec() - center;
                let vec1 = positions[v[1].pos].to_vec() - center;
                sum + vec0.cross(vec1)
            })
            .normalize();
        FaceNormal { face_id, normal }
    }
}

