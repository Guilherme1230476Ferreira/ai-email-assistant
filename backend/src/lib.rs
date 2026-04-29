/// Library entry point — re-exports all public modules so that
/// integration tests in `/tests` can import from `ai_email_assistant::`.
pub mod app_error;
pub mod handlers;
pub mod infrastructure;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod router;
pub mod services;
pub mod state;

pub use app_error::AppError;
