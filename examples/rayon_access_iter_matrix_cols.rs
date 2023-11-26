use nalgebra::{DMatrix, DVectorView, DVectorViewMut, Dyn, Scalar, U1};
use paradis::rayon::disjoint_indices_par_iter;
use paradis::UnsyncAccess;
use rayon::iter::ParallelIterator;
use std::marker::PhantomData;

/// Facilitates (parallel) unsynchronized access to columns of a DMatrix
pub struct DMatrixColUnsyncAccess<'a, T> {
    ptr: *mut T,
    rows: usize,
    cols: usize,
    marker: PhantomData<&'a T>,
}

impl<'a, T> DMatrixColUnsyncAccess<'a, T> {
    pub fn from_matrix_mut(matrix: &'a mut DMatrix<T>) -> Self {
        DMatrixColUnsyncAccess {
            rows: matrix.nrows(),
            cols: matrix.ncols(),
            marker: Default::default(),
            ptr: matrix.as_mut_ptr(),
        }
    }
}

unsafe impl<'a, T> Send for DMatrixColUnsyncAccess<'a, T> {}
unsafe impl<'a, T> Sync for DMatrixColUnsyncAccess<'a, T> {}

unsafe impl<'a, T: Scalar> UnsyncAccess for DMatrixColUnsyncAccess<'a, T> {
    type Record = DVectorView<'a, T>;
    type RecordMut = DVectorViewMut<'a, T>;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            rows: self.rows,
            cols: self.cols,
            marker: Default::default(),
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        let offset = index * self.rows;
        let len = self.rows;
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr.add(offset), len);
            DVectorView::from_slice_generic(slice, Dyn(len), U1)
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked_mut(&self, index: usize) -> Self::RecordMut {
        let offset = index * self.rows;
        let len = self.rows;
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.ptr.add(offset), len);
            DVectorViewMut::from_slice_generic(slice, Dyn(len), U1)
        }
    }

    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        index < self.cols
    }
}

fn main() {
    let m = 100;
    let n = 1000;
    let mut matrix = DMatrix::repeat(m, n, 2.0);
    let col_access = DMatrixColUnsyncAccess::from_matrix_mut(&mut matrix);
    let indices = 0..n;

    disjoint_indices_par_iter(col_access, indices).for_each(|mut col| {
        assert_eq!(col.nrows(), m);
        assert_eq!(col.ncols(), 1);
        col *= 2.0;
    });

    assert!(matrix.iter().all(|&x| x == 4.0));
}
