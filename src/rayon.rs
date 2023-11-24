use crate::{IntoUnsyncAccess, UnsyncAccess, DisjointIndices};
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

#[derive(Debug)]
pub struct UnsyncAccessParIter<Access, Indices> {
    access: Access,
    indices: Indices,
}

impl<Access, Indices> UnsyncAccessParIter<Access, Indices> {
    pub fn from_access_and_indices(
        into_access: impl IntoUnsyncAccess<Access = Access>,
        indices: Indices,
    ) -> Self {
        let access = into_access.into_unsync_access();
        Self { access, indices }
    }
}

pub fn disjoint_indices_par_iter<Access, Indices>(
    access: impl IntoUnsyncAccess<Access = Access>,
    indices: Indices
) -> UnsyncAccessParIter<Access, Indices> {
    UnsyncAccessParIter::from_access_and_indices(access, indices)
}

struct AccessProducerMut<'a, Access, Indices> {
    access: Access,
    indices: &'a Indices,
    start_idx: usize,
    end_idx: usize,
}

impl<'a, Access, Indices> Iterator for AccessProducerMut<'a, Access, Indices>
where
    Access: UnsyncAccess<Indices::Index>,
    Indices: DisjointIndices
{
    type Item = Access::RecordMut;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_idx < self.end_idx {
            let item = unsafe {
                let index = self.indices.get_unchecked(self.start_idx);
                self.access.get_unsync_mut(index)
            };
            self.start_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, Access, Indices> ExactSizeIterator for AccessProducerMut<'a, Access, Indices>
where
    Access: UnsyncAccess<Indices::Index>,
    Indices: DisjointIndices
{}

impl<'a, Access, Indices> DoubleEndedIterator for AccessProducerMut<'a, Access, Indices>
where
    Access: UnsyncAccess<Indices::Index>,
    Indices: DisjointIndices
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // TODO: Need to test this impl
        if self.end_idx > self.start_idx {
            self.end_idx -= 1;
            let item = unsafe {
                let index = self.indices.get_unchecked(self.end_idx);
                self.access.get_unsync_mut(index)
            };
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, Access, Indices> Producer for AccessProducerMut<'a, Access, Indices>
where
    Access: UnsyncAccess<Indices::Index>,
    Indices: DisjointIndices,
{
    type Item = Access::RecordMut;
    type IntoIter = Self;

    fn into_iter(self) -> Self::IntoIter {
        AccessProducerMut {
            access: self.access,
            indices: self.indices,
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
            indices: self.indices,
            start_idx: self.start_idx,
            end_idx: self.start_idx + index,
        };
        let right = Self {
            access: self.access,
            indices: self.indices,
            start_idx: left.end_idx,
            end_idx: self.end_idx,
        };
        (left, right)
    }
}

impl<Access, Indices> ParallelIterator for UnsyncAccessParIter<Access, Indices>
where
    Access: UnsyncAccess<Indices::Index>,
    Access::RecordMut: Send,
    Indices: DisjointIndices,
{
    type Item = Access::RecordMut;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.indices.num_indices())
    }
}

impl<Access, Indices> IndexedParallelIterator for UnsyncAccessParIter<Access, Indices>
where
    Access: UnsyncAccess<Indices::Index>,
    Access::RecordMut: Send,
    Indices: DisjointIndices,
{
    fn len(&self) -> usize {
        self.indices.num_indices()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        let access = self.access;
        callback.callback(AccessProducerMut {
            start_idx: 0,
            end_idx: self.indices.num_indices(),
            access,
            indices: &self.indices
        })
    }
}
