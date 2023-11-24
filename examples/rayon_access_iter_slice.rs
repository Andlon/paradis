use paradis::rayon::disjoint_indices_par_iter;
use rayon::iter::ParallelIterator;

fn main() {
    let mut data = vec![1.0; 10000];
    let range = 0..data.len();
    disjoint_indices_par_iter(data.as_mut_slice(), range).for_each(|x| *x *= 2.0);
    assert!(data.iter().all(|&x| x == 2.0));
}
