//! Minimal definitions for big-digit operations.

/// Type alias for a limb (a 64-bit word).
pub type BigDigit = u64;
/// Number of bits per limb.
pub const BITS: u64 = 64;
/// Double precision limb type.
pub type DoubleBigDigit = u128;
