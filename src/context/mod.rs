// Context module - variable resolution and expression evaluation
pub mod expression;
pub mod variables;

// Re-export commonly used types and functions
pub use expression::is_truthy;
pub use variables::{ExecContext, StepResult};
