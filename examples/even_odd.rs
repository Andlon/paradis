use paradis::{IntoRawIndexedAccess, RawIndexedAccess};
use std::thread::scope;

/// Multiply even numbers by 2, odd numbers by 4 by using separate threads for even and odd numbers.
fn par_even_odd(numbers: &mut [i32]) {
    let n = numbers.len();

    // Since creating an access takes a mutable reference to [i32], we know that we hold
    // an exclusive raw access to the data, so we can soundly manipulate its data in parallel, provided we are very careful.
    let access = numbers.into_raw_indexed_access();

    scope(|s| {
        // Transform the even numbers
        s.spawn(|| {
            for i in (0..n).step_by(2) {
                unsafe {
                    *access.get_raw_mut(i) *= 2;
                }
            }
        });
        // Transform the odd numbers
        s.spawn(|| {
            for i in (1..n).step_by(2) {
                unsafe {
                    *access.get_raw_mut(i) *= 4;
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
