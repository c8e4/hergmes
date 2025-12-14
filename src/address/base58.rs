const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

const DECODE_MAP: [u8; 256] = {
    let mut map = [0xFFu8; 256];
    let mut i = 0;
    while i < 58 {
        map[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    map
};

#[derive(Debug)]
pub enum Base58Error {
    InvalidCharacter(char),
}

// Each byte expands to ~1.38 base58 chars
#[inline(always)]
fn max_encoded_len(bytes_len: usize) -> usize {
    bytes_len * 138 / 100 + 1
}

// Each base58 char decodes to ~0.73 bytes
#[inline(always)]
fn max_decoded_len(chars_len: usize) -> usize {
    chars_len * 733 / 1000 + 1
}

#[cold]
fn err_non_ascii(s: &str, byte_index: usize) -> Base58Error {
    let mut start = byte_index.min(s.len());
    while start > 0 && !s.is_char_boundary(start) {
        start -= 1;
    }
    let ch = s[start..].chars().next().unwrap_or('\u{FFFD}');
    Base58Error::InvalidCharacter(ch)
}

pub fn encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut zeros = 0usize;
    while zeros < data.len() && data[zeros] == 0 {
        zeros += 1;
    }

    let input = &data[zeros..];
    let mut buf = vec![0u8; max_encoded_len(input.len()) + zeros];

    let mut index = 0usize;
    for &val in input {
        let mut carry = val as usize;

        for byte in &mut buf[..index] {
            carry += (*byte as usize) << 8;
            *byte = (carry % 58) as u8;
            carry /= 58;
        }

        while carry > 0 {
            buf[index] = (carry % 58) as u8;
            index += 1;
            carry /= 58;
        }
    }

    for _ in 0..zeros {
        buf[index] = 0;
        index += 1;
    }

    for v in &mut buf[..index] {
        *v = ALPHABET[*v as usize];
    }

    buf[..index].reverse();
    buf.truncate(index);
    unsafe { String::from_utf8_unchecked(buf) }
}

pub fn decode(s: &str) -> Result<Vec<u8>, Base58Error> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    let bytes = s.as_bytes();

    let mut zeros = 0usize;
    while zeros < bytes.len() && bytes[zeros] == b'1' {
        zeros += 1;
    }

    let mut buf = vec![0u8; zeros + max_decoded_len(bytes.len() - zeros)];
    let mut index = 0usize;

    for (i, &c) in bytes.iter().enumerate().skip(zeros) {
        if c >= 0x80 {
            return Err(err_non_ascii(s, i));
        }

        let mut val = DECODE_MAP[c as usize] as usize;
        if val == 0xFF {
            return Err(Base58Error::InvalidCharacter(c as char));
        }

        for byte in &mut buf[..index] {
            val += (*byte as usize) * 58;
            *byte = (val & 0xFF) as u8;
            val >>= 8;
        }

        while val > 0 {
            buf[index] = (val & 0xFF) as u8;
            index += 1;
            val >>= 8;
        }
    }

    for _ in 0..zeros {
        buf[index] = 0;
        index += 1;
    }

    buf[..index].reverse();
    buf.truncate(index);
    Ok(buf)
}

/// Decode directly into caller's buffer. No validation - assumes valid base58 input.
#[inline]
pub fn decode_into(s: &str, out: &mut [u8]) -> usize {
    if s.is_empty() {
        return 0;
    }

    let bytes = s.as_bytes();

    let mut zeros = 0usize;
    while zeros < bytes.len() && bytes[zeros] == b'1' {
        zeros += 1;
    }

    decode_into_inner(bytes, zeros, out)
}

#[inline(always)]
fn decode_into_inner(bytes: &[u8], zeros: usize, buf: &mut [u8]) -> usize {
    let mut index = 0usize;

    for &c in &bytes[zeros..] {
        let mut val = DECODE_MAP[c as usize] as usize;

        for byte in &mut buf[..index] {
            val += (*byte as usize) * 58;
            *byte = (val & 0xFF) as u8;
            val >>= 8;
        }

        while val > 0 {
            buf[index] = (val & 0xFF) as u8;
            index += 1;
            val >>= 8;
        }
    }

    for _ in 0..zeros {
        buf[index] = 0;
        index += 1;
    }

    buf[..index].reverse();
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_encode_zeros() {
        assert_eq!(encode(&[0]), "1");
        assert_eq!(encode(&[0, 0]), "11");
        assert_eq!(encode(&[0, 0, 0]), "111");
    }

    #[test]
    fn test_decode_zeros() {
        assert_eq!(decode("1").unwrap(), vec![0]);
        assert_eq!(decode("11").unwrap(), vec![0, 0]);
        assert_eq!(decode("111").unwrap(), vec![0, 0, 0]);
    }

    #[test]
    fn test_roundtrip() {
        let test_cases: &[&[u8]] = &[
            &[0x00, 0x01, 0x02, 0x03],
            &[0xff, 0xfe, 0xfd],
            &[0x00, 0x00, 0x01, 0x02],
            b"Hello, World!",
            &[0x61],
            &[0x62, 0x62, 0x62],
        ];

        for data in test_cases {
            let encoded = encode(data);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(&decoded, data);
        }
    }

    #[test]
    fn test_known_vectors() {
        let vectors: &[(&[u8], &str)] = &[
            (b"Hello World!", "2NEpo7TZRRrLZSi2U"),
            (&[0x00, 0x00, 0x28, 0x7f, 0xb4, 0xcd], "11233QC4"),
            (
                &[
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                "11111111111111111111111111111111",
            ),
            (
                &hex::decode(
                    "000111d38e5fc9071ffcd20b4a763cc9ae4f252bb4e48fd66a835e252ada93ff480d6dd43dc62a641155a5",
                )
                .unwrap(),
                "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
            ),
        ];

        for (data, expected) in vectors {
            assert_eq!(encode(data), *expected);
            assert_eq!(decode(expected).unwrap(), *data);
        }
    }

    #[test]
    fn test_invalid_character() {
        assert!(decode("0").is_err());
        assert!(decode("O").is_err());
        assert!(decode("I").is_err());
        assert!(decode("l").is_err());
        assert!(decode("abc0def").is_err());
    }

    #[test]
    fn test_ergo_address_roundtrip() {
        let addr = "9fRusAarL1KkrWQVsxSRVYnvWxaAT2A96cKtNn9tvPh5XUyCisr";
        let decoded = decode(addr).unwrap();
        assert_eq!(encode(&decoded), addr);

        let addr = "8sZ2fVu5VUQKEmWt4xRRDBYzuw5aevhhziPBDGB";
        let decoded = decode(addr).unwrap();
        assert_eq!(encode(&decoded), addr);
    }

    #[test]
    fn test_decode_into() {
        let mut buf = [0u8; 64];

        let addr = "9fRusAarL1KkrWQVsxSRVYnvWxaAT2A96cKtNn9tvPh5XUyCisr";
        let len = decode_into(addr, &mut buf);
        assert_eq!(&buf[..len], decode(addr).unwrap().as_slice());

        let addr = "11233QC4";
        let len = decode_into(addr, &mut buf);
        assert_eq!(&buf[..len], decode(addr).unwrap().as_slice());

        assert_eq!(decode_into("", &mut buf), 0);
    }
}
