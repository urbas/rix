#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HashType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, PartialEq)]
pub struct Hash {
    pub hash_type: HashType,
    pub bytes: Vec<u8>,
}

impl HashType {
    pub fn size(&self) -> usize {
        match self {
            HashType::Md5 => 16,
            HashType::Sha1 => 20,
            HashType::Sha256 => 32,
            HashType::Sha512 => 64,
        }
    }

    pub fn from_str(hash_type: &str) -> Option<HashType> {
        match hash_type {
            "md5" => Some(HashType::Md5),
            "sha1" => Some(HashType::Sha1),
            "sha256" => Some(HashType::Sha256),
            "sha512" => Some(HashType::Sha512),
            _ => None,
        }
    }
}

pub fn parse(hash_str: &str, hash_type: HashType) -> Result<Hash, String> {
    let hash_str_len = hash_str.as_bytes().len();
    let hash_size = hash_type.size();
    let mut bytes = Vec::new();
    bytes.resize(hash_size, 0);
    if hash_str_len == 2 * hash_size {
        from_base16(hash_str, hash_type)
    } else if hash_str_len == to_base32_len(hash_size) {
        from_base32(hash_str, hash_type)
    } else if hash_str_len == to_base64_len(hash_size) {
        from_base64(hash_str, hash_type)
    } else {
        Err(format!("hash '{}' with unexpected length.", hash_str))
    }
}

pub fn sri_hash_components<'a>(hash_str: &'a str) -> Result<(&'a str, &'a str), String> {
    hash_str
        .split_once('-')
        .or_else(|| hash_str.split_once(':'))
        .ok_or(format!("Failed to parse '{}'. Not an SRI hash.", hash_str))
}

pub fn to_base16(hash: &Hash) -> String {
    let bytes = &hash.bytes;
    let mut out_string = String::with_capacity(2 * bytes.len());
    for i in 0..bytes.len() {
        out_string.push(nibble_to_base16(bytes[i] >> 4));
        out_string.push(nibble_to_base16(bytes[i] & 0x0f));
    }
    return out_string;
}

pub fn from_base16(base16_str: &str, hash_type: HashType) -> Result<Hash, String> {
    let base16_str_bytes = base16_str.as_bytes();
    let mut bytes = vec![0; hash_type.size()];
    for idx in 0..bytes.len() {
        bytes[idx] = parse_base16_digit(base16_str_bytes[idx * 2])? << 4
            | parse_base16_digit(base16_str_bytes[idx * 2 + 1])?;
    }
    return Ok(Hash { hash_type, bytes });
}

pub fn to_base32(hash: &Hash) -> String {
    let bytes = &hash.bytes;
    let bytes_len = bytes.len();
    let len = to_base32_len(bytes_len);
    let mut out_string = String::with_capacity(len);

    for idx in (0..len).rev() {
        let b = idx * 5;
        let i = b / 8;
        let j = b % 8;
        let carry = if i >= bytes_len - 1 {
            0
        } else {
            bytes[i + 1].checked_shl(8 - j as u32).unwrap_or(0)
        };
        let c = (bytes[i] >> j) | carry;
        out_string.push(nibble_to_base32(c & 0x1f));
    }

    return out_string;
}

pub fn from_base32(base32_str: &str, hash_type: HashType) -> Result<Hash, String> {
    let mut bytes = vec![0; hash_type.size()];
    let base32_str_bytes = base32_str.as_bytes();
    let str_len = base32_str_bytes.len();
    for idx in 0..to_base32_len(bytes.len()) {
        let digit = parse_base32_digit(base32_str_bytes[str_len - idx - 1])?;
        let b = idx * 5;
        let i = b / 8;
        let j = b % 8;
        bytes[i] |= digit << j;

        let carry = digit.checked_shr(8 - j as u32).unwrap_or(0);
        if i < bytes.len() - 1 {
            bytes[i + 1] |= carry;
        } else if carry != 0 {
            return Err(format!("Invalid base-32 string '{}'", base32_str));
        }
    }
    return Ok(Hash { hash_type, bytes });
}

pub fn to_base64(hash: &Hash) -> String {
    let bytes = &hash.bytes;
    let mut out_string = String::with_capacity(to_base64_len(bytes.len()));
    let mut data: usize = 0;
    let mut nbits: usize = 0;

    for byte in bytes {
        data = data << 8 | (*byte as usize);
        nbits += 8;
        while nbits >= 6 {
            nbits -= 6;
            out_string.push(BASE_64_CHARS[data >> nbits & 0x3f] as char);
        }
    }

    if nbits > 0 {
        out_string.push(BASE_64_CHARS[data << (6 - nbits) & 0x3f] as char);
    }

    while out_string.len() % 4 > 0 {
        out_string.push('=');
    }

    return out_string;
}

