//! The plugin's background housekeeping: the service thread.

pub mod service_periods;
pub mod service_thread;

pub use service_periods::ServicePeriods;
pub use service_thread::ServiceThread;
