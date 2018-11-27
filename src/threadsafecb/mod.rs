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

impl ThreadSafeCB {
    pub fn new<T: Value>(this: Handle<T>, callback: Handle<JsFunction>) -> Self {
        let cb = unsafe {
            neon_runtime::threadsafecb::new(this.to_raw(), callback.to_raw())
        };
        ThreadSafeCB(Arc::new(ThreadSafeCBInner(cb)))
    }

    pub fn call<F: Fn(&mut TaskContext, Handle<JsValue>, Handle<JsFunction>)>(&self, arg_cb: F) {
        let callback = Box::into_raw(Box::new(arg_cb)) as *mut c_void;
        unsafe {
            neon_runtime::threadsafecb::call((*self.0).0, callback, handle_callback::<F>);
        }
    }
}

unsafe extern "C" fn handle_callback<F: Fn(&mut TaskContext, Handle<JsValue>, Handle<JsFunction>)>(this: raw::Local, func: raw::Local, callback: *mut c_void) {
    TaskContext::with(|mut cx: TaskContext| {
        let this = JsValue::new_internal(this);
        let func: Handle<JsFunction> = Handle::new_internal(JsFunction::from_raw(func));
        let callback: Box<F> = Box::from_raw(mem::transmute(callback));
        callback(&mut cx, this, func);
    })
}
