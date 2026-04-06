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

pub fn decode_bytes(input: &[u8]) -> Result<Vec<u8>, base64_simd::Error> {
    match classify_decode(input) {
        DecodeMode::UrlSafeDirect => {
            let mut out = vec![0; URL_SAFE_NO_PAD.estimated_decoded_length(input.len())];
            let decoded_len = {
                let decoded = URL_SAFE_NO_PAD.decode(input, out.as_mut_slice().as_out())?;
                decoded.len()
            };
            out.truncate(decoded_len);
            Ok(out)
        }
        DecodeMode::ForgivingStandard => forgiving_decode_to_vec(input),
        DecodeMode::RewriteUrlSafeForgiving => {
            let mut normalized = input.to_vec();
            for byte in &mut normalized {
                if *byte == b'-' {
                    *byte = b'+';
                } else if *byte == b'_' {
                    *byte = b'/';
                }
            }
            let decoded = forgiving_decode_inplace(&mut normalized)?;
            Ok(decoded.to_vec())
        }
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
