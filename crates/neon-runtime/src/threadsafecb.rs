//! Facilities for running background tasks in the libuv thread pool.

use raw::Local;
use std::os::raw::c_void;

extern "C" {

    /// Schedules a background task.
    #[link_name = "Neon_ThreadSafeCb_New"]
    pub fn new(this: Local, callback: Local) -> *mut c_void;

    /// Schedules a background task.
    /// self.0, arg_cb_raw, completion_cb_raw, perform_arg_cb
    #[link_name = "Neon_ThreadSafeCb_Call"]
    pub fn call(thread_safe_cb: *mut c_void, arg_cb_raw: *mut c_void, completion_cb_raw: *mut c_void,
                complete: unsafe extern fn(Local, Local, *mut c_void, *mut c_void));

    #[link_name = "Neon_ThreadSafeCb_Delete"]
    pub fn delete(thread_safe_cb: *mut c_void);

}
