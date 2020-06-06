//! Module that provides code for a ticking clock and frame related issues.

mod frame;

pub use frame::{FrameLimiter, FrameRateLimitConfig, FrameRateLimitStrategy};
