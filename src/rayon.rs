use crate::{IntoRawIndexedAccess, RawIndexedAccess};
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

#[derive(Debug)]
pub struct RawAccessParIter<Access>(Access);

impl<Access> RawAccessParIter<Access> {
    pub fn from_access(into_access: impl IntoRawIndexedAccess<Access = Access>) -> Self {
        let access = into_access.into_raw_indexed_access();
        Self(access)
    }
}

pub fn par_iter_from_access<Access>(
    access: impl IntoRawIndexedAccess<Access = Access>,
) -> RawAccessParIter<Access> {
    RawAccessParIter::from_access(access)
}

struct AccessProducerMut<Access> {
    access: Access,
    start_idx: usize,
    end_idx: usize,
}

impl<Access: RawIndexedAccess> Iterator for AccessProducerMut<Access> {
    type Item = Access::RecordMut;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_idx < self.end_idx {
            let item = unsafe { self.access.get_raw_mut(self.start_idx) };
            self.start_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<Access: RawIndexedAccess> ExactSizeIterator for AccessProducerMut<Access> {}

impl<Access: RawIndexedAccess> DoubleEndedIterator for AccessProducerMut<Access> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end_idx > self.start_idx {
            let item = unsafe { self.access.get_raw_mut(self.start_idx) };
            self.end_idx -= 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<Access> Producer for AccessProducerMut<Access>
where
    Access: RawIndexedAccess,
{
    type Item = Access::RecordMut;
    type IntoIter = Self;

    fn into_iter(self) -> Self::IntoIter {
        AccessProducerMut {
            access: self.access,
            start_idx: self.start_idx,
            end_idx: self.end_idx,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        debug_assert!(index < (self.end_idx - self.start_idx));
        // SAFETY: The two producers both obtain raw access to the underlying data structure,
        // but they work on non-overlapping index sets
        let left = Self {
            access: unsafe { self.access.clone_access() },
            start_idx: self.start_idx,
            end_idx: self.start_idx + index,
        };
        let right = Self {
            access: unsafe { self.access.clone_access() },
            start_idx: left.end_idx,
            end_idx: self.end_idx,
        };
        (left, right)
    }
}

impl<Access> ParallelIterator for RawAccessParIter<Access>
where
    Access: RawIndexedAccess,
    Access::RecordMut: Send,
{
    type Item = Access::RecordMut;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

impl<Access> IndexedParallelIterator for RawAccessParIter<Access>
where
    Access: RawIndexedAccess,
    Access::RecordMut: Send,
{
    fn len(&self) -> usize {
        self.0.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        let access = self.0;
        callback.callback(AccessProducerMut {
            start_idx: 0,
            end_idx: access.len(),
            access,
        })
    }
}
