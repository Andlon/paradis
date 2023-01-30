/// This is a baseline example that shows how to implement the even_odd example without the API
/// provided by this crate.
use std::thread::scope;

/// Multiply even numbers by 2, odd numbers by 4 by using separate threads for even and odd numbers.
fn par_even_odd(numbers: &mut [i32]) {
    let n = numbers.len();

    #[derive(Debug, Copy, Clone)]
    struct Ptr(*mut i32);
    unsafe impl Send for Ptr {}
    unsafe impl Sync for Ptr {}

    let ptr = Ptr(numbers.as_mut_ptr());

    scope(|s| {
        s.spawn(|| {
            let ptr = ptr;
            // Transform the even numbers
            for i in (0..n).step_by(2) {
                unsafe {
                    *ptr.0.add(i) *= 2;
                }
            }
        });
        s.spawn(|| {
            let ptr = ptr;
            // Transform the odd numbers
            for i in (1..n).step_by(2) {
                unsafe {
                    *ptr.0.add(i) *= 4;
                }
            }
        });
    });
}

fn main() {
    let mut numbers = [0, 1, 2, 3, 4, 5, 6, 7];
    par_even_odd(&mut numbers);
    assert_eq!(numbers, [0, 4, 4, 12, 8, 20, 12, 28]);
}
