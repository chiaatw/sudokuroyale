use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[derive(Clone)]
pub struct Cancelable {
    flag: Arc<AtomicBool>,
}

impl Cancelable {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub fn is_canceled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        self.flag.store(false, Ordering::Relaxed)
    }
}

pub fn create_signal() -> Cancelable {
    let c = Cancelable::new();
    let c2 = c.clone();
    ctrlc::set_handler(move || c2.cancel()).expect("Error setting Ctrl-C handler");
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_sets_signal() {
        let c = Cancelable::new();
        c.clear(); // sicherstellen, dass Signal false ist
        assert!(!c.is_canceled());

        c.cancel();
        assert!(c.is_canceled());
    }

    #[test]
    fn test_clear_resets_signal() {
        let c = Cancelable::new();
        c.cancel();
        assert!(c.is_canceled());

        c.clear();
        assert!(!c.is_canceled());
    }

    #[test]
    fn test_multiple_cancel_clear() {
        let c = Cancelable::new();

        c.clear();
        assert!(!c.is_canceled());

        c.cancel();
        assert!(c.is_canceled());

        c.cancel();
        assert!(c.is_canceled());

        c.clear();
        assert!(!c.is_canceled());
    }
}
