use crate::*;
use truck_meshalgo::tessellation::*;

macro_rules! toporedef {
    ($($type: ident, $is: ident, $into: ident, $to: ident),*) => {
        $(
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
                AbstractShape(SubAbstractShape::$type(self))
            }
        }
        )*

        /// wasm wrapped methods
        #[wasm_bindgen]
        impl AbstractShape {
            $(
                /// check the type
                #[inline(always)]
                pub fn $is(&self) -> bool {
                    match &self.0 {
                        SubAbstractShape::$type(_) => true,
                        _ => false,
                    }
                }

                /// downcast
                #[inline(always)]
                pub fn $into(self) -> Option<$type> {
                    match self.0 {
                        SubAbstractShape::$type(got) => Some(got),
                        _ => None,
                    }
                }
            )*
        }

        /// only for rust
        impl AbstractShape {
            $(
                /// reference downcast
                #[inline(always)]
                pub fn $to(&self) -> Option<&$type> {
                    match &self.0 {
                        SubAbstractShape::$type(got) => Some(got),
                        _ => None,
                    }
                }
            )*
        }
    }
}

#[rustfmt::skip]
toporedef!(
    Vertex, is_vertex, into_vertex, as_vertex,
    Edge, is_edge, into_edge, as_edge,
    Wire, is_wire, into_wire, as_wire,
    Face, is_face, into_face, as_face,
    Shell, is_shell, into_shell, as_shell,
    Solid, is_solid, into_solid, as_solid
);

/// abstract shape, effectively an enumerated type
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct AbstractShape(SubAbstractShape);

#[derive(Clone, Debug)]
enum SubAbstractShape {
    Vertex(Vertex),
    Edge(Edge),
    Wire(Wire),
    Face(Face),
    Shell(Shell),
    Solid(Solid),
}

macro_rules! impl_shape {
    ($type: ident) => {
        #[wasm_bindgen]
        impl $type {
            /// meshing shape
            pub fn to_polygon(&self, tol: f64) -> Option<PolygonMesh> {
                Some(self.triangulation(tol)?.to_polygon().into_wasm())
            }
            /// read shape from json
            pub fn from_json(data: &[u8]) -> Option<$type> {
                truck_modeling::$type::extract(
                    serde_json::from_reader(data)
                        .map_err(|e| eprintln!("{}", e))
                        .ok()?,
                )
                .map_err(|e| eprintln!("{}", e))
                .ok()
                .map(|res| res.into_wasm())
            }
            /// write shape from json
            pub fn to_json(&self) -> Vec<u8> {
                serde_json::to_vec_pretty(&self.0.compress())
                    .map_err(|e| eprintln!("{}", e))
                    .unwrap()
            }
        }
    };
    ($a: ident, $($b: ident),*) => { impl_shape!($a); impl_shape!($($b),*); }
}

impl_shape!(Shell, Solid);

#[wasm_bindgen]
impl Shell {
    /// Creates Solid if `self` is a closed shell.
    pub fn into_solid(self) -> Option<Solid> {
        truck_modeling::Solid::try_new(vec![self.0])
            .map_err(|e| eprintln!("{}", e))
            .ok()
            .map(IntoWasm::into_wasm)
    }
}
