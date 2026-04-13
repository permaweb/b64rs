use base64_simd::{forgiving_decode_inplace, forgiving_decode_to_vec, AsOut, URL_SAFE_NO_PAD};
use memchr::{memchr, memchr2, memchr3};
use rustler::{Binary, Env, NewBinary, NifResult};

enum DecodeMode {
    UrlSafeDirect,
    ForgivingStandard,
    RewriteUrlSafeForgiving,
}

fn classify_decode(input: &[u8]) -> DecodeMode {
    let has_urlsafe = memchr2(b'-', b'_', input).is_some();
    if !has_urlsafe {
        return DecodeMode::ForgivingStandard;
    }

    let has_standard = memchr2(b'+', b'/', input).is_some();
    let has_padding = memchr(b'=', input).is_some();
    let has_whitespace =
        memchr3(b' ', b'\n', b'\r', input).is_some() || memchr3(b'\t', 0x0b, 0x0c, input).is_some();

    if !has_standard && !has_padding && !has_whitespace {
        DecodeMode::UrlSafeDirect
    } else {
        DecodeMode::RewriteUrlSafeForgiving
    }
}

fn rewrite_urlsafe_to_standard_inplace(data: &mut [u8]) {
    for byte in data {
        if *byte == b'-' {
            *byte = b'+';
        } else if *byte == b'_' {
            *byte = b'/';
        }
    }
}

fn decode_via_rewrite_forgiving(input: &[u8]) -> Result<Vec<u8>, base64_simd::Error> {
    let mut normalized = input.to_vec();
    rewrite_urlsafe_to_standard_inplace(&mut normalized);
    let decoded = forgiving_decode_inplace(&mut normalized)?;
    Ok(decoded.to_vec())
}

pub fn decode_bytes(input: &[u8]) -> Result<Vec<u8>, base64_simd::Error> {
    match classify_decode(input) {
        DecodeMode::UrlSafeDirect => {
            let mut out = vec![0; URL_SAFE_NO_PAD.estimated_decoded_length(input.len())];
            match URL_SAFE_NO_PAD.decode(input, out.as_mut_slice().as_out()) {
                Ok(decoded) => {
                    let decoded_len = decoded.len();
                    out.truncate(decoded_len);
                    Ok(out)
                }
                // Recover from strict URL-safe rejection (e.g. non-zero trailing bits)
                // by running the forgiving path used for standard-compatible inputs.
                Err(_) => decode_via_rewrite_forgiving(input),
            }
        }
        DecodeMode::ForgivingStandard => forgiving_decode_to_vec(input),
        DecodeMode::RewriteUrlSafeForgiving => decode_via_rewrite_forgiving(input),
    }
}

pub fn encode_bytes(input: &[u8]) -> Vec<u8> {
    let mut out = vec![0; URL_SAFE_NO_PAD.encoded_length(input.len())];
    let _ = URL_SAFE_NO_PAD.encode(input, out.as_mut_slice().as_out());
    out
}

#[rustler::nif]
fn nif_decode<'a>(env: Env<'a>, input: Binary<'a>) -> NifResult<Binary<'a>> {
    let decoded = decode_bytes(input.as_slice()).map_err(|_| rustler::Error::BadArg)?;
    let mut out = NewBinary::new(env, decoded.len());
    out.as_mut_slice().copy_from_slice(&decoded);
    Ok(out.into())
}

#[rustler::nif]
fn nif_encode<'a>(env: Env<'a>, input: Binary<'a>) -> NifResult<Binary<'a>> {
    let mut out = NewBinary::new(env, URL_SAFE_NO_PAD.encoded_length(input.len()));
    let _ = URL_SAFE_NO_PAD.encode(input.as_slice(), out.as_mut_slice().as_out());
    Ok(out.into())
}

rustler::init!("b64rs");

#[cfg(test)]
mod tests {
    use super::{decode_bytes, decode_via_rewrite_forgiving, encode_bytes};
    use proptest::prelude::*;

    const URLSAFE_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    fn b64url_value(byte: u8) -> Option<u8> {
        URLSAFE_ALPHABET
            .iter()
            .position(|&x| x == byte)
            .map(|idx| idx as u8)
    }

    fn mutate_trailing_bits(mut encoded: Vec<u8>, low_bits: u8) -> Vec<u8> {
        if encoded.is_empty() {
            return encoded;
        }

        let mask = match encoded.len() % 4 {
            2 => 0x0f,
            3 => 0x03,
            _ => return encoded,
        };

        let idx = encoded.len() - 1;
        if let Some(value) = b64url_value(encoded[idx]) {
            let mutated = (value & !mask) | (low_bits & mask);
            encoded[idx] = URLSAFE_ALPHABET[usize::from(mutated)];
        }

        encoded
    }

    fn maybe_insert_whitespace(mut encoded: Vec<u8>, insert: bool, kind: u8, pos_hint: usize) -> Vec<u8> {
        if !insert {
            return encoded;
        }
        let whitespace = [b' ', b'\n', b'\r', b'\t'][usize::from(kind % 4)];
        let pos = pos_hint % (encoded.len() + 1);
        encoded.insert(pos, whitespace);
        encoded
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn decode_mutated_urlsafe_matches_canonical(
            payload in proptest::collection::vec(any::<u8>(), 0..4096),
            low_bits in any::<u8>(),
            insert_ws in any::<bool>(),
            ws_kind in any::<u8>(),
            ws_pos in 0usize..4096,
        ) {
            let encoded = encode_bytes(&payload);
            let mutated = mutate_trailing_bits(encoded, low_bits);
            let mutated = maybe_insert_whitespace(mutated, insert_ws, ws_kind, ws_pos);

            let expected = decode_via_rewrite_forgiving(&mutated);
            let actual = decode_bytes(&mutated);

            match (actual, expected) {
                (Ok(actual), Ok(expected)) => prop_assert_eq!(actual, expected),
                (Err(_), Err(_)) => {}
                (actual, expected) => {
                    prop_assert!(false, "decode mismatch: actual={actual:?}, expected={expected:?}");
                }
            }
        }
    }
}
