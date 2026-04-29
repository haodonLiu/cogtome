// Context module - variable resolution and expression evaluation
pub mod expression;
pub mod variables;

// Re-export commonly used types and functions
pub use variables::{ExecContext, StepResult};
