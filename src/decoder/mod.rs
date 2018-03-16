//! Utilities for working with decores.

mod map;
pub use self::map::Map;

/// Chain a compution on the result of a decoder.
pub fn map<R, D, F>(decoder: D, f: F) -> Map<R, D, F> {
    Map::new(decoder, f)
}
