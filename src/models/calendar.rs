//! Calendar - Time Availability Management
//!
//! Handles working hours, holidays, and time windows

use serde::{Deserialize, Serialize};

/// Calendar - Defines availability over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    /// Calendar identifier
    pub id: String,
    /// Available time windows
    pub time_windows: Vec<TimeWindow>,
    /// Blocked periods (holidays, maintenance)
    pub blocked_periods: Vec<TimeWindow>,
}

/// Time window - A period of availability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start time (epoch ms)
    pub start_ms: i64,
    /// End time (epoch ms)
    pub end_ms: i64,
}

impl TimeWindow {
    /// Create new time window
    pub fn new(start_ms: i64, end_ms: i64) -> Self {
        Self { start_ms, end_ms }
    }

    /// Duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        self.end_ms - self.start_ms
    }

    /// Check if timestamp falls within window
    pub fn contains(&self, timestamp_ms: i64) -> bool {
        timestamp_ms >= self.start_ms && timestamp_ms < self.end_ms
    }

    /// Check if overlaps with another window
    pub fn overlaps(&self, other: &TimeWindow) -> bool {
        self.start_ms < other.end_ms && self.end_ms > other.start_ms
    }
}

impl Calendar {
    /// Create new calendar
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            time_windows: Vec::new(),
            blocked_periods: Vec::new(),
        }
    }

    /// Create 24/7 calendar
    pub fn always_available(id: &str) -> Self {
        Self {
            id: id.to_string(),
            time_windows: vec![TimeWindow::new(0, i64::MAX)],
            blocked_periods: Vec::new(),
        }
    }

    /// Add time window
    pub fn with_window(mut self, start_ms: i64, end_ms: i64) -> Self {
        self.time_windows.push(TimeWindow::new(start_ms, end_ms));
        self
    }

    /// Add blocked period
    pub fn with_blocked(mut self, start_ms: i64, end_ms: i64) -> Self {
        self.blocked_periods.push(TimeWindow::new(start_ms, end_ms));
        self
    }

    /// Check if time is working time
    pub fn is_working_time(&self, timestamp_ms: i64) -> bool {
        // Check if in any time window
        let in_window = self.time_windows.iter().any(|w| w.contains(timestamp_ms));
        // Check if not blocked
        let not_blocked = !self.blocked_periods.iter().any(|w| w.contains(timestamp_ms));

        in_window && not_blocked
    }

    /// Find next available time from given timestamp
    pub fn next_available_time(&self, from_ms: i64) -> i64 {
        if self.is_working_time(from_ms) {
            return from_ms;
        }

        // Find next window start
        self.time_windows
            .iter()
            .filter(|w| w.end_ms > from_ms)
            .map(|w| w.start_ms.max(from_ms))
            .min()
            .unwrap_or(from_ms)
    }

    /// Calculate available time between two points
    pub fn available_time_between(&self, start_ms: i64, end_ms: i64) -> i64 {
        if self.time_windows.is_empty() {
            return end_ms - start_ms;
        }

        self.time_windows
            .iter()
            .filter(|w| w.overlaps(&TimeWindow::new(start_ms, end_ms)))
            .map(|w| {
                let overlap_start = w.start_ms.max(start_ms);
                let overlap_end = w.end_ms.min(end_ms);
                overlap_end - overlap_start
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window() {
        let window = TimeWindow::new(1000, 2000);

        assert_eq!(window.duration_ms(), 1000);
        assert!(window.contains(1500));
        assert!(!window.contains(2500));
    }

    #[test]
    fn test_calendar_availability() {
        let calendar = Calendar::new("cal1")
            .with_window(0, 8 * 3600 * 1000)      // 0-8 hours
            .with_window(9 * 3600 * 1000, 17 * 3600 * 1000); // 9-17 hours

        assert!(calendar.is_working_time(4 * 3600 * 1000));  // 4 AM
        assert!(!calendar.is_working_time(8 * 3600 * 1000 + 1000)); // 8 AM + 1s
        assert!(calendar.is_working_time(12 * 3600 * 1000)); // 12 PM
    }

    #[test]
    fn test_blocked_periods() {
        let calendar = Calendar::always_available("cal1")
            .with_blocked(5000, 10000);

        assert!(calendar.is_working_time(3000));
        assert!(!calendar.is_working_time(7000));
        assert!(calendar.is_working_time(15000));
    }
}
