//! Config Manager Module
//! Handles persistence of window state (position, monitor) specifically for Wayland usage.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{Monitor, PhysicalPosition, PhysicalSize};

const CONFIG_FILE: &str = "window_state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowState {
    pub monitor_name: Option<String>,
    pub x: i32,
    pub y: i32,
}

pub struct ConfigManager {
    data_dir: PathBuf,
    state: WindowState,
}

impl ConfigManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let mut manager = Self {
            data_dir,
            state: WindowState::default(),
        };
        let _ = manager.load();
        manager
    }

    pub fn get_state(&self) -> WindowState {
        self.state.clone()
    }

    pub fn save_state(&mut self, monitor_name: Option<String>, x: i32, y: i32) {
        self.state = WindowState { monitor_name, x, y };
        let _ = self.save_to_disk();
    }

    /// Calculates the best position for the window.
    /// If saved state is valid, returns it.
    /// If not (or first run), returns default position (Bottom-Center of primary monitor).
    pub fn get_valid_position(
        &self,
        available_monitors: &[Monitor],
        window_size: PhysicalSize<u32>,
    ) -> PhysicalPosition<i32> {
        // 1. Try to restore saved position
        if let Some(saved_monitor_name) = &self.state.monitor_name {
            // Find the monitor by name
            if let Some(monitor) = available_monitors.iter().find(|m| {
                m.name()
                    .is_some_and(|n| n.as_str() == saved_monitor_name.as_str())
            }) {
                // Check if the saved (x, y) is still inside this monitor
                if self.is_position_valid(self.state.x, self.state.y, monitor, window_size) {
                    return PhysicalPosition::new(self.state.x, self.state.y);
                }
            }
        }

        // 2. Fallback: Default to Bottom-Center of Primary (or first available)
        let target_monitor = available_monitors
            .iter()
            .find(|m| m.scale_factor() > 0.0) // Just a check to get first valid one
            .unwrap_or(&available_monitors[0]);

        Self::calculate_bottom_center(target_monitor, window_size)
    }

    fn is_position_valid(
        &self,
        x: i32,
        y: i32,
        monitor: &Monitor,
        window_size: PhysicalSize<u32>,
    ) -> bool {
        let m_pos = monitor.position();
        let m_size = monitor.size();

        // Basic check: Top-Left corner of window is inside monitor?
        x >= m_pos.x
            && x < (m_pos.x + m_size.width as i32)
            && y >= m_pos.y
            && y < (m_pos.y + m_size.height as i32 - (window_size.height as i32 / 2))
    }

    fn calculate_bottom_center(
        monitor: &Monitor,
        window_size: PhysicalSize<u32>,
    ) -> PhysicalPosition<i32> {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let scale_factor = monitor.scale_factor();

        const APP_CSS_WIDTH: f64 = 360.0;
        let effective_width = (APP_CSS_WIDTH * scale_factor) as i32;

        // Padding from bottom to avoid covering taskbars entirely
        let padding = 45;

        // X = center horizontally
        let x = m_pos.x + (m_size.width as i32 / 2) - (effective_width / 2);

        // Y = bottom - window height - padding
        let y = m_pos.y + m_size.height as i32 - window_size.height as i32 - padding;

        PhysicalPosition::new(x, y)
    }

    // --- IO ---

    fn config_path(&self) -> PathBuf {
        self.data_dir.join(CONFIG_FILE)
    }

    fn load(&mut self) -> Result<(), String> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.state = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), String> {
        if !self.data_dir.exists() {
            fs::create_dir_all(&self.data_dir).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(&self.state).map_err(|e| e.to_string())?;
        fs::write(self.config_path(), content).map_err(|e| e.to_string())?;
        Ok(())
    }
}
