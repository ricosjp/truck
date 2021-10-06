use crate::*;

/// Shape before wrapped wasm
pub trait IntoWasm: Sized {
    /// wasm wrapper struct
    type WasmWrapper: From<Self>;
    /// Into wasm wrapper
    fn into_wasm(self) -> Self::WasmWrapper { self.into() }
}

macro_rules! toporedef {
    ($type: ident, $member: ident) => {
        /// wasm shape wrapper
        #[wasm_bindgen]
        #[derive(Clone, Debug, From, Into, Deref, DerefMut, AsRef)]
        pub struct $type(truck_modeling::$type);

        impl IntoWasm for truck_modeling::$type {
            type WasmWrapper = $type;
        }
        #[wasm_bindgen]
        impl $type {
            /// upcast to abstract shape
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

/// abstract shape, effectively an enumerated type
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
    /// whether `self` is vertex or not.
    #[inline(always)]
    pub fn is_vertex(self) -> bool { self.vertex.is_some() }
    /// whether `self` is edge or not.
    #[inline(always)]
    pub fn is_edge(self) -> bool { self.edge.is_some() }
    /// whether `self` is wire or not.
    #[inline(always)]
    pub fn is_wire(self) -> bool { self.wire.is_some() }
    /// whether `self` is face or not.
    #[inline(always)]
    pub fn is_face(self) -> bool { self.face.is_some() }
    /// whether `self` is shell or not.
    #[inline(always)]
    pub fn is_shell(self) -> bool { self.shell.is_some() }
    /// whether `self` is solid or not.
    #[inline(always)]
    pub fn is_solid(self) -> bool { self.solid.is_some() }
    /// downcast
    #[inline(always)]
    pub fn into_vertex(self) -> Option<Vertex> { self.vertex }
    /// downcast
    #[inline(always)]
    pub fn into_edge(self) -> Option<Edge> { self.edge }
    /// downcast
    #[inline(always)]
    pub fn into_wire(self) -> Option<Wire> { self.wire }
    /// downcast
    #[inline(always)]
    pub fn into_face(self) -> Option<Face> { self.face }
    /// downcast
    #[inline(always)]
    pub fn into_shell(self) -> Option<Shell> { self.shell }
    /// downcast
    #[inline(always)]
    pub fn into_solid(self) -> Option<Solid> { self.solid }
}

impl AbstractShape {
    /// downcast as reference
    #[inline(always)]
    pub fn as_vertex(&self) -> Option<&Vertex> { self.vertex.as_ref() }
    /// downcast as reference
    #[inline(always)]
    pub fn as_edge(&self) -> Option<&Edge> { self.edge.as_ref() }
    /// downcast as reference
    #[inline(always)]
    pub fn as_wire(&self) -> Option<&Wire> { self.wire.as_ref() }
    /// downcast as reference
    #[inline(always)]
    pub fn as_face(&self) -> Option<&Face> { self.face.as_ref() }
    /// downcast as reference
    #[inline(always)]
    pub fn as_shell(&self) -> Option<&Shell> { self.shell.as_ref() }
    /// downcast as reference
    #[inline(always)]
    pub fn as_solid(&self) -> Option<&Solid> { self.solid.as_ref() }
}
