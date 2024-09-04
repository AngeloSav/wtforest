use qwt::{perf_and_test_utils::gen_sequence, AccessUnsigned, RSWide, RankUnsigned};

use crate::{wt_forest::WTForest, TinyRSW};

#[test]
fn test_build_tiny() {
    let s = gen_sequence(900, 255);
    let tiny = TinyRSW::<_, 1024>::new(&mut s.clone()).unwrap();

    println!("{:?}", tiny);

    for (i, &s) in s.iter().enumerate() {
        assert_eq!(Some(s), tiny.get(i));
    }
}

#[test]
fn test_build_forest() {
    let s = gen_sequence(200000, 255);
    let forest = WTForest::<_, RSWide, 1024>::new(&mut s.clone()).unwrap();

    // println!("{:?}", tiny);

    for (i, &s) in s.iter().enumerate() {
        assert_eq!(Some(s), forest.get(i));
    }
}

#[test]
fn test_rank_tiny() {
    let s = vec![1u8, 2, 3, 1, 1, 3, 5, 1, 2];
    let t = TinyRSW::<_, 64>::new(&mut s.clone()).unwrap();

    for i in 0..s.len() {
        println!("rank({}, {}) = {:?}", s[i], i, t.rank(s[i], i));
    }
}
