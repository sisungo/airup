//! # Airup SDK - FFI
//! The module contains source code of the Airup SDK for C/C++.

#![allow(nonstandard_style)]

/// A result type in Airup SDK.
#[repr(transparent)]
pub struct airup_sdk_result_t(u32);

/// Indicates the operation completed successfully.
pub const AIRUP_SDK_SUCCESS: airup_sdk_result_t = airup_sdk_result_t(0);

/// Indicates the operation failed because an OS error.
pub const AIRUP_SDK_OS_ERROR: airup_sdk_result_t = airup_sdk_result_t(1);

/// Creates a new Airup SDK context.
#[no_mangle]
pub extern "C" fn AirupSDK_CreateContext(
    out: &mut Box<tokio::runtime::Runtime>,
) -> airup_sdk_result_t {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    match runtime {
        Ok(rt) => {
            *out = Box::new(rt);
            AIRUP_SDK_SUCCESS
        }
        Err(_) => AIRUP_SDK_OS_ERROR,
    }
}

/// Deletes an Airup SDK context.
#[no_mangle]
pub extern "C" fn AirupSDK_DestroyContext(_: Box<tokio::runtime::Runtime>) {}
