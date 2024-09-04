use qwt::{perf_and_test_utils::gen_sequence, AccessUnsigned};

use crate::TinyRSW;

#[test]
fn test_build() {
    let s = gen_sequence(900, 255);
    let tiny = TinyRSW::<_, 1024>::new(&mut s.clone()).unwrap();

    println!("{:?}", tiny);

    for (i, &s) in s.iter().enumerate() {
        assert_eq!(Some(s), tiny.get(i));
    }
}
