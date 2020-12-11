use crate::*;

impl Light {
    pub(super) fn light_info(&self) -> LightInfo {
        LightInfo {
            light_position: self.position.to_homogeneous().cast().unwrap().into(),
            light_color: self.color.cast().unwrap().extend(1.0).into(),
            light_type: [self.light_type.type_id() as u32, 0, 0, 0],
        }
    }

    /// Creates a `UNIFORM` buffer of light.
    /// 
    /// This method is provided only for the advanced developer utility,
    /// and not used by [`Scene`](./struct.Scene.html).
    /// 
    /// # Shader Example
    /// ```glsl
    /// layout(// binding info //) uniform Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    /// ```
    pub fn buffer(&self, device: &Device) -> BufferHandler {
        let light_info = self.light_info();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[light_info]),
            usage: BufferUsage::UNIFORM,
            label: None,
        });
        BufferHandler::new(buffer, std::mem::size_of::<LightInfo>() as u64)
    }
}

impl Default for Light {
    #[inline(always)]
    fn default() -> Light {
        Light {
            position: Point3::origin(),
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
