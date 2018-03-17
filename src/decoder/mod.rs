//! Utilities for working with decores.

use async_codec::AsyncDecode;
use futures_io::AsyncRead;

mod map;
pub use self::map::Map;
mod chain;
pub use self::chain::Chain;

/// Chain a compution on the result of a decoder.
pub fn map<R, D, F>(decoder: D, f: F) -> Map<R, D, F> {
    Map::new(decoder, f)
}

/// Create new `Chain` which first decodes via the given `S` and then decodes via the given `T`.
pub fn chain<R, S, T>(first: S, second: T) -> Chain<R, S, T>
    where R: AsyncRead,
          S: AsyncDecode<R>
{
    Chain::new(first, second)
}
