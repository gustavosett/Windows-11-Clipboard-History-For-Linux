use std::sync::atomic::{AtomicU64, Ordering};

/// Invalidates activation callbacks when a newer show, hide, or focus loss occurs.
pub(crate) struct PopupActivation(AtomicU64);

impl PopupActivation {
    pub(crate) const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub(crate) fn invalidate(&self) -> u64 {
        self.0.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub(crate) fn is_current(&self, generation: u64) -> bool {
        self.0.load(Ordering::SeqCst) == generation
    }
}

#[cfg(test)]
mod tests {
    use super::PopupActivation;

    #[test]
    fn hide_or_focus_loss_cancels_delayed_activation() {
        let activation = PopupActivation::new();
        let showing = activation.invalidate();
        assert!(activation.is_current(showing));
        activation.invalidate();
        assert!(!activation.is_current(showing));
    }

    #[test]
    fn hide_then_reopen_does_not_revive_old_activation() {
        let activation = PopupActivation::new();
        let old_show = activation.invalidate();
        activation.invalidate();
        let new_show = activation.invalidate();
        assert!(!activation.is_current(old_show));
        assert!(activation.is_current(new_show));
    }
}
