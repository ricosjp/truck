use super::*;

pub struct TriangleIter<'a, I> {
    iter: I,
    current: Option<&'a [Vertex]>,
    loc_idx: usize,
}

impl<'a, I> Iterator for TriangleIter<'a, I>
where I: Iterator<Item = &'a [Vertex]>
{
    type Item = [Vertex; 3];
    fn next(&mut self) -> Option<[Vertex; 3]> {
        if self.current.is_none() {
            self.current = self.iter.next();
            self.loc_idx = 2;
        } else if self.loc_idx == self.current.unwrap().len() {
            self.current = self.iter.next();
            self.loc_idx = 2;
        }
        let res = self
            .current
            .map(|face| [face[0], face[self.loc_idx - 1], face[self.loc_idx]]);
        self.loc_idx += 1;
        res
    }
}

pub struct Triangulate<'a>(pub &'a PolygonMesh);

impl<'a> Triangulate<'a> {
    #[inline(always)]
    pub fn into_iter(&self) -> TriangleIter<'a, impl Iterator<Item = &'a [Vertex]>> {
        TriangleIter {
            iter: self.0.face_iter(),
            current: None,
            loc_idx: 0,
        }
    }
}
