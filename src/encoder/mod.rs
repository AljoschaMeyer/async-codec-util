//! Utilities for working with encoders.

mod chain;
pub use self::chain::Chain;

/// Chain two encoders, encoding them in sequence.
pub fn chain<S, T>(first: S, second: T) -> Chain<S, T> {
    Chain::new(first, second)
}
