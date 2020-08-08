use crate::*;

impl Light {
    pub fn buffer(&self, device: &Device) -> BufferHandler {
        let light_info = LightInfo {
            light_position: (&self.position).cast().unwrap().into(),
            light_strength: self.strength as f32,
            light_color: self.color.cast().unwrap().into(),
            light_type: self.light_type.type_id(),
        };
        let buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[light_info]),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        );
        BufferHandler::new(buffer, std::mem::size_of::<LightInfo>() as u64)
    }
}

impl Default for Light {
    #[inline(always)]
    fn default() -> Light {
        Light {
            position: Point3::origin(),
            strength: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
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
