//! Core primitives for slices.
use crate::{IntoRawIndexedAccess, RawIndexedAccess};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct RawSliceAccess<'a, T> {
    ptr: *mut T,
    len: usize,
    marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Sync> Sync for RawSliceAccess<'a, T> {}
unsafe impl<'a, T: Send> Send for RawSliceAccess<'a, T> {}

unsafe impl<'a, T: Sync + Send> RawIndexedAccess for RawSliceAccess<'a, T> {
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
    unsafe fn get_raw(&self, global_index: usize) -> Self::Record {
        &*self.ptr.add(global_index)
    }

    #[inline(always)]
    unsafe fn get_raw_mut(&self, global_index: usize) -> Self::RecordMut {
        &mut *self.ptr.add(global_index)
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T: Sync + Send> IntoRawIndexedAccess for &'a mut [T] {
    type Access = RawSliceAccess<'a, T>;

    fn into_raw_indexed_access(self) -> Self::Access {
        RawSliceAccess {
            ptr: self.as_mut_ptr(),
            len: self.len(),
            marker: PhantomData,
        }
    }
}
