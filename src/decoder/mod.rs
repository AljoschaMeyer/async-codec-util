//! Utilities for working with decores.

use async_codec::AsyncDecode;

mod and_then;
pub use self::and_then::AndThen;
mod decode_exact;
pub use self::decode_exact::{DecodeExact, DecodeExactError};
mod map;
pub use self::map::Map;
mod chain;
pub use self::chain::Chain;

/// Chain a compution on the result of a decoder.
pub fn map<D, F>(decoder: D, f: F) -> Map<D, F> {
    Map::new(decoder, f)
}

/// Create new `Chain` which first decodes via the given `S` and then decodes via the given `T`.
pub fn chain<S, T>(first: S, second: T) -> Chain<S, T>
    where S: AsyncDecode
{
    Chain::new(first, second)
}
