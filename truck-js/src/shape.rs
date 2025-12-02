use crate::*;
use js_sys::JsString;
use truck_meshalgo::tessellation::*;
use truck_modeling::algo::DefaultSplitParams;

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
                pub fn $is(&self) -> bool {
                    match &self.0 {
                        SubAbstractShape::$type(_) => true,
                        _ => false,
                    }
                }

                /// downcast
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

/// Describe STEP file header
#[derive(Clone, Debug, AsRef, Deref, DerefMut, From, Into)]
#[wasm_bindgen]
pub struct StepHeaderDescriptor(truck_stepio::out::StepHeaderDescriptor);

#[wasm_bindgen]
impl StepHeaderDescriptor {
    #[wasm_bindgen(getter)]
    pub fn filename(&self) -> JsString { self.file_name.as_str().into() }
    #[wasm_bindgen(setter)]
    pub fn set_filename(&mut self, filename: JsString) {
        self.file_name = filename.as_string().unwrap_or_default();
    }
    #[wasm_bindgen(getter)]
    pub fn time_stamp(&self) -> JsString { self.time_stamp.as_str().into() }
    #[wasm_bindgen(setter)]
    pub fn set_time_stamp(&mut self, time_stamp: JsString) {
        self.time_stamp = time_stamp.as_string().unwrap_or_default();
    }
    #[wasm_bindgen(getter)]
    pub fn authors(&self) -> Vec<JsString> {
        self.authors.iter().map(|s| s.as_str().into()).collect()
    }
    #[wasm_bindgen(setter)]
    pub fn set_authors(&mut self, authors: Vec<JsString>) {
        self.authors = authors
            .iter()
            .map(|s| s.as_string().unwrap_or_default())
            .collect();
    }
    #[wasm_bindgen(getter)]
    pub fn organization(&self) -> Vec<JsString> {
        self.organization
            .iter()
            .map(|s| s.as_str().into())
            .collect()
    }
    #[wasm_bindgen(setter)]
    pub fn set_organization(&mut self, organization: Vec<JsString>) {
        self.organization = organization
            .iter()
            .map(|s| s.as_string().unwrap_or_default())
            .collect();
    }
    #[wasm_bindgen(getter)]
    pub fn organization_system(&self) -> JsString { self.organization_system.as_str().into() }
    #[wasm_bindgen(setter)]
    pub fn set_organization_system(&mut self, organization_system: JsString) {
        self.organization_system = organization_system.as_string().unwrap_or_default();
    }
    #[wasm_bindgen(getter)]
    pub fn authorization(&self) -> JsString { self.authorization.as_str().into() }
    #[wasm_bindgen(setter)]
    pub fn set_authorization(&mut self, authorization: JsString) {
        self.authorization = authorization.as_string().unwrap_or_default();
    }
}

macro_rules! impl_shape {
    ($type: ident) => {
        #[wasm_bindgen]
        impl $type {
            /// meshing shape
            pub fn to_polygon(&self, tol: f64) -> PolygonMesh {
                self.triangulation(DefaultSplitParams::new(tol)).to_polygon().into_wasm()
            }
            /// read shape from json
            pub fn from_json(data: &[u8]) -> Option<$type> {
                serde_json::from_reader::<_, truck_modeling::$type>(data)
                .map_err(|e| println!("{e}"))
                .ok()
                .map(|res| res.into_wasm())
            }
            /// write shape to json
            pub fn to_json(&self) -> Vec<u8> {
                serde_json::to_vec_pretty(&self.0)
                    .map_err(|e| gloo::console::error!(format!("{e}")))
                    .unwrap()
            }
            /// write shape to STEP
            pub fn to_step(&self, header: StepHeaderDescriptor) -> String {
                use truck_stepio::out;
                let compressed = self.0.compress();
                out::CompleteStepDisplay::new(
                    out::StepModel::from(&compressed),
                    header.into(),
                ).to_string()
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
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()
            .map(IntoWasm::into_wasm)
    }
}
