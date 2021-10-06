mod utils;

use derive_more::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub trait IntoWasm: Sized {
    type WasmStruct: From<Self>;
    fn into_wasm(self) -> Self::WasmStruct { self.into() }
}

macro_rules! toporedef {
    ($type: ident, $member: ident) => {
        /// wrapper from `truck-modeling`
        #[wasm_bindgen]
        #[derive(Clone, Debug, From, Into, Deref, DerefMut)]
        pub struct $type(truck_modeling::$type);

        impl IntoWasm for truck_modeling::$type {
            type WasmStruct = $type;
        }
        #[wasm_bindgen]
        impl $type {
            #[inline(always)]
            pub fn upcast(self) -> AbstractShape {
                let mut res = AbstractShape::empty();
                res.$member = Some(self);
                res
            }
        }
    };
    ($type: ident, $member: ident, $($a: ident, $b: ident),*) => {
        toporedef!($type, $member);
        toporedef!($($a, $b),*);
    }
}

toporedef!(Vertex, vertex, Edge, edge, Wire, wire, Face, face, Shell, shell, Solid, solid);

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct AbstractShape {
    vertex: Option<Vertex>,
    edge: Option<Edge>,
    wire: Option<Wire>,
    face: Option<Face>,
    shell: Option<Shell>,
    solid: Option<Solid>,
}

impl AbstractShape {
    fn empty() -> Self {
        Self {
            vertex: None,
            edge: None,
            wire: None,
            face: None,
            shell: None,
            solid: None,
        }
    }
}

#[wasm_bindgen]
impl AbstractShape {
    #[inline(always)]
    pub fn is_vertex(self) -> bool { self.vertex.is_some() }
    #[inline(always)]
    pub fn is_edge(self) -> bool { self.edge.is_some() }
    #[inline(always)]
    pub fn is_wire(self) -> bool { self.wire.is_some() }
    #[inline(always)]
    pub fn is_face(self) -> bool { self.face.is_some() }
    #[inline(always)]
    pub fn is_shell(self) -> bool { self.shell.is_some() }
    #[inline(always)]
    pub fn is_solid(self) -> bool { self.solid.is_some() }
    #[inline(always)]
    pub fn into_vertex(self) -> Vertex { self.vertex.unwrap() }
    #[inline(always)]
    pub fn into_edge(self) -> Edge { self.edge.unwrap() }
    #[inline(always)]
    pub fn into_wire(self) -> Wire { self.wire.unwrap() }
    #[inline(always)]
    pub fn into_face(self) -> Face { self.face.unwrap() }
    #[inline(always)]
    pub fn into_shell(self) -> Shell { self.shell.unwrap() }
    #[inline(always)]
    pub fn into_solid(self) -> Solid { self.solid.unwrap() }
}

pub mod builder;
