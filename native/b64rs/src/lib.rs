use base64_simd::URL_SAFE_NO_PAD;
use rustler::{Binary, Env, NewBinary, NifResult};

#[rustler::nif]
fn nif_decode<'a>(env: Env<'a>, input: Binary<'a>) -> NifResult<Binary<'a>> {
    let decoded = URL_SAFE_NO_PAD
        .decode_to_vec(input.as_slice())
        .map_err(|_| rustler::Error::BadArg)?;
    let mut out = NewBinary::new(env, decoded.len());
    out.as_mut_slice().copy_from_slice(&decoded);
    Ok(out.into())
}

#[rustler::nif]
fn nif_encode<'a>(env: Env<'a>, input: Binary<'a>) -> NifResult<Binary<'a>> {
    let encoded = URL_SAFE_NO_PAD.encode_to_string(input.as_slice());
    let mut out = NewBinary::new(env, encoded.len());
    out.as_mut_slice().copy_from_slice(encoded.as_bytes());
    Ok(out.into())
}

rustler::init!("b64rs");
