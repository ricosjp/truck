use super::*;

/// STL face generate from `PolygonMesh`
#[derive(Debug)]
pub struct TriangleIter<'a> {
    tri_faces: std::slice::Iter<'a, [Vertex; 3]>,
    quad_faces: std::slice::Iter<'a, [Vertex; 4]>,
    other_faces: std::slice::Iter<'a, [Vertex; 3]>,
    current_face: Option<&'a [Vertex]>,
    current_vertex: usize,
    len: usize,
}

impl<'a> Iterator for TriangleIter<'a> {
    type Item = [Vertex; 3];
    fn next(&mut self) -> Option<[Vertex; 3]> {
        if self.current_face.is_none() {
            self.current_face = if let Some(face) = self.tri_faces.next() {
                Some(face)
            } else if let Some(face) = self.quad_faces.next() {
                Some(face)
            } else if let Some(face) = self.other_faces.next() {
                Some(face)
            } else {
                return None;
            }
        }
        let face = self.current_face.unwrap();
        let res = [
            face[0],
            face[self.current_vertex + 1],
            face[self.current_vertex + 2],
        ];
        if self.current_vertex + 3 == face.len() {
            self.current_face = None;
            self.current_vertex = 0;
        } else {
            self.current_vertex += 1;
        }
        self.len -= 1;
        Some(res)
    }
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }
}

impl<'a> ExactSizeIterator for TriangleIter<'a> {}

#[derive(Clone, Debug)]
pub struct Triangulate<'a> {
    entity: &'a PolygonMesh,
    other_faces: Vec<[Vertex; 3]>,
}

impl<'a> Triangulate<'a> {
    #[inline(always)]
    pub fn new(entity: &'a PolygonMesh) -> Self {
        Triangulate {
            other_faces: entity
                .faces()
                .other_faces()
                .iter()
                .flat_map(move |face| (2..face.len()).map(move |i| [face[0], face[i - 1], face[i]]))
                .collect(),
            entity,
        }
    }
    #[inline(always)]
    pub fn get(&self, idx: usize) -> [Vertex; 3] {
        if idx < self.entity.tri_faces().len() {
            self.entity.tri_faces()[idx]
        } else if idx < self.entity.tri_faces().len() + 2 * self.entity.quad_faces().len() {
            let idx = idx - self.entity.tri_faces().len();
            let face = self.entity.quad_faces()[idx / 2];
            if idx % 2 == 0 {
                [face[0], face[1], face[2]]
            } else {
                [face[0], face[2], face[3]]
            }
        } else {
            self.other_faces
                [idx - self.entity.tri_faces().len() - 2 * self.entity.quad_faces().len()]
        }
    }
    #[inline(always)]
    pub fn entity(&self) -> &PolygonMesh { &self.entity }
}

impl<'a> IntoIterator for &'a Triangulate<'a> {
    type Item = [Vertex; 3];
    type IntoIter = TriangleIter<'a>;
    #[inline(always)]
    fn into_iter(self) -> TriangleIter<'a> {
        let len = self
            .entity
            .face_iter()
            .fold(0, |len, face| len + face.len());
        TriangleIter {
            tri_faces: self.entity.tri_faces().iter(),
            quad_faces: self.entity.quad_faces().iter(),
            other_faces: self.other_faces.iter(),
            current_face: None,
            current_vertex: 0,
            len,
        }
    }
}
