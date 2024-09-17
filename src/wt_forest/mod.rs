use num_traits::AsPrimitive;
use qwt::{AccessUnsigned, SpaceUsage, WTIndexable};
use std::fmt::Debug;

pub mod tinywt;
pub use tinywt::TinyWT;

#[derive(Debug)]
pub struct WTForest<T, const BLOCK_SIZE: usize> {
    forest: Box<[TinyWT<T, BLOCK_SIZE>]>,
    n: usize,
}

impl<T, const BLOCK_SIZE: usize> WTForest<T, BLOCK_SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
{
    pub fn new(sequence: &mut [T]) -> Result<Self, String> {
        let mut forest = Vec::with_capacity(sequence.len() / BLOCK_SIZE);
        let n = sequence.len();

        for s in sequence.chunks_exact_mut(BLOCK_SIZE) {
            forest.push(TinyWT::<T, BLOCK_SIZE>::new(s)?)
        }

        let remainder = sequence.chunks_exact_mut(BLOCK_SIZE).into_remainder();

        if !remainder.is_empty() {
            forest.push(TinyWT::<T, BLOCK_SIZE>::new(remainder)?)
        }

        Ok(Self {
            forest: forest.into_boxed_slice(),
            n,
        })
    }
}

impl<T, const BLOCK_SIZE: usize> AccessUnsigned for WTForest<T, BLOCK_SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
{
    type Item = T;

    fn get(&self, i: usize) -> Option<Self::Item> {
        if i > self.n {
            return None;
        }

        Some(unsafe { self.get_unchecked(i) })
    }

    unsafe fn get_unchecked(&self, i: usize) -> Self::Item {
        self.forest[i / BLOCK_SIZE].get_unchecked(i % BLOCK_SIZE)
    }
}

impl<T, const BLOCK_SIZE: usize> SpaceUsage for WTForest<T, BLOCK_SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
{
    fn space_usage_byte(&self) -> usize {
        8 + self.forest.iter().fold(0, |a, x| a + x.space_usage_byte())
    }
}
#[cfg(test)]
mod tests;
