/// Df2 files are waveform files from an Optech LiDAR system.

use std::path::Path;

/// Our custom error enum.
#[derive(Debug)]
pub enum Error {}

/// Our custom result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Reads df2 waveform data.
#[derive(Debug)]
pub struct Reader;

impl Reader {
    /// Opens a reader for the file at a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use df2::Reader;
    /// let reader = Reader::from_path("../data/one-shot.df2").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(_: P) -> Result<Reader> {
        Ok(Reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_from_path() {
        assert!(Reader::from_path("../data/one-shot.df2").is_ok());
    }
}
