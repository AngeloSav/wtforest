use qwt::{perf_and_test_utils::gen_sequence, AccessUnsigned, RSWide};

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
