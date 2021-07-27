use super::encoders;
use super::frame::Plane;
use super::stream::StreamWriter;
use std::io;
use std::io::{Read, Write};

pub trait Codec {
    fn encode<T: AsRef<[u16]>, W: Write>(&self, plane: &Plane<T>, dest: W) -> io::Result<()>;
    fn decode<T: AsMut<[u16]>, R: Read>(&self, source: R, plane: &mut Plane<T>) -> io::Result<()>;
}

pub struct FixedPredictionCodec;

trait FixedPrediction {
    fn fixed_prediction(&self, a: u16, b: u16, c: u16) -> i32;
}

impl FixedPrediction for FixedPredictionCodec {
    fn fixed_prediction(&self, a: u16, b: u16, c: u16) -> i32 {
        let min_a_b = a.min(b);
        let max_a_b = a.max(b);

        if c >= max_a_b {
            min_a_b as _
        } else if c <= min_a_b {
            max_a_b as _
        } else {
            a as i32 + b as i32 - c as i32
        }
    }
}

impl Codec for FixedPredictionCodec {
    fn encode<T: AsRef<[u16]>, W: Write>(&self, plane: &Plane<T>, dest: W) -> io::Result<()> {
        let mut bitstream = StreamWriter::new(dest);
        let data = plane.data.as_ref();

        let mut b = 0;
        for row in 0..plane.height {
            let mut a = 0;
            let mut c = 0;
            for col in 0..plane.width {
                let x = data[row * plane.row_stride + col * plane.sample_stride];
                let d = if row > 0 && col + 1 < plane.width {
                    data[(row - 1) * plane.row_stride + (col + 1) * plane.sample_stride]
                } else {
                    0
                };

                let prediction = self.fixed_prediction(a, b, c);
                let prediction_residual = x as i32 - prediction;
                let k = super::encoders::rice_compute_k(a, b, c, d);

                super::encoders::rice_encode_value(k, prediction_residual, &mut bitstream)?;

                c = b;
                b = d;
                a = x;
            }
            b = data[row * plane.row_stride]
        }

        bitstream.flush()
    }

    fn decode<T: AsMut<[u16]>, R: Read>(&self, source: R, plane: &mut Plane<T>) -> io::Result<()> {
        let mut bitstream = super::stream::StreamReader::new(source);
        let data = plane.data.as_mut();

        let mut b = 0;

        for row in 0..plane.height {
            let mut a = 0;
            let mut c = 0;

            for col in 0..plane.width {
                let d = if row > 0 && col + 1 < plane.width {
                    data[(row - 1) * plane.row_stride + (col + 1) * plane.sample_stride]
                } else {
                    0
                };

                let prediction = self.fixed_prediction(a, b, c);
                let k = encoders::rice_compute_k(a, b, c, d);
                let prediction_residual = super::encoders::rice_decode_value(k, &mut bitstream)?;

                let x = (prediction + prediction_residual) as u16;
                data[row * plane.row_stride + col * plane.sample_stride] = x;

                c = b;
                b = d;
                a = x;
            }
            b = data[row * plane.row_stride];
        }
        Ok(())
    }
}
