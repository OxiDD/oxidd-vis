/// Signals value changes when explicitly called, or when dropped
pub struct Signaller(Option<Box<dyn FnOnce() -> ()>>);

impl Signaller {
    pub fn new<F: FnOnce() -> () + 'static>(func: F) -> Signaller {
        Signaller(Some(Box::new(func)))
    }

    /// Perform signalling
    pub fn signal(self) {
        // No need to do anything, the drop will perform a signal
    }
}
impl Drop for Signaller {
    fn drop(&mut self) {
        if let Some(func) = self.0.take() {
            func()
        }
    }
}
