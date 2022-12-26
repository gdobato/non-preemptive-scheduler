#[cfg(feature = "arm-cm")]
mod arm_cm;
#[cfg(feature = "arm-cm")]
pub use arm_cm::critical_section;
