//! Core primitives for slices.
use crate::{IntoUnsyncAccess, UnsyncAccess};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct UnsyncSliceAccess<'a, T> {
    ptr: *mut T,
    len: usize,
    marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Sync> Sync for UnsyncSliceAccess<'a, T> {}
unsafe impl<'a, T: Send> Send for UnsyncSliceAccess<'a, T> {}

unsafe impl<'a, T: Sync + Send> UnsyncAccess for UnsyncSliceAccess<'a, T> {
    type Record = &'a T;
    type RecordMut = &'a mut T;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            marker: Default::default(),
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        &*self.ptr.add(index)
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked_mut(&self, index: usize) -> Self::RecordMut {
        &mut *self.ptr.add(index)
    }

    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        index < self.len
    }
}

impl<'a, T: Sync + Send> IntoUnsyncAccess<usize> for &'a mut [T] {
    type Access = UnsyncSliceAccess<'a, T>;

    fn into_unsync_access(self) -> Self::Access {
        UnsyncSliceAccess {
            ptr: self.as_mut_ptr(),
            len: self.len(),
            marker: PhantomData,
        }
    }
}
