use std::{
    io::{self, Read, Write},
    path::Path,
};

use thiserror::Error;

use super::codec::Codec;

pub struct Plane<T> {
    pub data: T,
    pub width: usize,
    pub height: usize,
    pub sample_stride: usize,
    pub row_stride: usize,
}

impl<T: AsRef<[u16]>> Plane<T> {
    pub fn sample(&self, col: usize, row: usize) -> u16 {
        self.data.as_ref()[row * self.row_stride + col * self.sample_stride]
    }
}

#[derive(Error, Debug)]
pub enum FrameOpenError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    TiffError(#[from] tiff::TiffError),
    #[error("Unsupported Color Type: {0:?}")]
    UnsupportedColorType(tiff::ColorType),
    #[error("Unsupported Sample Type")]
    UnsupportedSampleType,
}

#[derive(PartialEq)]
pub struct RGB48Frame {
    pub data: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

impl RGB48Frame {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, FrameOpenError> {
        let file = std::fs::File::open(path)?;
        let mut dec =
            tiff::decoder::Decoder::new(file)?.with_limits(tiff::decoder::Limits::unlimited());
        let (width, height) = dec.dimensions()?;

        Ok(match dec.colortype()? {
            tiff::ColorType::RGB(16) => match dec.read_image()? {
                tiff::decoder::DecodingResult::U16(data) => RGB48Frame {
                    data,
                    width: width as _,
                    height: height as _,
                },
                _ => return Err(FrameOpenError::UnsupportedSampleType),
            },
            color_type => return Err(FrameOpenError::UnsupportedColorType(color_type)),
        })
    }

    pub fn planes(&self) -> Vec<Plane<&[u16]>> {
        vec![
            Plane {
                data: &self.data,
                width: self.width,
                height: self.height,
                row_stride: 3 * self.width,
                sample_stride: 3,
            },
            Plane {
                data: &self.data[1..],
                width: self.width,
                height: self.height,
                row_stride: 3 * self.width,
                sample_stride: 3,
            },
            Plane {
                data: &self.data[2..],
                width: self.width,
                height: self.height,
                row_stride: 3 * self.width,
                sample_stride: 3,
            },
        ]
    }

    pub fn encode<W: Write>(&self, mut dest: W, c: &impl Codec) -> io::Result<()> {
        for plane in self.planes() {
            c.encode(&plane, dest.by_ref())?;
        }
        Ok(())
    }

    pub fn decode<R: Read>(
        &self,
        mut source: R,
        width: usize,
        height: usize,
        c: &impl Codec,
    ) -> io::Result<Self> {
        let mut ret = Self {
            data: vec![0; width * height * 3],
            width,
            height,
        };

        for plane in 0..3 {
            c.decode(
                &mut source,
                &mut Plane {
                    data: &mut ret.data[plane..],
                    width: width,
                    height: height,
                    row_stride: 3 * width,
                    sample_stride: 3,
                },
            )?;
        }
        Ok(ret)
    }
}