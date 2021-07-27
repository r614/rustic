use std::io::{Read, Result, Write};

use super::stream::{StreamReader, StreamWriter};

pub fn rice_compute_k(a: u16, b: u16, c: u16, d: u16) -> u32 {
    let activity_level =
        (d as i32 - b as i32).abs() + (b as i32 - c as i32).abs() + (c as i32 - a as i32).abs();
    let mut k = 0;
    while (3 << k) < activity_level {
        k += 1
    }
    k
}

pub fn rice_decode_value<T: Read>(k: u32, source: &mut StreamReader<T>) -> Result<i32> {
    let mut high_bits = 0;
    while source.read(1)? == 0 {
        high_bits += 1
    }
    let x = (high_bits << k) | source.read(k as _)? as u32;
    Ok((x as i32 >> 1) ^ ((x << 31) as i32 >> 31))
}

pub fn rice_encode_value<T: Write>(k: u32, x: i32, dest: &mut StreamWriter<T>) -> Result<()> {
    let x = ((x >> 30) ^ (2 * x)) as u32;
    let bits = x >> k;

    dest.write(1, (bits + 1) as _)?;
    dest.write((x & ((1 << k) - 1)) as _, k as _)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_value() {
        let mut buf = Vec::new();
        for k in 0..10 {
            for &x in [-38368, -10, -1, 0, 1, 2, 3, 4, 5, 6, 38368, 38369].iter() {
                buf.clear();
                {
                    let mut dest = StreamWriter::new(&mut buf);
                    rice_encode_value(k, x, &mut dest).unwrap();
                    dest.flush().unwrap();
                }
                let mut bitstream = StreamReader::new(&*buf);
                let decoded = rice_decode_value(k, &mut bitstream).unwrap();
                assert_eq!(
                    x, decoded,
                    "k = {}, x = {}, roundtripped = {}",
                    k, x, decoded
                );
            }
        }
    }   
}