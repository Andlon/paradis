#[cfg(feature = "rayon")]
pub mod rayon;

pub use paradis_core::{slice, IntoRawIndexedAccess, RawIndexedAccess};
