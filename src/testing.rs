//! Helpers for testing implementations of the async-codec traits.

use std::fmt::Debug;

use async_codec::{AsyncDecode, DecodeError, AsyncEncode, AsyncEncodeLen};
use futures_io::{AsyncRead, AsyncWrite};
use futures_io::ErrorKind::{UnexpectedEof, WriteZero};
use futures_executor::block_on;
use futures_util::FutureExt;

use super::{decode, encode};

/// Returns whether the given decoder returns an error of kind `UnexpectedEof` when trying to fully
/// decode from the given reader.
pub fn unexpected_eof_errors<R: AsyncRead, D: AsyncDecode<R>>(reader: R, dec: D) -> bool {
    match block_on(decode(reader, dec)) {
        Ok(_) => false,
        Err((_, err)) => {
            match err {
                DecodeError::ReaderError(err) => err.kind() == UnexpectedEof,
                DecodeError::DataError(_) => false,
            }
        }
    }
}

/// Returns whether the given encoder returns an error of kind `WriteZero` when trying to fully
/// encode into the given writer.
pub fn write_zero_errors<W: AsyncWrite, C: AsyncEncode<W>>(writer: W, co: C) -> bool {
    match block_on(encode(writer, co)) {
        Ok(_) => false,
        Err((_, err)) => err.kind() == WriteZero,
    }
}

fn test_codec_intern<R: AsyncRead, W: AsyncWrite, D: AsyncDecode<R>, C: AsyncEncode<W>>
    (reader: R,
     writer: W,
     dec: D,
     co: C)
     -> (D::Item, usize, usize)
    where D::Error: Debug
{
    let c = encode(writer, co);
    let d = decode(reader, dec);

    let c = c.map(|(_, written)| (None, written))
        .map_err(|(_, err)| panic!("{:?}", err));
    let d = d.map(|(_, item, read)| (Some(item), read))
        .map_err(|(_, err)| panic!("{:?}", err));
    block_on(c.join(d)
                 .map(|((_, written), (item, read)): ((Option<D::Item>, usize),
                                                      (Option<D::Item>, usize))| {
                          let item = item.unwrap();
                          (item, written, read)
                      }))
            .map_err(|_| unreachable!())
            .unwrap()
}

/// Run an encoder and a decoder concurrently, returning the decoded output and whether the encoder
/// produced as many bytes as the decoder consumed.
pub fn test_codec<R: AsyncRead, W: AsyncWrite, D: AsyncDecode<R>, C: AsyncEncode<W>>
    (reader: R,
     writer: W,
     dec: D,
     co: C)
     -> (D::Item, bool)
    where D::Error: Debug
{
    let (item, written, read) = test_codec_intern(reader, writer, dec, co);
    (item, written == read)
}

/// Run an encoder and a decoder concurrently, returning the decoded output and whether the encoder
/// produced as many bytes as it promised and as the decoder consumed.
pub fn test_codec_len<R: AsyncRead, W: AsyncWrite, D: AsyncDecode<R>, C: AsyncEncodeLen<W>>
    (reader: R,
     writer: W,
     dec: D,
     co: C)
     -> (D::Item, bool)
    where D::Error: Debug
{
    let expected_len = co.remaining_bytes();
    let (item, written, read) = test_codec_intern(reader, writer, dec, co);
    (item, written == read && written == expected_len)
}