pub fn from_base64(base64_str: &str, hash_type: HashType) -> Result<Hash, String> {
    let mut bytes = vec![0; hash_type.size()];
    let base64_str_bytes = base64_str.as_bytes();
    let mut d: u32 = 0;
    let mut bits: u32 = 0;
    let mut byte = 0;

    for chr in base64_str_bytes {
        if *chr == b'=' {
            break;
        }
        let digit = BASE_64_CHAR_VALUES[*chr as usize];
        if digit == INVALID_CHAR_VALUE {
            return Err(format!(
                "Character '{}' is not a valid base-64 character.",
                *chr as char
            ));
        }
        bits += 6;
        d = d << 6 | digit as u32;
        if bits >= 8 {
            bytes[byte] = (d >> (bits - 8) & 0xff) as u8;
            bits -= 8;
            byte += 1;
        }
    }
    return Ok(Hash { hash_type, bytes });
}

fn nibble_to_base16(nibble: u8) -> char {
    if nibble < 10 {
        return (b'0' + nibble) as char;
    }
    return (b'a' + nibble - 10) as char;
}

fn parse_base16_digit(chr: u8) -> Result<u8, String> {
    match chr {
        b'0'..=b'9' => Ok(chr - b'0'),
        b'A'..=b'F' => Ok(chr - b'A' + 10),
        b'a'..=b'f' => Ok(chr - b'a' + 10),
        _ => Err("Not a hex numeral.".to_owned()),
    }
}

fn to_base32_len(bytes_count: usize) -> usize {
    (bytes_count * 8 - 1) / 5 + 1
}

fn nibble_to_base32(nibble: u8) -> char {
    if nibble < 10 {
        return (b'0' + nibble) as char;
    } else if nibble < 14 {
        return (b'a' + nibble - 10) as char;
    } else if nibble < 23 {
        return (b'f' + nibble - 14) as char;
    } else if nibble < 27 {
        return (b'p' + nibble - 23) as char;
    }
    return (b'v' + nibble - 27) as char;
}

fn parse_base32_digit(chr: u8) -> Result<u8, String> {
    match chr {
        b'0'..=b'9' => Ok(chr - b'0'),
        b'a'..=b'd' => Ok(chr - b'a' + 10),
        b'f'..=b'n' => Ok(chr - b'f' + 14),
        b'p'..=b's' => Ok(chr - b'p' + 23),
        b'v'..=b'z' => Ok(chr - b'v' + 27),
        _ => {
            return Err(format!(
                "Character '{}' is not a valid base-32 character.",
                chr as char
            ))
        }
    }
}

fn to_base64_len(bytes_count: usize) -> usize {
    ((4 * bytes_count / 3) + 3) & !3
}

const BASE_64_CHARS: &[u8] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".as_bytes();
const BASE_64_CHAR_VALUES: [u8; 256] = compute_base64_char_values();
const INVALID_CHAR_VALUE: u8 = 255;

