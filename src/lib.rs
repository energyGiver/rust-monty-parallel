//! # rust-monty-parallel
//!
//! This library implements Montgomery modular exponentiation and speeds up the
//! precomputation stage using multithreading when the "parallel" feature is enabled.
//!
//! The algorithm itself is the right-to-left windowed exponentiation method using Montgomery multiplication.
//! The precomputation of a reduced set of residuals is parallelized if enabled.
//!
//! By default the library runs in single-threaded mode.

pub mod big_digit;
pub mod biguint;
pub mod monty;

pub use monty::monty_modpow;

#[cfg(test)]
mod tests;
