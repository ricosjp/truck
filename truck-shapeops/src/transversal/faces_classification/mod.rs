use super::loops_store::ShapesOpStatus;
use rustc_hash::FxHashMap as HashMap;
use truck_topology::*;

#[derive(Clone, Debug)]
pub struct FacesClassification<P, C, S> {
    shell: Shell<P, C, S>,
    status: HashMap<FaceID<S>, ShapesOpStatus>,
}

impl<P, C, S> Default for FacesClassification<P, C, S> {
    fn default() -> Self {
        Self {
            shell: Default::default(),
            status: HashMap::default(),
        }
    }
}

impl<P, C, S> FacesClassification<P, C, S> {
    pub fn push(&mut self, face: Face<P, C, S>, status: ShapesOpStatus) {
        self.status.insert(face.id(), status);
        self.shell.push(face);
    }

    pub fn and_or_unknown(&self) -> [Shell<P, C, S>; 3] {
        let [mut and, mut or, mut unknown] = <[Shell<P, C, S>; 3]>::default();
        for face in &self.shell {
            match self.status.get(&face.id()).unwrap() {
                ShapesOpStatus::And => and.push(face.clone()),
                ShapesOpStatus::Or => or.push(face.clone()),
                ShapesOpStatus::Unknown => unknown.push(face.clone()),
            }
        }
        [and, or, unknown]
    }

    pub fn integrate_by_component(&mut self) {
        let [and, or, unknown] = self.and_or_unknown();
        let and_boundary = and.extract_boundaries();
        let or_boundary = or.extract_boundaries();
        let components = unknown.connected_components();
        for comp in components {
            let boundary = comp.extract_boundaries();
            if and_boundary
                .iter()
                .flatten()
                .any(|edge| edge.id() == boundary[0][0].id())
            {
                comp.iter().for_each(|face| {
                    *self.status.get_mut(&face.id()).unwrap() = ShapesOpStatus::And;
                })
            } else if or_boundary
                .iter()
                .flatten()
                .any(|edge| edge.id() == boundary[0][0].id())
            {
                comp.iter().for_each(|face| {
                    *self.status.get_mut(&face.id()).unwrap() = ShapesOpStatus::Or;
                })
            }
        }
    }
}

#[cfg(test)]
mod tests;