const fn compute_base64_char_values() -> [u8; 256] {
    let mut char_values: [u8; 256] = [INVALID_CHAR_VALUE; 256];
    let mut idx = 0;
    while idx < 64 {
        char_values[BASE_64_CHARS[idx] as usize] = idx as u8;
        idx += 1;
    }
    return char_values;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sha256_sample() -> Hash {
        Hash {
            hash_type: HashType::Sha256,
            bytes: vec![
                0xd5, 0x31, 0x38, 0x62, 0x85, 0x6f, 0x77, 0x70, 0xbd, 0xff, 0xed, 0x2d, 0xfe, 0x8c,
                0x41, 0x7a, 0x84, 0xf3, 0xf6, 0xd5, 0xe1, 0x1c, 0x3b, 0x5c, 0x19, 0x42, 0x0f, 0x21,
                0x30, 0x76, 0x6f, 0x81,
            ],
        }
    }

    fn sha512_sample() -> Hash {
        Hash {
            hash_type: HashType::Sha512,
            bytes: vec![
                0xfb, 0x2e, 0x19, 0x9d, 0xe3, 0xe9, 0xbd, 0x6b, 0x35, 0x7d, 0xcf, 0xcb, 0x85, 0x94,
                0x53, 0x1e, 0x44, 0xde, 0xb1, 0xb5, 0xe4, 0xc8, 0x16, 0x2e, 0x38, 0x1f, 0xb9, 0x0b,
                0x2a, 0x1d, 0x66, 0xaa, 0xc4, 0xb8, 0x44, 0xd7, 0x8b, 0x7c, 0xce, 0x55, 0xfa, 0x40,
                0x40, 0x87, 0x60, 0x0b, 0x79, 0x57, 0x6c, 0x72, 0xd3, 0x0c, 0x6f, 0x5d, 0x42, 0x8b,
                0x31, 0x47, 0xd0, 0x61, 0xbc, 0xb2, 0x83, 0x2d,
            ],
        }
    }

    #[test]
    fn test_hash_type_size() {
        assert_eq!(HashType::Md5.size(), 16);
        assert_eq!(HashType::Sha1.size(), 20);
        assert_eq!(HashType::Sha256.size(), 32);
        assert_eq!(HashType::Sha512.size(), 64);
    }

    #[test]
    fn test_hash_type_from_str() {
        assert_eq!(HashType::from_str("md5"), Some(HashType::Md5));
        assert_eq!(HashType::from_str("sha1"), Some(HashType::Sha1));
        assert_eq!(HashType::from_str("sha256"), Some(HashType::Sha256));
        assert_eq!(HashType::from_str("sha512"), Some(HashType::Sha512));
        assert_eq!(HashType::from_str("foobar"), None);
    }

    #[test]
    fn test_parse_sha256_base16() {
        assert_eq!(
            parse(
                "d5313862856f7770bdffed2dfe8c417a84f3f6d5e11c3b5c19420f2130766f81",
                HashType::Sha256,
            ),
            Ok(sha256_sample()),
        );
    }

    #[test]
    fn test_parse_sha256_base32() {
        assert_eq!(
            parse(
                "10bgfqq223s235f3n771spvg713s866gwbgdzyyp0xvghmi3hcfm",
                HashType::Sha256,
            ),
            Ok(sha256_sample()),
        );
    }

    #[test]
    fn test_parse_sha256_base64() {
        assert_eq!(
            parse(
                "1TE4YoVvd3C9/+0t/oxBeoTz9tXhHDtcGUIPITB2b4E=",
                HashType::Sha256,
            ),
            Ok(sha256_sample()),
        );
    }

    #[test]
    fn test_parse_sha256_invalid() {
        assert_eq!(
            parse("foobar", HashType::Sha256),
            Err("hash 'foobar' with unexpected length.".to_owned()),
        );
    }

    #[test]
    fn test_parse_sha512_base64() {
        assert_eq!(
            parse("+y4ZnePpvWs1fc/LhZRTHkTesbXkyBYuOB+5CyodZqrEuETXi3zOVfpAQIdgC3lXbHLTDG9dQosxR9BhvLKDLQ==", HashType::Sha512),
            Ok(sha512_sample()),
        );
    }

    #[test]
    fn test_to_base16() {
        assert_eq!(
            to_base16(&sha256_sample()),
            "d5313862856f7770bdffed2dfe8c417a84f3f6d5e11c3b5c19420f2130766f81"
        );
    }

    #[test]
    fn test_to_base3() {
        assert_eq!(
            to_base32(&sha256_sample()),
            "10bgfqq223s235f3n771spvg713s866gwbgdzyyp0xvghmi3hcfm"
        );
    }

    #[test]
    fn test_to_base64() {
        assert_eq!(
            to_base64(&sha256_sample()),
            "1TE4YoVvd3C9/+0t/oxBeoTz9tXhHDtcGUIPITB2b4E="
        );
    }

    #[test]
    fn test_from_base32_invalid_char() {
        assert_eq!(
            from_base32(")", HashType::Sha256),
            Err("Character ')' is not a valid base-32 character.".to_owned()),
        );
    }

    #[test]
    fn test_from_base64_invalid_char() {
        assert_eq!(
            from_base64(")", HashType::Sha256),
            Err("Character ')' is not a valid base-64 character.".to_owned()),
        );
    }

    #[test]
    fn test_sri_hash_components() {
        assert_eq!(sri_hash_components("md5-foobar"), Ok(("md5", "foobar")));
        assert_eq!(sri_hash_components("sha256:abc"), Ok(("sha256", "abc")),);
    }

    #[test]
    fn test_sri_hash_components_fail() {
        assert_eq!(
            sri_hash_components("md5foobar"),
            Err("Failed to parse 'md5foobar'. Not an SRI hash.".to_owned())
        );
    }
}
