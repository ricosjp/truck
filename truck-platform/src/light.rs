use crate::*;

impl Light {
    #[inline(always)]
    pub(super) fn light_info(&self) -> LightInfo {
        LightInfo {
            light_position: self.position.to_homogeneous().cast().unwrap().into(),
            light_color: self.color.cast().unwrap().extend(1.0).into(),
            light_type: [self.light_type.into(), 0, 0, 0],
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
    #[inline(always)]
    pub fn buffer(&self, device: &Device) -> BufferHandler {
        BufferHandler::from_slice(&[self.light_info()], device, BufferUsages::UNIFORM)
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
    #[inline(always)]
    fn from(light_type: LightType) -> usize {
        match light_type {
            LightType::Point => 0,
            LightType::Uniform => 1,
        }
    }
}

impl From<LightType> for u32 {
    #[inline(always)]
    fn from(light_type: LightType) -> u32 {
        match light_type {
            LightType::Point => 0,
            LightType::Uniform => 1,
        }
    }
}
