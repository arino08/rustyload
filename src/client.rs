//! Backward compatibility module
//!
//! This module re-exports types from the protocols module for backward compatibility.
//! New code should use the `protocols` module directly.

#[allow(unused_imports)]
pub use crate::protocols::http::HttpMethod;
#[allow(unused_imports)]
pub use crate::protocols::{LoadTestConfig, LoadTestStats, RequestResult};
