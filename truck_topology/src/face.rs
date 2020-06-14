use crate::errors::Error;
use crate::id::IDGenerator;
use crate::{Result, Wire, Face};
use std::collections::HashMap;

lazy_static! {
    static ref ID_GENERATOR: IDGenerator  = IDGenerator::new();
}

impl Face {
    /// construct new face by a wire.
    /// # Panic
    /// `boundary` must be simple and closed.
    #[inline(always)]
    pub fn new(boundary: Wire) -> Face {
        match Face::try_new(boundary) {
            Ok(got) => got,
            Err(error) => panic!("{}", error),
        }
    }
    
    /// construct new face by a wire.
    /// # Failure
    /// `boundary` must be simple and closed.
    #[inline(always)]
    pub fn try_new(boundary: Wire) -> Result<Face> {
        if !boundary.is_closed() {
            Err(Error::NotClosedWire)
        } else if !boundary.is_simple() {
            Err(Error::NotSimpleWire)
        } else {
            Ok(Face::new_unchecked(boundary))
        }
    }

    /// construct new face by a wire.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether `boundary` is simple and closed.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn new_unchecked(boundary: Wire) -> Face {
        Face {
            boundary: boundary,
            id: ID_GENERATOR.generate(),
        }
    }

    /// get the reference of the boundary wire.
    #[inline(always)]
    pub fn boundary(&self) -> &Wire { &self.boundary }

    #[inline(always)]
    pub fn into_boundary(self) -> Wire { self.boundary }

    /// get the face id.
    #[inline(always)]
    pub fn id(&self) -> usize { self.id }

    /// inverse the direction of face and give a new id.
    #[inline(always)]
    pub fn inverse(&mut self) -> &mut Self {
        self.boundary.inverse();
        self.id = ID_GENERATOR.generate();
        self
    }

    /// return true, if the two faces have the shared edge.
    pub fn border_on(&self, other: &Face) -> bool {
        let mut hashmap = HashMap::new();
        for edge in self.boundary.edge_iter() {
            hashmap.insert(edge.id(), edge);
        }
        for edge in other.boundary.edge_iter() {
            if hashmap.insert(edge.id(), edge).is_some() {
                return true;
            }
        }
        false
    }
}
