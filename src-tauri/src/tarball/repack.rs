use crate::pull_error::PullError;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression as GzipLevel;
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub enum LayerCompression { Gzip, Zstd, None }

pub fn sniff(data: &[u8]) -> LayerCompression {
    if data.len() >= 2 && &data[..2] == &[0x1f, 0x8b] { LayerCompression::Gzip }
    else if data.len() >= 4 && &data[..4] == &[0x28, 0xb5, 0x2f, 0xfd] { LayerCompression::Zstd }
    else { LayerCompression::None }
}

pub fn rezip(data: &[u8]) -> Result<Vec<u8>, PullError> {
    let decompressed = match sniff(data) {
        LayerCompression::Gzip => { let mut d = GzDecoder::new(data); let mut out = Vec::new(); d.read_to_end(&mut out).map_err(|e| PullError::Decompress(e.to_string()))?; out }
        LayerCompression::Zstd => { let mut d = zstd::Decoder::new(data).map_err(|e| PullError::Decompress(e.to_string()))?; let mut out = Vec::new(); d.read_to_end(&mut out).map_err(|e| PullError::Decompress(e.to_string()))?; out }
        LayerCompression::None => data.to_vec(),
    };
    let mut out = Vec::with_capacity(decompressed.len() / 2);
    let mut encoder = GzEncoder::new(&mut out, GzipLevel::new(6));
    encoder.write_all(&decompressed).map_err(|e| PullError::Regzip(e.to_string()))?;
    encoder.finish().map_err(|e| PullError::Regzip(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    #[test]
    fn sniff_gzip() {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(b"hello").unwrap();
        let gz = e.finish().unwrap();
        assert_eq!(sniff(&gz), LayerCompression::Gzip);
    }

    #[test]
    fn sniff_zstd() {
        let z = zstd::encode_all(std::io::Cursor::new(b"hi"), 1).unwrap();
        assert_eq!(sniff(&z), LayerCompression::Zstd);
    }

    #[test]
    fn sniff_none() { assert_eq!(sniff(b"plain"), LayerCompression::None); }

    #[test]
    fn rezip_roundtrip() {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(b"hello world").unwrap();
        let gz = e.finish().unwrap();
        let out = rezip(&gz).unwrap();
        assert_eq!(sniff(&out), LayerCompression::Gzip);
        let mut d = GzDecoder::new(&out[..]);
        let mut s = String::new();
        d.read_to_string(&mut s).unwrap();
        assert_eq!(s, "hello world");
    }
}
