#[cfg(feature = "rayon")]
pub mod rayon;

use std::collections::HashSet;
pub use paradis_core::{slice, IntoRawIndexedAccess, RawIndexedAccess};

pub unsafe trait DisjointIndices {
    fn get_index(&self, i: usize) -> usize;
    fn num_indices(&self) -> usize;
    fn max_index(&self) -> usize;
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
    max_idx: usize,
}

#[derive(Debug)]
pub struct NotDisjoint;

impl DisjointIndicesVec {
    pub fn try_from_index_iter<I>(iter: I) -> Result<Self, NotDisjoint>
    where
        I: IntoIterator<Item=usize>
    {
        // Remove outer generic call to avoid excessive monomorphization
        Self::try_from_index_iter_inner(iter.into_iter())
    }

    fn try_from_index_iter_inner<I>(iter: I) -> Result<Self, NotDisjoint>
    where
        I: Iterator<Item=usize>
    {
        let mut max_idx = 0;
        // TODO: Use faster hash? And/or switch to bitvec for sufficiently large number of indices
        let mut visited_indices = HashSet::new();

        let indices = iter
            .map(|idx| {
                if idx > max_idx {
                    max_idx = idx;
                }
                if visited_indices.insert(idx) {
                    Ok(idx)
                } else {
                    Err(NotDisjoint)
                }
            }).collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            indices,
            max_idx,
        })
    }
}

unsafe impl DisjointIndices for DisjointIndicesVec {
    fn get_index(&self, i: usize) -> usize {
        self.indices[i]
    }

    fn num_indices(&self) -> usize {
        self.indices.len()
    }

    fn max_index(&self) -> usize {
        self.max_idx
    }
}