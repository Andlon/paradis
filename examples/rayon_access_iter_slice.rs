use paradis::IntoRawIndexedAccess;
use paradis::rayon::par_iter_from_access;
use rayon::iter::ParallelIterator;

fn main() {
    let mut data = vec![1.0; 10000];
    par_iter_from_access(data.into_raw_indexed_access())
        .for_each(|x| *x *= 2.0);
    assert!(data.iter().all(|&x| x == 2.0));
}