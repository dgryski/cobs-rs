pub struct Encoder {}

#[derive(Debug, Clone, Copy)]
pub struct CorruptError {}

impl Encoder {
    pub fn encode(src: &[u8]) -> Vec<u8> {
        let mut dst = Vec::<u8>::new();

        if src.len() == 0 {
            return dst;
        }

        let encoded_size = (src.len() as f64 * 1.04) as usize;
        dst.reserve(encoded_size);

        dst.push(0);

        let mut ptr = 0usize;
        let mut code = 1u8;

        for &b in src {
            if b == 0 {
                dst[ptr] = code;
                ptr = dst.len();
                dst.push(0);
                code = 1;
                continue;
            }

            dst.push(b);
            code += 1;
            if code == 0xff {
                dst[ptr] = code;
                ptr = dst.len();
                dst.push(0);
                code = 1;
            }
        }

        dst[ptr] = code;

        return dst;
    }

    pub fn decode(src: &[u8]) -> Result<Vec<u8>, CorruptError> {
        let mut dst = Vec::<u8>::new();
        dst.reserve(src.len());

        let mut ptr = 0usize;

        while ptr < src.len() {
            let code = src[ptr];
            if ptr + (code as usize) > src.len() {
                return Err(CorruptError {});
            }

            ptr += 1;

            for _ in 1..code {
                dst.push(src[ptr]);
                ptr += 1;
            }

            if code < 0xff {
                dst.push(0);
            }
        }

        if dst.len() == 0 {
            return Ok(dst);
        }

        // trim phantom zero
        dst.pop();

        Ok(dst)
    }
}

pub struct ZPE {}

impl ZPE {
    pub fn encode(src: &[u8]) -> Vec<u8> {
        // guess at how much extra space we need
        let l = (src.len() as f64 * 1.045) as usize;

        let mut dst = Vec::<u8>::new();

        if src.len() == 0 {
            return dst;
        }

        dst.reserve(l);
        dst.push(0);

        let mut ptr = 0usize;
        let mut code = 1u8;

        let mut want_pair = false;

        for &b in src {
            if want_pair {
                want_pair = false;
                if b == 0 {
                    // assert code < 31
                    code |= 0xE0u8;
                    dst[ptr] = code;
                    ptr = dst.len();
                    dst.push(0);
                    code = 0x01;
                    continue;
                }

                // was looking for a pair of zeros but didn't find it -- encode as normal
                dst[ptr] = code;
                ptr = dst.len();
                dst.push(0);
                code = 0x01;
                dst.push(b);
                code += 1;
                continue;
            }

            if b == 0 {
                if code < 31 {
                    want_pair = true;
                    continue;
                }

                // too long to encode with ZPE -- encode as normal
                dst[ptr] = code;
                ptr = dst.len();
                dst.push(0);
                code = 0x01;
                continue;
            }

            dst.push(b);
            code += 1;

            if code == 0xE0 {
                dst[ptr] = code;
                ptr = dst.len();
                dst.push(0);
                code = 0x01;
            }
        }

        if want_pair {
            code = 0xE0 | code
        }

        dst[ptr] = code;
        return dst;
    }

    pub fn decode(src: &[u8]) -> Result<Vec<u8>, CorruptError> {
        let mut dst = Vec::<u8>::new();

        dst.reserve(src.len());

        let mut ptr = 0usize;

        while ptr < src.len() {
            let code = src[ptr];

            let l = if code > 0xE0 {
                (code & 0x1F) as usize
            } else {
                code as usize
            };

            if ptr + l > src.len() {
                return Err(CorruptError {});
            }

            ptr += 1;

            for _ in 1..l {
                dst.push(src[ptr]);
                ptr += 1;
            }

            match code {
                0x00..=0xDF => dst.push(0),
                0xE0 => { /* nothing */ }
                0xE1..=0xFF => {
                    dst.push(0);
                    dst.push(0);
                }
            }
        }

        if dst.len() == 0 {
            return Ok(dst);
        }

        // trim phantom zero
        dst.pop();

        Ok(dst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::Path;

    macro_rules! test_roundtrip {
        ($name:ident, $encoder:ident, $src:expr, $dst:expr) => {
            #[test]
            fn $name() {
                let got = $encoder::encode($src);
                assert_eq!(got, $dst);
                let orig = $encoder::decode(&got).unwrap();
                assert_eq!(orig, $src);
            }
        };
    }

    test_roundtrip!(test_only_zero, Encoder, &[0x00], &[0x01, 0x01]);
    test_roundtrip!(
        test_one_zero,
        Encoder,
        &[0x11, 0x22, 0x00, 0x33],
        &[0x03, 0x11, 0x22, 0x02, 0x33]
    );
    test_roundtrip!(
        test_multiple_zeros,
        Encoder,
        &[0x11, 0x00, 0x00, 0x00],
        &[0x02, 0x11, 0x01, 0x01, 0x01]
    );

    test_roundtrip!(
        test_zpe_basic,
        ZPE,
        &[0x45, 0x00, 0x00, 0x2C, 0x4C, 0x79, 0x00, 0x00, 0x40, 0x06, 0x4F, 0x37],
        &[0xE2, 0x45, 0xE4, 0x2C, 0x4C, 0x79, 0x05, 0x40, 0x06, 0x4F, 0x37]
    );

    test_roundtrip!(
        test_zpe_trailing,
        ZPE,
        &[0x11, 0x00, 0x00, 0x00],
        &[0xE2, 0x11, 0xE1]
    );

    test_roundtrip!(
        test_zpe_single,
        ZPE,
        &[0x11, 0x22, 0x00, 0x33],
        &[0x03, 0x11, 0x22, 0x02, 0x33]
    );

    #[test]
    fn test_roundtrip_big_nonzero() {
        let mut src = Vec::<u8>::new();
        src.resize(1024, 4);

        let got = Encoder::encode(&src);
        assert!(!got.contains(&0));
        let orig = Encoder::decode(&got).unwrap();
        assert_eq!(orig, src);
    }

    #[test]
    fn test_fuzzer_corpus() {
        let path = Path::new("fuzz/corpus/fuzz_roundtrip");
        let dir = fs::read_dir(path).unwrap();
        for entry in dir {
            let entry = entry.unwrap();
            let data = fs::read(entry.path()).unwrap();

            let zgot = ZPE::encode(&data);
            let zorig = ZPE::decode(&zgot).unwrap();
            assert_eq!(zorig, data);

            let got = Encoder::encode(&data);
            let orig = Encoder::decode(&got).unwrap();
            assert_eq!(orig, data);
        }
    }
}
