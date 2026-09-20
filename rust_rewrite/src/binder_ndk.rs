#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::{c_char, c_void};

pub type binder_status_t = i32;
pub type binder_flags_t = u32;
pub type binder_exception_t = i32;
pub type transaction_code_t = u32;

pub const STATUS_OK: binder_status_t = 0;
pub const STATUS_UNKNOWN_ERROR: binder_status_t = -2147483648; // 0x80000000

#[repr(C)]
pub struct AIBinder {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AIBinder_Class {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AParcel {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AStatus {
    _private: [u8; 0],
}

pub type AIBinder_Class_onCreate = Option<unsafe extern "C" fn(args: *mut c_void) -> *mut c_void>;
pub type AIBinder_Class_onDestroy = Option<unsafe extern "C" fn(userData: *mut c_void)>;
pub type AIBinder_Class_onTransact = Option<
    unsafe extern "C" fn(
        binder: *mut AIBinder,
        code: transaction_code_t,
        in_parcel: *const AParcel,
        out_parcel: *mut AParcel,
    ) -> binder_status_t,
>;

pub type AParcel_stringAllocator = Option<
    unsafe extern "C" fn(stringData: *mut c_void, length: i32, buffer: *mut *mut c_char) -> bool,
>;

unsafe extern "C" {
    pub fn AIBinder_new(clazz: *const AIBinder_Class, args: *mut c_void) -> *mut AIBinder;
    pub fn AIBinder_transact(
        binder: *mut AIBinder,
        code: transaction_code_t,
        in_parcel: *mut *mut AParcel,
        out_parcel: *mut *mut AParcel,
        flags: binder_flags_t,
    ) -> binder_status_t;
    pub fn AIBinder_prepareTransaction(
        binder: *mut AIBinder,
        in_parcel: *mut *mut AParcel,
    ) -> binder_status_t;
    pub fn AIBinder_Class_define(
        interfaceDescriptor: *const c_char,
        onCreate: AIBinder_Class_onCreate,
        onDestroy: AIBinder_Class_onDestroy,
        onTransact: AIBinder_Class_onTransact,
    ) -> *mut AIBinder_Class;
    pub fn AIBinder_associateClass(binder: *mut AIBinder, clazz: *const AIBinder_Class) -> bool;

    pub fn AServiceManager_getService(instance: *const c_char) -> *mut AIBinder;

    pub fn AParcel_writeInt32(parcel: *mut AParcel, value: i32) -> binder_status_t;
    pub fn AParcel_readInt32(parcel: *const AParcel, value: *mut i32) -> binder_status_t;

    pub fn AParcel_readString(
        parcel: *const AParcel,
        stringData: *mut c_void,
        allocator: AParcel_stringAllocator,
    ) -> binder_status_t;
    pub fn AParcel_writeString(
        parcel: *mut AParcel,
        string: *const c_char,
        length: i32,
    ) -> binder_status_t;

    pub fn AParcel_writeInterfaceToken(
        parcel: *mut AParcel,
        interface: *const c_char,
    ) -> binder_status_t;
    pub fn AParcel_writeStrongBinder(
        parcel: *mut AParcel,
        binder: *mut AIBinder,
    ) -> binder_status_t;
    pub fn AParcel_readStrongBinder(
        parcel: *const AParcel,
        binder: *mut *mut AIBinder,
    ) -> binder_status_t;

    pub fn AParcel_delete(parcel: *mut AParcel);

    pub fn ABinderProcess_startThreadPool();
    pub fn ABinderProcess_joinThreadPool();
}
