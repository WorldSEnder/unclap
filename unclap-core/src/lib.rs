#![warn(unused_crate_dependencies, missing_docs)]
//! A proc macro that generates program configurations for external programs.

mod std_impls;
mod traits;

pub use std_impls::*;
pub use traits::*;
