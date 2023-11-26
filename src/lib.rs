//! Parallel processing of disjoint indices.
//!
//! **VERY EXPERIMENTAL, DO NOT USE**.

#[cfg(feature = "rayon")]
pub mod rayon;

pub use paradis_core::{slice, IntoUnsyncAccess, UnsyncAccess};
use std::{collections::HashSet, ops::Range};

pub unsafe trait DisjointIndices: Sync + Send {
    type Index;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index;
    fn num_indices(&self) -> usize;
}

unsafe impl DisjointIndices for Range<usize> {
    type Index = usize;

    fn num_indices(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        self.start + i
    }
}

pub unsafe trait DisjointIndexSubsets {
    type IndexSubset<'subset>;

    fn num_subsets(&self) -> usize;
    fn subset_len(&self, subset_index: usize) -> usize;
    fn get_subset<'subset>(&self, subset_index: usize) -> Self::IndexSubset<'subset>;
}

#[derive(Debug, Clone)]
pub struct DisjointIndicesVec {
    indices: Vec<usize>,
    // max_idx: usize,
}

#[derive(Debug)]
pub struct NotDisjoint;

impl DisjointIndicesVec {
    pub fn try_from_index_iter<I>(iter: I) -> Result<Self, NotDisjoint>
    where
        I: IntoIterator<Item = usize>,
    {
        // Remove outer generic call to avoid excessive monomorphization
        Self::try_from_index_iter_inner(iter.into_iter())
    }

    fn try_from_index_iter_inner<I>(iter: I) -> Result<Self, NotDisjoint>
    where
        I: Iterator<Item = usize>,
    {
        // let mut max_idx = 0;
        // TODO: Use faster hash? And/or switch to bitvec for sufficiently large number of indices
        let mut visited_indices = HashSet::new();

        let indices = iter
            .map(|idx| {
                // if idx > max_idx {
                //     max_idx = idx;
                // }
                if visited_indices.insert(idx) {
                    Ok(idx)
                } else {
                    Err(NotDisjoint)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            indices,
            // max_idx,
        })
    }
}

unsafe impl DisjointIndices for DisjointIndicesVec {
    type Index = usize;

    fn num_indices(&self) -> usize {
        self.indices.len()
    }

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        *self.indices.get_unchecked(i)
    }
}
