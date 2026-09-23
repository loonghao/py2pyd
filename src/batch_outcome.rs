//! Batch outcome handling shared by both compilation backends.
//!
//! The uv-backed compiler and the legacy Rust-transform compiler run batches
//! under the same contract, and that contract is part of the CLI surface: a
//! batch in which every file failed has to exit non-zero so CI can tell it
//! apart from a successful run.
//!
//! The decision used to be written out twice, once per backend. Two copies of
//! a policy drift apart silently, and the drift only shows up as "with uv a
//! totally failed batch exits 1, without uv it exits 0" — exactly the class of
//! bug that is invisible to a green CI. Both backends now call
//! [`batch_outcome`].

use anyhow::{anyhow, Result};
use log::{info, warn};

/// Turn batch counters into a result.
///
/// Individual file failures are tolerated on purpose so that one broken file
/// does not discard the rest of the batch. A batch in which nothing compiled
/// is a failure, otherwise CI cannot tell it apart from a successful run.
///
/// # Errors
///
/// Returns an error when at least one file failed and none succeeded.
pub fn batch_outcome(success_count: usize, failure_count: usize) -> Result<()> {
    info!("Batch compilation complete: {success_count} succeeded, {failure_count} failed");

    if failure_count == 0 {
        return Ok(());
    }

    if success_count > 0 {
        warn!("{failure_count} file(s) failed to compile, {success_count} succeeded");
        return Ok(());
    }

    Err(anyhow!(
        "Batch compilation failed: all {failure_count} file(s) failed to compile"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_outcome_all_succeeded() {
        assert!(batch_outcome(3, 0).is_ok());
    }

    #[test]
    fn test_batch_outcome_partial_failure_is_tolerated() {
        assert!(batch_outcome(2, 1).is_ok());
    }

    /// Every file failing has to surface as an error so CI notices.
    #[test]
    fn test_batch_outcome_total_failure_is_an_error() {
        let err = batch_outcome(0, 2).unwrap_err().to_string();
        assert!(err.contains("all 2 file(s) failed to compile"), "{err}");
    }

    /// An empty batch is not the same as a failed one: nothing was expected to
    /// compile, so nothing failed.
    #[test]
    fn test_batch_outcome_no_files_is_not_an_error() {
        assert!(batch_outcome(0, 0).is_ok());
    }
}
