//! Testing utilities for AlphaField strategies
//!
//! This module provides a shared test harness and data generators
//! for testing trading strategies across different market conditions.

pub mod data_generators;
pub mod harness;

pub use data_generators::*;
pub use harness::*;
