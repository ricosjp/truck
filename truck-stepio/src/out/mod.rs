use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, Debug)]
struct SliceDisplay<'a, T>(&'a [T]);

impl<'a, T: Debug + Copy> Display for SliceDisplay<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("(")?;
        self.0.iter().enumerate().try_for_each(|(i, x)| {
            if i != 0 {
                f.write_str(", ")?;
            }
            Debug::fmt(x, f)
        })?;
        f.write_str(")")
    }
}

impl<'a, T: Debug + Copy> Display for SliceDisplay<'a, SliceDisplay<'a, T>> {
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
pub struct CompleteStepDisplay<T>(T);

impl<T: Display> Display for CompleteStepDisplay<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from Truck'),'2;1');
FILE_SCHEMA(('ISO-10303-042'));
ENDSEC;
DATA;\n{}ENDSEC;\nEND-ISO-10303-21;\n",
            self.0,
        ))
    }
}

impl<T> CompleteStepDisplay<StepDisplay<T>> {
    #[inline]
    pub fn new(x: T) -> Self {
        CompleteStepDisplay(StepDisplay::new(x, 1))
    }
}

mod geometry;
mod topology;
