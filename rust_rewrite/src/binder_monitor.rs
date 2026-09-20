use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_void};
use std::io::Write;
use std::process::{Command, Stdio};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::binder_ndk::*;

pub struct ProcessObserverCallbacks {
    pub on_foreground_activities_changed: Option<Box<dyn Fn(i32, i32, bool) + Send + Sync>>,
    pub on_process_died: Option<Box<dyn Fn(i32, i32) + Send + Sync>>,
}

type DisplayStateCallback = Box<dyn Fn(bool) + Send + Sync>;
type PowerSaveCallback = Box<dyn Fn(bool) + Send + Sync>;

#[repr(u32)]
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum TxCode {
    OnProcessStarted = 1,
    IsPowerSaveMode = 2,
    OnDisplayEvent = 3,
    GetPackageNameForUid = 4,
}

pub struct BinderMonitorState {
    pub power_binder: *mut AIBinder,
    pub notification_binder: *mut AIBinder,
    pub activity_binder: *mut AIBinder,
    pub display_binder: *mut AIBinder,
    pub package_binder: *mut AIBinder,

    pub process_observer_binder: *mut AIBinder,
    pub display_callback_binder: *mut AIBinder,

    pub process_callbacks: ProcessObserverCallbacks,
    pub display_callback: Option<DisplayStateCallback>,
    pub power_save_callback: Option<PowerSaveCallback>,

    pub display_last_state: bool,
    pub power_save_last_state: bool,
    pub stop_power_save_polling: bool,

    pub tx_codes: HashMap<TxCode, u32>,
}

unsafe impl Send for BinderMonitorState {}
unsafe impl Sync for BinderMonitorState {}

lazy_static::lazy_static! {
    pub static ref BINDER_MONITOR_STATE: Mutex<BinderMonitorState> = Mutex::new(BinderMonitorState {
        power_binder: ptr::null_mut(),
        notification_binder: ptr::null_mut(),
        activity_binder: ptr::null_mut(),
        display_binder: ptr::null_mut(),
        package_binder: ptr::null_mut(),
        process_observer_binder: ptr::null_mut(),
        display_callback_binder: ptr::null_mut(),
        process_callbacks: ProcessObserverCallbacks {
            on_foreground_activities_changed: None,
            on_process_died: None,
        },
        display_callback: None,
        power_save_callback: None,
        display_last_state: false,
        power_save_last_state: false,
        stop_power_save_polling: false,
        tx_codes: HashMap::new(),
    });
}

pub fn initialize() -> bool {
    let mut tx_codes = HashMap::new();

    let child = Command::new("app_process")
        .arg("-cp")
        .arg(crate::constants::MODPATH.to_owned() + "/binder_resolver.apk")
        .arg("/system/bin")
        .arg("dev.rem01gaming.encore.binderresolver.Main")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    if let Ok(mut child_proc) = child {
        let input = "android.content.pm.IPackageManager.Stub::TRANSACTION_getNameForUid\n";
        if let Some(mut stdin) = child_proc.stdin.take() {
            let _ = stdin.write_all(input.as_bytes());
        }

        if let Ok(output) = child_proc.wait_with_output() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    if let Ok(code) = parts[1].parse::<u32>() {
                        if parts[0]
                            == "android.content.pm.IPackageManager.Stub::TRANSACTION_getNameForUid"
                        {
                            tx_codes.insert(TxCode::GetPackageNameForUid, code);
                        }
                    }
                }
            }
        }
    }

    unsafe {
        let power_name = CString::new("power").unwrap();
        let notif_name = CString::new("notification").unwrap();
        let activity_name = CString::new("activity").unwrap();
        let display_name = CString::new("display").unwrap();
        let package_name = CString::new("package").unwrap();

        let power_binder = AServiceManager_getService(power_name.as_ptr());
        let notif_binder = AServiceManager_getService(notif_name.as_ptr());
        let activity_binder = AServiceManager_getService(activity_name.as_ptr());
        let display_binder = AServiceManager_getService(display_name.as_ptr());
        let package_binder = AServiceManager_getService(package_name.as_ptr());

        let mut state = BINDER_MONITOR_STATE.lock().unwrap();
        state.power_binder = power_binder;
        state.notification_binder = notif_binder;
        state.activity_binder = activity_binder;
        state.display_binder = display_binder;
        state.package_binder = package_binder;
        state.tx_codes = tx_codes;

        ABinderProcess_startThreadPool();
    }
    true
}

pub fn get_package_name_for_uid(uid: i32) -> String {
    let output = Command::new("cmd")
        .args(["package", "list", "packages", "--uid", &uid.to_string()])
        .output();

    if let Ok(out) = output {
        let out_str = String::from_utf8_lossy(&out.stdout);
        for line in out_str.lines() {
            if line.contains(&format!("uid:{}", uid)) {
                if let Some(pkg) = line.split("uid:").next() {
                    let clean_pkg = pkg.replace("package:", "").trim().to_string();
                    return clean_pkg;
                }
            }
        }
    }
    String::new()
}

pub fn set_process_observer_callbacks(callbacks: ProcessObserverCallbacks) {
    let mut state = BINDER_MONITOR_STATE.lock().unwrap();
    state.process_callbacks = callbacks;
}

pub fn set_display_state_callback(callback: DisplayStateCallback) {
    let mut state = BINDER_MONITOR_STATE.lock().unwrap();
    state.display_callback = Some(callback);
}

pub fn join_thread_pool() {
    unsafe {
        ABinderProcess_joinThreadPool();
    }
}
