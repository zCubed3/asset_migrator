use std::time;

/// Measures time from the creation of this watch to when it exists scope
///
/// Useful for debugging the performance of a closure
pub struct Dropwatch {
    start: time::Instant,
    id: String,
}

impl Dropwatch {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            start: time::Instant::now(),
            id
        }
    }
}

impl Drop for Dropwatch {
    fn drop(&mut self) {
        let elapsed = (time::Instant::now() - start).as_secs_f32();

        println!("{}: {} seconds elapsed", self.id, elapsed);
    }
}
