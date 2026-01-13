pub mod onboarding;
pub mod preferences;
mod recording;
pub mod registry;

// Re-export all commands for convenience
pub use onboarding::*;
pub use preferences::*;
pub use recording::*;
