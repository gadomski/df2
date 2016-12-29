/// Df2 files are waveform files from an Optech LiDAR system.

extern crate byteorder;

use std::fs::File;
use std::io::{self, BufReader, Read, ErrorKind};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

/// Our custom error enum.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

/// Our custom result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Reads df2 waveform data.
#[derive(Debug)]
pub struct Reader<R: Read> {
    reader: R,
}

impl Reader<BufReader<File>> {
    /// Opens a reader for the file at a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use df2::Reader;
    /// let reader = Reader::from_path("data/one-shot.df2").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<BufReader<File>>> {
        let file = BufReader::new(File::open(path)?);
        Ok(Reader { reader: file })
    }
}

impl<R: Read> Reader<R> {
    /// Reads one shot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use df2::Reader;
    /// let mut reader = Reader::from_path("data/one-shot.df2").unwrap();
    /// let shot = reader.read_one().unwrap().unwrap();
    /// ```
    pub fn read_one(&mut self) -> Result<Option<Shot>> {
        let number = match self.reader.read_u16::<LittleEndian>() {
            Ok(number) => number,
            Err(err) => {
                match err.kind() {
                    ErrorKind::UnexpectedEof => return Ok(None),
                    _ => return Err(err.into()),
                }
            }
        };
        Ok(Some(Shot { number: number }))
    }
}

impl<R: Read> Iterator for Reader<R> {
    type Item = Result<Shot>;
    fn next(&mut self) -> Option<Result<Shot>> {
        match self.read_one() {
            Ok(option) => {
                match option {
                    Some(shot) => Some(Ok(shot)),
                    None => None,
                }
            }
            Err(err) => Some(Err(err)),
        }
    }
}

/// A laser shot.
pub struct Shot {
    /// The shot number (one-indexed).
    pub number: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_from_path() {
        assert!(Reader::from_path("data/one-shot.df2").is_ok());
    }

    #[test]
    fn reader_read_one() {
        let mut reader = Reader::from_path("data/one-shot.df2").unwrap();
        let shot = reader.read_one().unwrap().unwrap();
        assert_eq!(1, shot.number);
    }

    #[test]
    fn reader_iterator() {
        let shots: Vec<_> = Reader::from_path("data/one-shot.df2").unwrap().collect();
        assert_eq!(1, shots.len());
    }
}
