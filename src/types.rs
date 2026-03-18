/// A type alias for transaction IDs.
// `u16` mandated by requirements.
pub type TransactionId = u16;

/// A type alias for Client IDs.
// `u32` mandated by requirements.
pub type ClientId = u32;

/// A type alias for monetary values.
// Because `f32` is an [IEEE 754](https://en.wikipedia.org/wiki/IEEE_754)-style single-precision
// 32-bit float, it is known to guarantee precision up to 7 decimal digits; which is more than
// enough for our use case that requires 4 decimal digits.
pub type Value = f32;
