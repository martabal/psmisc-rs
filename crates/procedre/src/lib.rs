pub mod cli;
pub mod output;
pub mod process;

#[cfg(all(feature = "rayon", feature = "orx-parallel"))]
compile_error!("feature \"rayon\" and feature \"oxc-parallel\" cannot be enabled at the same time");
