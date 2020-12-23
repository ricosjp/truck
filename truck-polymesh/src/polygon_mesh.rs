use crate::*;

impl PolygonMesh {
    /// Creates the bounding box of the polygon mesh.
    #[inline(always)]
    pub fn bounding_box(&self) -> BoundingBox<Point3> { self.positions.iter().collect() }

    /// merge `mesh` to `self`.
    pub fn merge(&mut self, mesh: PolygonMesh) {
        let n_pos = self.positions.len();
        let n_uv = self.uv_coords.len();
        let n_nor = self.normals.len();
        let PolygonMesh {
            positions,
            uv_coords,
            normals,
            tri_faces,
            quad_faces,
            other_faces,
        } = mesh;
        self.positions.extend(positions);
        self.uv_coords.extend(uv_coords);
        self.normals.extend(normals);
        for face in tri_faces {
            self.tri_faces.push([
                [face[0][0] + n_pos, face[0][1] + n_uv, face[0][2] + n_nor],
                [face[1][0] + n_pos, face[1][1] + n_uv, face[1][2] + n_nor],
                [face[2][0] + n_pos, face[2][1] + n_uv, face[2][2] + n_nor],
            ]);
        }
        for face in quad_faces {
            self.quad_faces.push([
                [face[0][0] + n_pos, face[0][1] + n_uv, face[0][2] + n_nor],
                [face[1][0] + n_pos, face[1][1] + n_uv, face[1][2] + n_nor],
                [face[2][0] + n_pos, face[2][1] + n_uv, face[2][2] + n_nor],
                [face[3][0] + n_pos, face[3][1] + n_uv, face[3][2] + n_nor],
            ]);
        }
        for face in other_faces {
            let new_face = face
                .into_iter()
                .map(move |v| [v[0] + n_pos, v[1] + n_uv, v[2] + n_nor])
                .collect();
            self.other_faces.push(new_face);
        }
    }
}
