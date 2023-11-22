use crate::{IntoUnsyncAccess, UnsyncAccess};
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

#[derive(Debug)]
pub struct UnsyncAccessParIter<Access>(Access);

impl<Access> UnsyncAccessParIter<Access> {
    pub fn from_access(into_access: impl IntoUnsyncAccess<Access = Access>) -> Self {
        let access = into_access.into_unsync_access();
        Self(access)
    }
}

pub fn par_iter_from_access<Access>(
    access: impl IntoUnsyncAccess<Access = Access>,
) -> UnsyncAccessParIter<Access> {
    UnsyncAccessParIter::from_access(access)
}

struct AccessProducerMut<Access> {
    access: Access,
    start_idx: usize,
    end_idx: usize,
}

impl<Access: UnsyncAccess> Iterator for AccessProducerMut<Access> {
    type Item = Access::RecordMut;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_idx < self.end_idx {
            let item = unsafe { self.access.get_unsync_mut(self.start_idx) };
            self.start_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<Access: UnsyncAccess> ExactSizeIterator for AccessProducerMut<Access> {}

impl<Access: UnsyncAccess> DoubleEndedIterator for AccessProducerMut<Access> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end_idx > self.start_idx {
            let item = unsafe { self.access.get_unsync_mut(self.start_idx) };
            self.end_idx -= 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<Access> Producer for AccessProducerMut<Access>
where
    Access: UnsyncAccess,
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
        // SAFETY: The two producers both obtain unsyncrhonized access to the underlying data structure,
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

impl<Access> ParallelIterator for UnsyncAccessParIter<Access>
where
    Access: UnsyncAccess,
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

impl<Access> IndexedParallelIterator for UnsyncAccessParIter<Access>
where
    Access: UnsyncAccess,
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
