pub mod wt_forest;
use qwt::{BitVector, RSNarrow, RSWide};
pub use wt_forest::TinyWT;

pub type TinySimple<T, const SIZE: usize> = TinyWT<T, BitVector, SIZE>;
pub type TinyRSW<T, const SIZE: usize> = TinyWT<T, RSWide, SIZE>;
pub type TinyRSN<T, const SIZE: usize> = TinyWT<T, RSNarrow, SIZE>;
