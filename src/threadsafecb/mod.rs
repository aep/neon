//! Asynchronous background _tasks_ that run in the Node thread pool.

use std::mem;
use std::os::raw::c_void;

use types::*;
use handle::{Handle, Managed};
use context::*;
use neon_runtime;
use neon_runtime::raw;
use std::sync::Arc;


struct ThreadSafeCBInner(*mut c_void);

unsafe impl Send for ThreadSafeCBInner {}
unsafe impl Sync for ThreadSafeCBInner {}

impl Drop for ThreadSafeCBInner {
    fn drop(&mut self) {
        unsafe {
            neon_runtime::threadsafecb::delete(self.0);
        }
    }
}

#[derive(Clone)]
pub struct ThreadSafeCB(Arc<ThreadSafeCBInner>);

type ArgCb = fn(&mut TaskContext, Handle<JsValue>, Handle<JsFunction>);

impl ThreadSafeCB {
    pub fn new<T: Value>(this: Handle<T>, callback: Handle<JsFunction>) -> Self {
        let cb = unsafe {
            neon_runtime::threadsafecb::new(this.to_raw(), callback.to_raw())
        };
        ThreadSafeCB(Arc::new(ThreadSafeCBInner(cb)))
    }

    pub fn call(&self, arg_cb: ArgCb) {
        let arg_cb_raw = Box::into_raw(Box::new(arg_cb)) as *mut c_void;
        unsafe {
            neon_runtime::threadsafecb::call((*self.0).0, arg_cb_raw, perform_arg_cb);
        }
    }
}

unsafe extern "C" fn perform_arg_cb(this: raw::Local, callback: raw::Local, arg_cb: *mut c_void) {
    TaskContext::with(|mut cx: TaskContext| {
        let this = JsValue::new_internal(this);
        let callback: Handle<JsFunction> = Handle::new_internal(JsFunction::from_raw(callback));
        let cb: Box<ArgCb> = Box::from_raw(mem::transmute(arg_cb));
        cb(&mut cx, this, callback);
    })
}
