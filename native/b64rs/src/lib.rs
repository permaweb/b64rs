use base64_simd::{AsOut, URL_SAFE_NO_PAD};
use rustler::{Binary, Env, NewBinary, NifResult};

pub fn decode_bytes(input: &[u8]) -> Result<Vec<u8>, base64_simd::Error> {
    let mut out = vec![0; URL_SAFE_NO_PAD.decoded_length(input)?];
    let decoded_len = {
        let decoded = URL_SAFE_NO_PAD.decode(input, out.as_mut_slice().as_out())?;
        decoded.len()
    };
    out.truncate(decoded_len);
    Ok(out)
}

pub fn encode_bytes(input: &[u8]) -> Vec<u8> {
    let mut out = vec![0; URL_SAFE_NO_PAD.encoded_length(input.len())];
    let _ = URL_SAFE_NO_PAD.encode(input, out.as_mut_slice().as_out());
    out
}

#[rustler::nif(schedule = "DirtyCpu")]
fn nif_decode<'a>(env: Env<'a>, input: Binary<'a>) -> NifResult<Binary<'a>> {
    let decoded_len = URL_SAFE_NO_PAD
        .decoded_length(input.as_slice())
        .map_err(|_| rustler::Error::BadArg)?;
    let mut out = NewBinary::new(env, decoded_len);
    let decoded = URL_SAFE_NO_PAD
        .decode(input.as_slice(), out.as_mut_slice().as_out())
        .map_err(|_| rustler::Error::BadArg)?;
    debug_assert_eq!(decoded.len(), decoded_len);
    Ok(out.into())
}

#[rustler::nif(schedule = "DirtyCpu")]
fn nif_encode<'a>(env: Env<'a>, input: Binary<'a>) -> NifResult<Binary<'a>> {
    let mut out = NewBinary::new(env, URL_SAFE_NO_PAD.encoded_length(input.len()));
    let _ = URL_SAFE_NO_PAD.encode(input.as_slice(), out.as_mut_slice().as_out());
    Ok(out.into())
}

rustler::init!("b64rs");
