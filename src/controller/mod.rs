// Conditional module definitions based on the target platform
#[cfg(all(target_arch = "arm", target_os = "linux"))]
mod manager;

#[cfg(not(all(target_arch = "arm", target_os = "linux")))]
#[path = "manager_stub.rs"]
mod manager;

// Conditional module definitions and re-exports
#[cfg(all(target_arch = "arm", target_os = "linux"))]
mod predictive;

#[cfg(all(target_arch = "arm", target_os = "linux"))]
mod pid;

#[cfg(all(target_arch = "arm", target_os = "linux"))]
mod threshold;

// This section will only be compiled for ARM + Linux targets
#[cfg(all(target_arch = "arm", target_os = "linux"))]
pub(self) use {
    self::pid::PidController,
    manager::Controller,
    predictive::PredictiveController,
    threshold::ThresholdController,
};

// These re-exports are always available
pub use manager::ControlMethod;
pub use manager::ControllerManager;
