use qwt::{
    perf_and_test_utils::{gen_queries, gen_sequence, type_of, TimingQueries},
    utils::msb,
    AccessUnsigned, HQWT256Pfs, QWT256Pfs, SpaceUsage, WT,
};
use wtforest::wt_forest::WTForest;

const N_RUNS: usize = 1;
const TEXT_SIZE: usize = 1 << 31;

fn test_access_latency<T: AccessUnsigned<Item = u8> + SpaceUsage>(
    ds: &T,
    n: usize,
    queries: &[usize],
    file: String,
) {
    let mut t = TimingQueries::new(N_RUNS, queries.len());
    let mut result: u8 = 0;
    for _ in 0..N_RUNS {
        t.start();
        for &pos in queries.iter() {
            let i = (pos * ((result as usize) + 42)) % n;
            result = unsafe { ds.get_unchecked(i) };
        }
        t.stop()
    }

    let (t_min, t_max, t_avg) = t.get();
    println!(
        "RESULT algo={} exp=access_latency input={} n={} logn={:?} min_time_ns={} max_time_ns={} avg_time_ns={} space_in_bytes={} space_in_mib={:.2} n_queries={} n_runs={}",
        type_of(&ds).chars().filter(|c| !c.is_whitespace()).collect::<String>(),
        file,
        n,
        msb(n),
        t_min,
        t_max,
        t_avg,
        ds.space_usage_byte(),
        ds.space_usage_MiB(),
        queries.len(),
        N_RUNS
    );
    println!("Result: {result}");
}

fn main() {
    let s = gen_sequence(TEXT_SIZE, 255);
    let access_queries = gen_queries(1_000_000, TEXT_SIZE);

    let wt = WT::new(&mut s.clone());
    test_access_latency(&wt, s.len(), &access_queries, "random".to_string());

    let qwt = QWT256Pfs::new(&mut s.clone());
    test_access_latency(&qwt, s.len(), &access_queries, "random".to_string());

    let hqwt = HQWT256Pfs::new(&mut s.clone());
    test_access_latency(&hqwt, s.len(), &access_queries, "random".to_string());

    let f = WTForest::<_, 64>::new(&mut s.clone()).unwrap();
    test_access_latency(&f, s.len(), &access_queries, "random".to_string());

    let f = WTForest::<_, 256>::new(&mut s.clone()).unwrap();
    test_access_latency(&f, s.len(), &access_queries, "random".to_string());

    let f = WTForest::<_, 512>::new(&mut s.clone()).unwrap();
    test_access_latency(&f, s.len(), &access_queries, "random".to_string());

    let f = WTForest::<_, 1024>::new(&mut s.clone()).unwrap();
    test_access_latency(&f, s.len(), &access_queries, "random".to_string());

    let f = WTForest::<_, 2048>::new(&mut s.clone()).unwrap();
    test_access_latency(&f, s.len(), &access_queries, "random".to_string());
}
