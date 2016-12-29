/// Df2 files are waveform files from an Optech LiDAR system.
#[deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate byteorder;
extern crate rustc_serialize;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, BufReader, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

/// Our custom error enum.
#[derive(Debug)]
pub enum Error {
    /// Shot has an invalid offset.
    InvalidOffset { shot_number: u16, offset: u16 },
    /// The shot number is invalid.
    InvalidShotNumber(u16),
    /// Wrapper around `std::io::Error`.
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
        // FIXME this isn't exactly correct, a spare byte could be allowed
        let number = match self.reader.read_u16::<LittleEndian>() {
            Ok(number) => number,
            Err(err) => {
                match err.kind() {
                    ErrorKind::UnexpectedEof => return Ok(None),
                    _ => return Err(err.into()),
                }
            }
        };
        let offset = self.reader.read_u16::<LittleEndian>()?;
        let mut bytes_remaining = offset * 2;
        let outgoing = Segment::from_read(&mut self.reader)?;
        bytes_remaining -= outgoing.len();
        let mut segments = Vec::new();
        while bytes_remaining > 0 {
            let segment = Segment::from_read(&mut self.reader)?;
            if segment.len() > bytes_remaining {
                return Err(Error::InvalidOffset {
                    shot_number: number,
                    offset: offset,
                });
            }
            bytes_remaining -= segment.len();
            segments.push(segment);
        }
        Ok(Some(Shot {
            number: number,
            outgoing: outgoing.data,
            segments: segments,
        }))
    }
}

impl<R: Read + Seek> Reader<R> {
    /// Seeks to the shot with the given number.
    ///
    /// Seeking to the n + 1 shot is not an error, but any shots after that *are* errors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use df2::Reader;
    /// let mut reader = Reader::from_path("data/two-shots.df2").unwrap();
    /// assert!(reader.seek(0).is_err());
    /// assert!(reader.seek(1).is_ok());
    /// assert!(reader.seek(2).is_ok());
    /// assert!(reader.seek(3).is_ok());
    /// assert!(reader.seek(4).is_err());
    /// ```
    pub fn seek(&mut self, number: u16) -> Result<()> {
        if number == 0 {
            return Err(Error::InvalidShotNumber(number));
        }
        // TODO optimize by saving locations?
        self.reader.seek(SeekFrom::Start(2))?;
        let mut position: u64 = 2;
        let mut current = 1;
        while current < number {
            let offset = self.reader.read_u16::<LittleEndian>()? * 2 + 2;
            position += 2;
            let new_position = self.reader.seek(SeekFrom::Current(offset as i64))?;
            if new_position != position + offset as u64 {
                return Err(Error::InvalidShotNumber(number));
            }
            position += offset as u64;
            current += 1;
        }
        self.reader.seek(SeekFrom::Current(-2))?;
        Ok(())
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
#[derive(Debug, PartialEq, RustcEncodable)]
pub struct Shot {
    /// The shot number (one-indexed).
    pub number: u16,
    /// The outgoing laser pulse.
    pub outgoing: Vec<u16>,
    /// The waveform segments.
    pub segments: Vec<Segment>,
}

/// A waveform segment.
#[derive(Debug, PartialEq, RustcEncodable)]
pub struct Segment {
    /// The waveform samples.
    pub data: Vec<u16>,
    /// The time interval, in cycles.
    pub time_interval: u16,
}

impl Segment {
    /// Reads a Segment from a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use df2::Segment;
    /// let segment = Segment::from_path("data/one-segment.bin").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Segment> {
        File::open(path)
            .map_err(|err| err.into())
            .and_then(|mut file| Segment::from_read(&mut file))
    }

    /// Reads a Segment from a `Read`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use df2::Segment;
    /// let mut file = File::open("data/one-segment.bin").unwrap();
    /// let segment = Segment::from_read(&mut file).unwrap();
    /// ```
    pub fn from_read<R: Read>(read: &mut R) -> Result<Segment> {
        let nsamples = read.read_u16::<LittleEndian>()?;
        let data = (0..nsamples).map(|_| read.read_u16::<LittleEndian>().map_err(|err| err.into()))
            .collect::<Result<Vec<u16>>>()?;
        let time_interval = read.read_u16::<LittleEndian>()?;
        // reserved
        let _ = read.read_u16::<LittleEndian>()?;
        Ok(Segment {
            data: data,
            time_interval: time_interval,
        })
    }

    /// Returns the length of this segment in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use df2::Segment;
    /// let segment = Segment::from_path("data/one-segment.bin").unwrap();
    /// assert_eq!(110, segment.len());
    /// ```
    pub fn len(&self) -> u16 {
        self.data.len() as u16 * 2 + 6
    }

    /// Returns true if this segment has no data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use df2::Segment;
    /// let segment = Segment { data: Vec::new(), time_interval: 0 };
    /// assert!(segment.is_empty());
    /// let segment = Segment::from_path("data/one-segment.bin").unwrap();
    /// assert!(!segment.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
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

    #[test]
    fn reader_seek_identical() {
        let shot_by_iter =
            Reader::from_path("data/two-shots.df2").unwrap().skip(1).next().unwrap().unwrap();
        let mut reader = Reader::from_path("data/two-shots.df2").unwrap();
        reader.seek(2).unwrap();
        let shot_by_seek = reader.read_one().unwrap().unwrap();
        assert_eq!(shot_by_iter, shot_by_seek);
    }

    #[test]
    fn reader_seek_skip_two() {
        let mut reader = Reader::from_path("data/four-shots.df2").unwrap();
        reader.seek(3).unwrap();
        assert_eq!(2, reader.collect::<Result<Vec<Shot>>>().unwrap().len());
    }

    #[test]
    fn segment_from_path() {
        assert!(Segment::from_path("data/one-segment.bin").is_ok());
    }

    #[test]
    fn segment_len() {
        let segment = Segment::from_path("data/one-segment.bin").unwrap();
        assert_eq!(110, segment.len());
    }
}
