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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_roundtrip {
        ($name:ident, $src:expr, $dst:expr) => {
            #[test]
            fn $name() {
                let got = Encoder::encode($src);
                assert_eq!(got, $dst);
                let orig = Encoder::decode(&got).unwrap();
                assert_eq!(orig, $src);
            }
        };
    }

    test_roundtrip!(test_only_zero, &[0x00], &[0x01, 0x01]);
    test_roundtrip!(
        test_one_zero,
        &[0x11, 0x22, 0x00, 0x33],
        &[0x03, 0x11, 0x22, 0x02, 0x33]
    );
    test_roundtrip!(
        test_multiple_zeros,
        &[0x11, 0x00, 0x00, 0x00],
        &[0x02, 0x11, 0x01, 0x01, 0x01]
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
}
