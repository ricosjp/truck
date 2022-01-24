use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, Debug)]
struct SliceDisplay<'a, T>(&'a [T]);

impl<'a> Display for SliceDisplay<'a, f64> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.iter().enumerate().try_for_each(|(i, x)| {
            if i != 0 {
                f.write_str(", ")?;
            }
            if f64::abs(*x) < 1.0e-6 {
                f.write_str("0.0")
            } else if f64::abs(*x) < 1.0e-2 && *x != 0.0 {
                f.write_fmt(format_args!("{:.7E}", x))
            } else {
                f.write_fmt(format_args!("{:?}", x))
            }
        })?;
        f.write_str(")")
    }
}

impl<'a> Display for SliceDisplay<'a, usize> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.iter().enumerate().try_for_each(|(i, x)| {
            if i != 0 {
                f.write_str(", ")?;
            }
            Display::fmt(x, f)
        })?;
        f.write_str(")")
    }
}

impl<'a> Display for SliceDisplay<'a, String> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.iter().enumerate().try_for_each(|(i, x)| {
            if i != 0 {
                f.write_str(", ")?;
            }
            f.write_fmt(format_args!("'{}'", x))
        })?;
        f.write_str(")")
    }
}

impl<'a> Display for SliceDisplay<'a, SliceDisplay<'a, f64>> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.iter().enumerate().try_for_each(|(i, x)| {
            if i != 0 {
                f.write_str(", ")?;
            }
            Display::fmt(x, f)
        })?;
        f.write_str(")")
    }
}

#[derive(Clone, Debug)]
struct IndexSliceDisplay<I>(I);

impl<I: Clone + Iterator<Item = usize>> Display for IndexSliceDisplay<I> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.clone().enumerate().try_for_each(|(i, idx)| {
            if i != 0 {
                f.write_fmt(format_args!(", #{}", idx))
            } else {
                f.write_fmt(format_args!("#{}", idx))
            }
        })?;
        f.write_str(")")
    }
}

impl<'a, I: Clone + Iterator<Item = usize>> Display for SliceDisplay<'a, IndexSliceDisplay<I>> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.iter().enumerate().try_for_each(|(i, x)| {
            if i != 0 {
                f.write_str(", ")?;
            }
            Display::fmt(x, f)
        })?;
        f.write_str(")")
    }
}

#[derive(Clone, Debug)]
pub struct StepDisplay<T> {
    entity: T,
    idx: usize,
}

impl<'a, T> Display for SliceDisplay<'a, StepDisplay<T>>
where
    StepDisplay<T>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.0.iter().try_for_each(|x| Display::fmt(x, f))
    }
}

impl<T> StepDisplay<T> {
    #[inline]
    pub fn new(entity: T, idx: usize) -> Self {
        Self { entity, idx }
    }
}

pub trait StepLength {
    fn step_length(&self) -> usize;
}

macro_rules! impl_step_length {
    ($type: ty, $len: expr) => {
        impl<'a> StepLength for $type {
            #[inline]
            fn step_length(&self) -> usize {
                $len
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct StepHeaderDescriptor {
    pub file_name: String,
    pub time_stamp: String,
    pub authors: Vec<String>,
    pub organization: Vec<String>,
    pub origination_system: String,
    pub authorization: String,
}

#[derive(Clone, Debug)]
pub struct StepHeader {
    file_name: String,
    time_stamp: String,
    authors: Vec<String>,
    organization: Vec<String>,
    origination_system: String,
    authorization: String,
    schema: String,
}

impl Default for StepHeaderDescriptor {
    fn default() -> Self {
        Self {
            file_name: Default::default(),
            time_stamp: chrono::Utc::now().naive_local().to_string(),
            authors: Default::default(),
            organization: Default::default(),
            origination_system: Default::default(),
            authorization: Default::default(),
        }
    }
}

impl Display for StepHeader {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let empty_string = [String::new()];
        f.write_fmt(format_args!(
            "HEADER;
FILE_DESCRIPTION(('Shape Data from Truck'), '2;1');
FILE_NAME('{}', '{}', ({}), ({}), 'truck', '{}', '{}');
FILE_SCHEMA(('{}'));
ENDSEC;\n",
            self.file_name,
            self.time_stamp,
            if self.authors.is_empty() {
                SliceDisplay(&empty_string)
            } else {
                SliceDisplay(&self.authors)
            },
            if self.organization.is_empty() {
                SliceDisplay(&empty_string)
            } else {
                SliceDisplay(&self.organization)
            },
            self.origination_system,
            self.authorization,
            self.schema,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct CompleteStepDisplay<T> {
    display: T,
    header: StepHeader,
}

impl<T: Display> Display for CompleteStepDisplay<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "ISO-10303-21;\n{}DATA;\n{}ENDSEC;\nEND-ISO-10303-21;\n",
            self.header, self.display,
        ))
    }
}

impl<T> CompleteStepDisplay<StepDisplay<T>> {
    #[inline]
    pub fn new(x: T, header: StepHeaderDescriptor) -> Self {
        CompleteStepDisplay {
            display: StepDisplay::new(x, 1),
            header: StepHeader {
                file_name: header.file_name,
                time_stamp: header.time_stamp,
                authors: header.authors,
                organization: header.organization,
                origination_system: header.origination_system,
                authorization: header.authorization,
                schema: "ISO-10303-042".to_string(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct SolidStepDisplay<T> {
    display: T,
    header: StepHeader,
}

impl<T: Display> Display for SolidStepDisplay<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "ISO-10303-21;{}DATA;
#1 = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4, #10);
#4 = PRODUCT_DEFINITION_SHAPE('','', #5);
#5 = PRODUCT_DEFINITION('design','', #6, #9);
#6 = PRODUCT_DEFINITION_FORMATION('','', #7);
#7 = PRODUCT('','','', (#8));
#8 = PRODUCT_CONTEXT('', #2, 'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition', #2, 'design');
#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), #11);
#11 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3) 
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#15 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #12, 'distance_accuracy_value','confusion accuracy');
{}ENDSEC;\nEND-ISO-10303-21;\n",
            self.header, self.display,
        ))
    }
}

impl<T> SolidStepDisplay<StepDisplay<T>> {
    #[inline]
    pub fn new(x: T, header: StepHeaderDescriptor) -> Self {
        Self {
            display: StepDisplay::new(x, 16),
            header: StepHeader {
                file_name: header.file_name,
                time_stamp: header.time_stamp,
                authors: header.authors,
                organization: header.organization,
                origination_system: header.origination_system,
                authorization: header.authorization,
                schema: "ISO-10303-203".to_string(),
            },
        }
    }
}

mod geometry;
mod topology;
