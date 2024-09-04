use num_traits::AsPrimitive;
use qwt::{
    utils::{msb, stable_partition_of_2},
    AccessBin, AccessUnsigned, BinWTSupport, BitVector, BitVectorMut, RSWide, RankBin, SpaceUsage,
    WTIndexable,
};
use std::{fmt::Debug, marker::PhantomData};

pub trait SuitableBV:
    From<BitVector> + Debug + AccessBin + RankBin + BinWTSupport + SpaceUsage
{
}
impl SuitableBV for RSWide {}
// impl SuitableBV for RSNarrow {}
// impl SuitableBV for BitVector {}

#[derive(Debug, Default)]
pub struct TinyWT<T, BV, const SIZE: usize> {
    data: BV,
    n: usize,
    n_levels: usize,
    _phantom: PhantomData<T>,
}

impl<T, BV, const SIZE: usize> TinyWT<T, BV, SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
    BV: SuitableBV,
{
    #[must_use]
    pub fn new(sequence: &mut [T]) -> Result<Self, String> {
        if sequence.len() == 0 {
            return Ok(Self {
                data: BV::from(BitVectorMut::new().into()).into(),
                n: 0,
                n_levels: 0,
                _phantom: PhantomData,
            });
        }
        if sequence.len() > SIZE {
            return Err("Sequence too big!".to_string());
        }

        let n = sequence.len();

        let mut bv = BitVectorMut::new();

        let sigma = *sequence.iter().max().unwrap();
        let log_sigma = msb(sigma) + 1; // Note that sigma equals the largest symbol, so it's already "alphabet_size - 1"
        let n_levels = log_sigma as usize;

        let mut shift = 1;
        for _ in 0..n_levels {
            let mut this_level = Vec::with_capacity(SIZE);

            for &s in sequence.iter() {
                this_level.push(((s >> (n_levels - shift as usize)).as_() & 1) == 1)
            }

            let remaining = SIZE - this_level.len();
            bv.extend(this_level);
            bv.extend_with_zeros(remaining);

            stable_partition_of_2(sequence, n_levels - shift);

            shift += 1;
        }

        Ok(Self {
            data: BV::from(bv.into()),
            n,
            n_levels,
            _phantom: PhantomData,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.n
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    #[must_use]
    pub fn n_levels(&self) -> usize {
        self.n_levels
    }
}

impl<T, BV, const SIZE: usize> AccessUnsigned for TinyWT<T, BV, SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
    BV: SuitableBV,
{
    type Item = T;

    fn get(&self, i: usize) -> Option<Self::Item> {
        if i > SIZE || i > self.n {
            return None;
        }

        Some(unsafe { self.get_unchecked(i) })
    }

    unsafe fn get_unchecked(&self, i: usize) -> Self::Item {
        for l in 0..self.n_levels {
            self.data.prefetch_data(l * SIZE);
            self.data.prefetch_info(l * SIZE);
        }

        let mut cur_i = i;
        let mut result = 0;
        let mut n_ones_up_to_level = 0;

        for l in 0..self.n_levels {
            let s = self.data.get_unchecked(SIZE * l + cur_i);

            // println!("accessing position {} | s = {}", cur_i, s);

            result = (result << 1) | s as usize;

            let tmp = self.data.rank1_unchecked((l + 1) * SIZE);
            let n_ones_in_level = tmp - n_ones_up_to_level;

            let n_zeros_in_level = self.n - n_ones_in_level;
            let rank_in_level = self.data.rank1_unchecked(l * SIZE + cur_i) - n_ones_up_to_level;

            n_ones_up_to_level = tmp;

            cur_i = if s {
                rank_in_level + n_zeros_in_level
            } else {
                cur_i - rank_in_level
            };
        }

        result.as_()
    }
}

impl<T, BV, const SIZE: usize> SpaceUsage for TinyWT<T, BV, SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
    BV: SuitableBV,
{
    fn space_usage_byte(&self) -> usize {
        16 + self.data.space_usage_byte()
    }
}

#[derive(Debug)]
pub struct WTForest<T, BV, const BLOCK_SIZE: usize> {
    forest: Box<[TinyWT<T, BV, BLOCK_SIZE>]>,
    n: usize,
}

impl<T, BV, const BLOCK_SIZE: usize> WTForest<T, BV, BLOCK_SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
    BV: SuitableBV,
{
    pub fn new(sequence: &mut [T]) -> Result<Self, String> {
        let mut forest = Vec::with_capacity(sequence.len() / BLOCK_SIZE);
        let n = sequence.len();

        for s in sequence.chunks_exact_mut(BLOCK_SIZE) {
            forest.push(TinyWT::<T, BV, BLOCK_SIZE>::new(s)?)
        }

        let remainder = sequence.chunks_exact_mut(BLOCK_SIZE).into_remainder();

        if remainder.len() > 0 {
            forest.push(TinyWT::<T, BV, BLOCK_SIZE>::new(remainder)?)
        }

        Ok(Self {
            forest: forest.into_boxed_slice(),
            n,
        })
    }
}

impl<T, BV, const BLOCK_SIZE: usize> AccessUnsigned for WTForest<T, BV, BLOCK_SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
    BV: SuitableBV,
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

impl<T, BV, const BLOCK_SIZE: usize> SpaceUsage for WTForest<T, BV, BLOCK_SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
    BV: SuitableBV,
{
    fn space_usage_byte(&self) -> usize {
        8 + self.forest.iter().fold(0, |a, x| a + x.space_usage_byte())
    }
}
#[cfg(test)]
mod tests;
