use std::sync::Mutex;
use std::vec::Vec;

pub(super) struct IDGenerator {
    generator: Mutex<usize>,
}

impl IDGenerator {
    /// constructor  
    pub(super) fn new() -> IDGenerator {
        IDGenerator {
            generator: Mutex::new(0),
        }
    }

    /// get next id
    pub(super) fn generate(&self) -> usize {
        let mut id = self.generator.lock().unwrap();
        *id += 1;
        *id
    }

    /// get the array of ids
    pub(super) fn multi_generate(&self, len: usize) -> Vec<usize> {
        let mut id = self.generator.lock().unwrap();
        (0..len)
            .map(|_| {
                *id += 1;
                *id
            })
            .collect()
    }
}
