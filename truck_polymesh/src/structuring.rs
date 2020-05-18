use crate::MeshHandler;

impl MeshHandler {
    pub fn destructuring(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        for quad in &mesh.quad_faces {
            mesh.tri_faces.push([quad[0].clone(), quad[1].clone(), quad[3].clone()]);
            mesh.tri_faces.push([quad[2].clone(), quad[3].clone(), quad[1].clone()]);
        }
        mesh.quad_faces = Vec::new();

        self
    }
}
