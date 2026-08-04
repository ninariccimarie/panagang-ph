//! Scam report intake.

mod models;
mod repository;
mod service;

pub use models::Report;
pub use service::{ReportService, SubmitReportInput};
