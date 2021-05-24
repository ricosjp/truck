use super::*;
use std::collections::{HashMap, HashSet};
use truck_topology::shell::ShellCondition;

pub trait AsShell {
    //    fn extract_boundaries(&self) -> Vec<Vec<Vertex>>;
    fn shell_condition(&self) -> ShellCondition;
}

#[derive(Clone, Debug)]
struct Boundaries {
    checked: HashSet<[usize; 2]>,
    boundary: HashMap<[usize; 2], bool>,
    condition: ShellCondition,
}

impl Boundaries {
    #[inline(always)]
    fn new() -> Self {
        Boundaries {
            checked: Default::default(),
            boundary: Default::default(),
            condition: ShellCondition::Oriented,
        }
    }
    #[inline(always)]
    fn insert(&mut self, edge: [Vertex; 2]) {
        let ori = edge[0].pos < edge[1].pos;
        let edge = match ori {
            true => [edge[0].pos, edge[1].pos],
            false => [edge[1].pos, edge[0].pos],
        };
        self.condition = self.condition & match (self.checked.insert(edge), self.boundary.insert(edge, ori)) {
            (true, None) => ShellCondition::Oriented,
            (false, None) => ShellCondition::Irregular,
            (true, Some(_)) => panic!("unexpected case!"),
            (false, Some(ori0)) => match ori == ori0 {
                true => ShellCondition::Regular,
                false => ShellCondition::Oriented,
            },
        };
    }

    #[inline(always)]
    fn condition(&self) -> ShellCondition {
        if self.condition == ShellCondition::Oriented && self.boundary.is_empty() {
            ShellCondition::Closed
        } else {
            self.condition
        }
    }
}

impl std::iter::FromIterator<[Vertex; 2]> for Boundaries {
    fn from_iter<I: IntoIterator<Item = [Vertex; 2]>>(iter: I) -> Boundaries {
        let mut boundaries = Boundaries::new();
        iter.into_iter().for_each(|edge| boundaries.insert(edge));
        boundaries
    }
}

impl AsShell for PolygonMesh {
    fn shell_condition(&self) -> ShellCondition {
        let boundaries: Boundaries = self.faces().face_iter().flat_map(move |face| {
            let len = face.len();
            (0..len).map(move |i| [face[i], face[(i + 1) % len]])
        }).collect();
        boundaries.condition()
    }
}
