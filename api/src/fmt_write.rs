/// A [`core::fmt::Write`] to [`std::io::Write`] adapter.
pub(crate) struct FmtWrite<W>(pub(crate) W);

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
