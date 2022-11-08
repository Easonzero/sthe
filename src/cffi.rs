use super::*;
use std::ffi::{c_char, CStr, CString};

macro_rules! throw {
    ($call: expr) => {
        match $call {
            Ok(ret) => ret,
            Err(err) => {
                return err;
            }
        }
    };
    ($call: expr, $code: ident) => {
        throw!($call.map_err(|_| RetCode::$code))
    };
}

#[repr(C)]
pub enum RetCode {
    Succ,
    InvalidArgs,
}

#[repr(C)]
pub enum DescpType {
    Json,
    Toml,
}

#[no_mangle]
pub unsafe extern "C" fn compile_opt(
    descp: *const c_char,
    ty: DescpType,
    out: &mut *const ExtractOptCompiled,
) -> RetCode {
    let descp = throw!(CStr::from_ptr(descp).to_str(), InvalidArgs);
    let opt: ExtractOpt = match ty {
        DescpType::Json => throw!(serde_json::from_str(descp), InvalidArgs),
        DescpType::Toml => throw!(toml::from_str(descp), InvalidArgs),
    };
    *out = Box::into_raw(Box::new(throw!(opt.compile(), InvalidArgs)));

    RetCode::Succ
}

#[no_mangle]
pub unsafe extern "C" fn release_opt(opt: *mut ExtractOptCompiled) {
    let _ = Box::from_raw(opt);
}

fn extract2c(extract: Extract, ty: DescpType) -> Result<*mut c_char, RetCode> {
    let mut buf = vec![];
    match ty {
        DescpType::Json => {
            serde_json::to_writer(&mut buf, &extract).map_err(|_| RetCode::InvalidArgs)?
        }
        DescpType::Toml => buf = toml::to_vec(&extract).map_err(|_| RetCode::InvalidArgs)?,
    };
    let c_extract = unsafe { CString::from_vec_unchecked(buf) };

    Ok(c_extract.into_raw())
}

#[no_mangle]
pub unsafe extern "C" fn extract_fragment(
    fragment: *const c_char,
    opt: *const ExtractOptCompiled,
    ty: DescpType,
    out: &mut *const c_char,
) -> RetCode {
    let opt = throw!(opt.as_ref().ok_or(RetCode::InvalidArgs));
    let fragment = throw!(CStr::from_ptr(fragment).to_str(), InvalidArgs);
    let extract = super::extract_fragment(fragment, opt);

    *out = throw!(extract2c(extract, ty));
    RetCode::Succ
}

#[no_mangle]
pub unsafe extern "C" fn extract_document(
    document: *const c_char,
    opt: *const ExtractOptCompiled,
    ty: DescpType,
    out: &mut *const c_char,
) -> RetCode {
    let opt = throw!(opt.as_ref().ok_or(RetCode::InvalidArgs));
    let document = throw!(CStr::from_ptr(document).to_str(), InvalidArgs);
    let extract = super::extract_document(document, opt);

    *out = throw!(extract2c(extract, ty));
    RetCode::Succ
}

#[no_mangle]
pub unsafe extern "C" fn release_extract(ret: *mut c_char) {
    let _ = CString::from_raw(ret);
}
