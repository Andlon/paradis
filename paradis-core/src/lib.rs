//! Core primitives for `paradis`.
//!
//! `paradis-core` contains the core abstractions used by `paradis`. `paradis-core` is expected
//! to need breaking changes very rarely. Hopefully once the APIs are stabilized
//! no further breaking changes are necessary. Therefore, library authors who only want to
//! expose their data structures to `paradis` algorithms should depend on this crate
//! instead `paradis`.

pub mod slice;

/// Facilitates unsynchronized access to (mutable) records stored in the collection.
///
/// The trait provides *unsynchronized* access to (possibly mutable) *records*, defined by the
/// associated types [`Record`][`UnsyncAccess::Record`] and
/// [`RecordMut`][`UnsyncAccess::RecordMut`].
///
/// # Safety
///
/// An implementor must ensure that it is sound for multiple threads to access a single record
/// *immutably*, provided that no thread accesses the same record mutably.
///
/// An implementor must furthermore ensure that it is sound for multiple threads to access
/// *disjoint* records mutably.
///
/// It is the responsibility of the consumer that:
///
/// - If any thread accesses a record mutably, then no other thread must access the same record.
/// - A mutable record must always be exclusive, even on a single thread.
///   In particular, a single thread is not permitted to obtain two records associated with the
///   same index in the collection if either record is mutable.
///
/// TODO: Make the invariants more precise
pub unsafe trait UnsyncAccess<Index = usize>: Sync + Send {
    type Record;
    type RecordMut;

    // TODO: Should this be unsafe instead of using `Clone`? I think so, because otherwise
    // we might obtain an access using safe code, then clone it several times and pass
    // it off to methods that might eventually try to access the same entries
    // TODO: Currently we need this to be unsafe, because a user could otherwise create multiple accesses
    // and pass them into safe routines that assume that the access is exclusive. We might want an intermediate
    // "exclusive access" trait to accommodate this?
    unsafe fn clone_access(&self) -> Self;

    unsafe fn get_unsync(&self, index: Index) -> Self::Record;
    unsafe fn get_unsync_mut(&self, index: Index) -> Self::RecordMut;

    unsafe fn get_unsync_unchecked(&self, index: Index) -> Self::Record;
    unsafe fn get_unsync_unchecked_mut(&self, index: Index) -> Self::RecordMut;

    // TODO: Rename to shape, can we use the same index type to describe the shape?
    // TODO: Actually, maybe we should remove len altogether
    // fn len(&self) -> Index;

    // fn is_in_bounds(&self, index: Index) -> bool;
}

pub trait IntoUnsyncAccess<Index = usize> {
    type Access: UnsyncAccess<Index>;

    fn into_unsync_access(self) -> Self::Access;
}

impl<Index, Access: UnsyncAccess<Index>> IntoUnsyncAccess<Index> for Access {
    type Access = Self;

    fn into_unsync_access(self) -> Self::Access {
        self
    }
}
