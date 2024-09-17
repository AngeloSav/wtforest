use num_traits::AsPrimitive;
use qwt::{
    utils::{msb, stable_partition_of_2},
    AccessBin, AccessUnsigned, BitVector, BitVectorMut, SpaceUsage, WTIndexable,
};
use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug, Default)]
pub struct TinyWT<T, const SIZE: usize> {
    data: BitVector,
    n: usize,
    n_levels: usize,
    rank_samples: Box<[usize]>,
    ones_level: Box<[usize]>,
    _phantom: PhantomData<T>,
}

const RANK_SAMPLE_SIZE: usize = 512;

impl<T, const SIZE: usize> TinyWT<T, SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
{
    #[must_use]
    pub fn new(sequence: &mut [T]) -> Result<Self, String> {
        if sequence.is_empty() {
            return Ok(Self {
                data: BitVector::default(),
                n: 0,
                n_levels: 0,
                rank_samples: Box::default(),
                ones_level: Box::default(),
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

        //rank support
        let mut rank_samples: Vec<usize> = Vec::new();
        let mut ones_level = Vec::new();

        let mut shift = 1;
        for _ in 0..n_levels {
            let mut this_level = Vec::with_capacity(SIZE);
            let mut rank_level = 0;

            for (i, &s) in sequence.iter().enumerate() {
                if i % RANK_SAMPLE_SIZE == 0 {
                    rank_samples.push(rank_level);
                }
                let bit = ((s >> (n_levels - shift)).as_() & 1) == 1;
                this_level.push(bit);
                if bit {
                    rank_level += 1;
                }
            }

            ones_level.push(rank_level);

            let remaining = SIZE - this_level.len();
            bv.extend(this_level);
            bv.extend_with_zeros(remaining);

            stable_partition_of_2(sequence, n_levels - shift);

            shift += 1;
        }

        Ok(Self {
            data: BitVector::from(bv),
            n,
            n_levels,
            rank_samples: rank_samples.into_boxed_slice(),
            ones_level: ones_level.into_boxed_slice(),
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

    #[inline]
    fn rank_level_pos(&self, l: usize, index: usize) -> usize {
        let actual_pos = SIZE * l + index;
        let blocks_per_level = (self.n - 1) / RANK_SAMPLE_SIZE + 1;

        let r_sample = self.rank_samples[l * blocks_per_level + index / RANK_SAMPLE_SIZE];
        let mut r = 0;

        let mut p = l * SIZE + (index / RANK_SAMPLE_SIZE) * RANK_SAMPLE_SIZE;

        while p < actual_pos {
            let w = self.data.get_bits(p, 64.min(actual_pos - p)).unwrap();
            r += w.count_ones();
            p += 64;
        }

        // println!("r_sample {} | r {}", r_sample, r);

        r_sample + r as usize
    }

    fn get_level_pos(&self, l: usize, index: usize) -> bool {
        unsafe { self.data.get_unchecked(l * SIZE + index) }
    }
}

impl<T, const SIZE: usize> AccessUnsigned for TinyWT<T, SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
{
    type Item = T;

    fn get(&self, i: usize) -> Option<Self::Item> {
        if i > SIZE || i > self.n {
            return None;
        }

        Some(unsafe { self.get_unchecked(i) })
    }

    unsafe fn get_unchecked(&self, i: usize) -> Self::Item {
        for l in 0..self.data.n_lines() {
            self.data.prefetch_line(l);
        }

        let mut cur_i = i;
        let mut result = 0;

        for l in 0..self.n_levels {
            let s = self.get_level_pos(l, cur_i);

            // println!("accessing position {} | s = {}", cur_i, s);

            result = (result << 1) | s as usize;

            let zeros_in_level = self.n - self.ones_level[l];
            let r = self.rank_level_pos(l, cur_i);
            // println!("rank obtained: {}\n", r);

            cur_i = if s { r + zeros_in_level } else { cur_i - r };
        }

        result.as_()
    }
}

// impl<T, const SIZE: usize> RankUnsigned for TinyWT<T, SIZE>
// where
//     T: WTIndexable,
//     usize: AsPrimitive<T>,
// {
//     fn rank(&self, symbol: Self::Item, i: usize) -> Option<usize> {
//         if i > self.n {
//             return None;
//         }

//         Some(unsafe { self.rank_unchecked(symbol, i) })
//     }

//     unsafe fn rank_unchecked(&self, symbol: Self::Item, i: usize) -> usize {
//         let mut cur_i = i;
//         let mut cur_p = 0;
//         let mut n_ones_up_to_level = 0;

//         for l in 0..self.n_levels {
//             let bit = ((symbol.as_() >> (self.n_levels - l - 1)) & 1) == 1;

//             let tmp = self.data.rank1_unchecked((l + 1) * SIZE);
//             let n_ones_in_level = tmp - n_ones_up_to_level;

//             let n_zeros_in_level = self.n - n_ones_in_level;

//             let tmp_p = self.data.rank1_unchecked((SIZE * l) + cur_p) - n_ones_up_to_level;
//             let tmp_i = self.data.rank1_unchecked((SIZE * l) + cur_i) - n_ones_up_to_level;

//             n_ones_up_to_level = tmp;

//             cur_p = if bit {
//                 tmp_p + n_zeros_in_level
//             } else {
//                 cur_p - tmp_p
//             };
//             cur_i = if bit {
//                 tmp_i + n_zeros_in_level
//             } else {
//                 cur_i - tmp_i
//             };
//         }

//         cur_i - cur_p
//     }
// }

impl<T, const SIZE: usize> SpaceUsage for TinyWT<T, SIZE>
where
    T: WTIndexable,
    usize: AsPrimitive<T>,
{
    fn space_usage_byte(&self) -> usize {
        16 + self.data.space_usage_byte()
    }
}
