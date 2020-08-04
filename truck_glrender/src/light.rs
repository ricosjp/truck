use crate::*;

impl Default for Light {
    #[inline(always)]
    fn default() -> Light {
        Light {
            position: Vector3::zero(),
            strength: 1.0,
            light_type: LightType::Point,
        }
    }
}

impl From<LightType> for usize {
    fn from(light_type: LightType) -> usize {
        match light_type {
            LightType::Point => 0,
            LightType::Uniform => 1,
        }
    }
}

impl LightType {
    pub fn type_id(&self) -> i32 {
        match *self {
            LightType::Point => 0,
            LightType::Uniform => 1,
        }
    }
}
