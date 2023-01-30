pub mod slice;

#[cfg(feature = "rayon")]
pub mod rayon;

/// Facilitates unsynchronized access to (mutable) records stored in the collection.
///
/// The trait provides *unsynchronized* access to (possibly mutable) *records*, defined by the
/// associated types [`Record`][`RawIndexedAccess::Record`] and
/// [`RecordMut`][`RawIndexedAccess::RecordMut`].
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
pub unsafe trait RawIndexedAccess: Sync + Send {
    type Record;
    type RecordMut;

    // TODO: Should this be unsafe instead of using `Clone`? I think so, because otherwise
    // we might obtain an access using safe code, then clone it several times and pass
    // it off to methods that might eventually try to access the same entries
    unsafe fn clone_access(&self) -> Self;

    unsafe fn get_raw(&self, index: usize) -> Self::Record;
    unsafe fn get_raw_mut(&self, index: usize) -> Self::RecordMut;
    fn len(&self) -> usize;
}

pub trait IntoRawIndexedAccess {
    type Access: RawIndexedAccess;

    fn into_raw_indexed_access(self) -> Self::Access;
}
