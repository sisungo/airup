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

/// Indicates the operation failed because [airup_sdk_init] was already called previously.
pub const AIRUP_SDK_INITIALIZED: airup_sdk_result_t = airup_sdk_result_t(2);

/// Indicates the operation failed because [airup_sdk_init] was not called previously.
pub const AIRUP_SDK_NOT_INITIALIZED: airup_sdk_result_t = airup_sdk_result_t(2);
