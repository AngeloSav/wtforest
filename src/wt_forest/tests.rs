use qwt::{perf_and_test_utils::gen_sequence, AccessUnsigned};

use crate::{wt_forest::WTForest, TinyWT};

#[test]
fn test_build_tiny() {
    let s = gen_sequence(500, 255);
    // println!("primo elemento: {} {:8b}", s[0], s[0]);
    // let s: Vec<u8> = vec![0, 3, 2, 1, 2, 7];
    let tiny = TinyWT::<_, 512>::new(&mut s.clone()).unwrap();

    // println!("{:?}", tiny);

    for (i, &s) in s.iter().enumerate() {
        assert_eq!(Some(s), tiny.get(i));
        // println!("-------------")
    }
}

#[test]
fn test_build_forest() {
    let s = gen_sequence(200000, 255);
    let forest = WTForest::<_, 1024>::new(&mut s.clone()).unwrap();

    // println!("{:?}", tiny);

    for (i, &s) in s.iter().enumerate() {
        assert_eq!(Some(s), forest.get(i));
    }
}

// #[test]
// fn test_rank_tiny() {
//     let s = vec![1u8, 2, 3, 1, 1, 3, 5, 1, 2];
//     let t = TinyWT::<_, 64>::new(&mut s.clone()).unwrap();

//     for i in 0..s.len() {
//         println!("rank({}, {}) = {:?}", s[i], i, t.rank(s[i], i));
//     }
// }
