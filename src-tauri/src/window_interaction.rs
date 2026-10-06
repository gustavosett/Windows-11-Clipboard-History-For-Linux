use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Tracks the GTK activation followed by compositor focus handoff when moving
/// or resizing. Initial activation must not end protection for the operation.
#[derive(Default)]
pub(crate) struct NativeWindowInteraction {
    // 0 = none, 1 = pressed before handoff, 2 = compositor has taken focus.
    phase: AtomicU8,
    pointer_down: AtomicBool,
}

impl NativeWindowInteraction {
    pub(crate) fn begin_press(
        &self,
        resizable: bool,
        maximized: bool,
        fullscreen: bool,
        on_border: bool,
        on_drag_handle: bool,
    ) {
        // Every in-window press needs held-click protection, even when native
        // resizing/moving is unavailable in this window state.
        let native_operation =
            !maximized && !fullscreen && ((resizable && on_border) || on_drag_handle);
        self.begin(native_operation);
    }

    pub(crate) fn begin(&self, protect: bool) {
        self.pointer_down.store(true, Ordering::SeqCst);
        self.phase.store(u8::from(protect), Ordering::SeqCst);
    }

    pub(crate) fn release(&self) {
        self.pointer_down.store(false, Ordering::SeqCst);
        let _ = self.phase.compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst);
    }

    pub(crate) fn reset(&self) {
        self.pointer_down.store(false, Ordering::SeqCst);
        self.phase.store(0, Ordering::SeqCst);
    }

    pub(crate) fn focus_gained(&self) {
        // Only a return AFTER the compositor's focus loss ends protection.
        if self
            .phase
            .compare_exchange(2, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // The compositor may consume the release during move/resize.
            self.pointer_down.store(false, Ordering::SeqCst);
        }
    }

    pub(crate) fn focus_lost(&self) -> bool {
        let _ = self
            .phase
            .compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst);
        self.is_protected()
    }

    pub(crate) fn is_protected(&self) -> bool {
        self.phase.load(Ordering::SeqCst) != 0 || self.pointer_down.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::NativeWindowInteraction;

    #[test]
    fn held_clicks_survive_maximized_fullscreen_and_fixed_size_windows() {
        for (resizable, maximized, fullscreen) in
            [(true, true, false), (true, false, true), (false, false, false)]
        {
            let interaction = NativeWindowInteraction::default();
            interaction.begin_press(resizable, maximized, fullscreen, true, false);
            interaction.focus_gained();
            assert!(interaction.focus_lost());
            interaction.focus_gained();
            assert!(interaction.is_protected());
            interaction.release();
            assert!(!interaction.focus_lost());
        }
    }

    #[test]
    fn recorded_resize_and_drag_sequences_preserve_protection() {
        // Fedora trace: resize produces one activation after press; moving
        // produces two. Both then hand focus to the compositor.
        for activations_after_press in [1, 2] {
            let interaction = NativeWindowInteraction::default();
            interaction.focus_gained();
            interaction.begin(true);
            for _ in 0..activations_after_press {
                interaction.focus_gained();
                assert!(interaction.is_protected());
            }
            assert!(interaction.focus_lost());
            assert!(interaction.focus_lost());

            // Returning after the native operation must restore outside-click
            // dismissal, rather than leaving protection stuck on.
            interaction.focus_gained();
            assert!(!interaction.focus_lost());
        }
    }

    #[test]
    fn ordinary_clicks_and_hiding_clear_unused_protection() {
        let interaction = NativeWindowInteraction::default();
        interaction.begin(true);
        interaction.begin(false);
        interaction.release();
        assert!(!interaction.focus_lost());
    }

    #[test]
    fn held_inside_click_is_protected_until_release() {
        let interaction = NativeWindowInteraction::default();
        interaction.begin(false);
        interaction.focus_gained();
        assert!(interaction.focus_lost());
        assert!(interaction.focus_lost());
        interaction.focus_gained();
        assert!(interaction.is_protected());
        interaction.release();
        assert!(!interaction.focus_lost());
        interaction.begin(false);
        interaction.reset();
        assert!(!interaction.is_protected());
    }
}
