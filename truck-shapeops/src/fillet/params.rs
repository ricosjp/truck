use std::num::NonZeroUsize;

/// Profile shape for fillet operations.
#[derive(Debug, Clone, Copy, Default)]
pub enum FilletProfile {
    /// Circular arc cross-section (traditional fillet).
    #[default]
    Round,
    /// Flat ruled surface (chamfer/bevel).
    Chamfer,
}

/// Radius specification for fillet operations.
pub enum RadiusSpec {
    /// Constant radius along the entire edge/wire.
    Constant(f64),
    /// Variable radius as a function of normalized parameter `t` in `[0, 1]`.
    ///
    /// Supported for single-edge fillets ([`simple_fillet`](super::simple_fillet),
    /// [`fillet_with_side`](super::fillet_with_side)).
    /// Rejected by [`fillet_along_wire`](super::fillet_along_wire) with
    /// [`FilletError::VariableRadiusUnsupported`](super::FilletError::VariableRadiusUnsupported).
    Variable(Box<dyn Fn(f64) -> f64>),
}

impl std::fmt::Debug for RadiusSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(r) => f.debug_tuple("Constant").field(r).finish(),
            Self::Variable(_) => f.debug_tuple("Variable").field(&"<fn>").finish(),
        }
    }
}

/// Options for fillet operations.
#[derive(Debug)]
pub struct FilletOptions {
    /// Radius specification.
    pub radius: RadiusSpec,
    /// Number of divisions for the rolling ball algorithm. Default: 5.
    pub division: NonZeroUsize,
    /// Profile shape. Default: [`FilletProfile::Round`].
    pub profile: FilletProfile,
}

impl Default for FilletOptions {
    fn default() -> Self {
        Self {
            radius: RadiusSpec::Constant(0.1),
            // SAFETY: 5 != 0
            division: NonZeroUsize::new(5).unwrap(),
            profile: FilletProfile::default(),
        }
    }
}

impl FilletOptions {
    /// Creates options with a constant radius and default division (5).
    pub fn constant(radius: f64) -> Self {
        Self {
            radius: RadiusSpec::Constant(radius),
            ..Default::default()
        }
    }

    /// Creates options with a variable radius function and default division (5).
    pub fn variable(f: impl Fn(f64) -> f64 + 'static) -> Self {
        Self {
            radius: RadiusSpec::Variable(Box::new(f)),
            ..Default::default()
        }
    }

    /// Sets the division count.
    pub fn with_division(mut self, division: NonZeroUsize) -> Self {
        self.division = division;
        self
    }

    /// Sets the profile shape.
    pub fn with_profile(mut self, profile: FilletProfile) -> Self {
        self.profile = profile;
        self
    }
}
