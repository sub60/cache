//! Types related to the `/{user}/{cache}/nar-multiparts` endpoints.

mod complete_request_body;
mod etag;
mod nar_multiparts_response_body;
mod parts_request_body;
mod parts_response_body;

/// TODO: docs.
pub type UploadId = String;

pub use complete_request_body::{
    CompleteRequestBody,
    CompleteRequestBodyFromStrError,
};
pub use etag::{Etag, EtagFromStrError};
pub use nar_multiparts_response_body::NarMultipartsResponseBody;
pub use parts_request_body::{PartsRequestBody, PartsRequestBodyFromStrError};
pub use parts_response_body::{
    PartsResponseBody,
    PartsResponseBodyFromStrError,
};

/// A [`core::fmt::Write`] to [`std::io::Write`] adapter.
struct FmtWrite<W>(W);

impl<W: core::fmt::Write> std::io::Write for FmtWrite<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = str::from_utf8(buf).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, err)
        })?;

        self.0
            .write_str(s)
            .map_err(|_| std::io::Error::other("formatting failed"))?;

        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
