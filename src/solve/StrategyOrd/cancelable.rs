use std::sync::atomic::{AtomicBool, Ordering};

pub struct Cancelable{}

impl Cancelable {
    pub fn new() -> Self {
        Self{}
    }

    pub fn cancel(&self) {
        SIGNAL.store(true, Ordering::Relaxed);
    }

    pub fn is_canceled(&self) -> bool {
        SIGNAL.load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        SIGNAL.store(false, Ordering::Relaxed)
    }
}

pub fn create_signal() -> Cancelable {
    ctrlc::set_handler(|| SIGNAL.store(true, Ordering::Relaxed))
        .expect("Error setting Ctrl-C handler");

    Cancelable::new()
}

static SIGNAL: AtomicBool = AtomicBool::new(false);

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
