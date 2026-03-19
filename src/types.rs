/// A type alias for transaction IDs.
// `u16` mandated by requirements.
pub type TransactionId = u16;

/// A type alias for Client IDs.
// `u32` mandated by requirements.
pub type ClientId = u32;

/// A type alias for monetary values.
// [IEEE 754](https://en.wikipedia.org/wiki/IEEE_754)-style single-precision 32-bit floats in the
// style of `f32` are not guaranteed hold up precision. Because our use case requires precision up
// to 4 decimal digits, a fixed point float is used instead.
// Namely, I am using a 64-bit sized fixed point float where 50 bits are used for the integer part
// and 14 bits are used for the decimal part.
// As a consequence, these are the allowed ranges:
// - Integer part: [0, 1125899906842623]
// - Decimal part: [0, 9999]
pub type Value = fixed::FixedU64<fixed::types::extra::U14>;
