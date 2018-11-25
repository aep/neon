//! Asynchronous background _tasks_ that run in the Node thread pool.

use std::mem;
use std::os::raw::c_void;

use types::*;
use result::JsResult;
use handle::{Handle, Managed};
use context::*;
use neon_runtime;
use neon_runtime::raw;
use std::sync::Arc;


struct ThreadSafeCBInner(*mut c_void);

impl Drop for ThreadSafeCBInner {
    fn drop(&mut self) {
        unsafe {
            neon_runtime::threadsafecb::delete(self.0);
        }
    }
}

#[derive(Clone)]
pub struct ThreadSafeCB(Arc<ThreadSafeCBInner>);

type ArgCb = for<'a> fn(&mut TaskContext<'a>) -> Vec<Handle<'a, JsValue>>;
type CompletionCb = fn(result: JsResult<JsValue>);

impl ThreadSafeCB {
    pub fn new<T: Value>(this: Handle<T>, callback: Handle<JsFunction>) -> Self {
        let cb = unsafe {
            neon_runtime::threadsafecb::new(this.to_raw(), callback.to_raw())
        };
        ThreadSafeCB(Arc::new(ThreadSafeCBInner(cb)))
    }

    pub fn call(&self, arg_cb: ArgCb, completion_cb: CompletionCb) {
        let arg_cb_raw = Box::into_raw(Box::new(arg_cb));
        let completion_cb_raw = Box::into_raw(Box::new(completion_cb));
        unsafe {
            neon_runtime::threadsafecb::call((*self.0).0, arg_cb_raw as *mut c_void,
                                         completion_cb_raw as *mut c_void,
                                         perform_arg_cb);
        }
    }
}

unsafe extern "C" fn perform_arg_cb(this: raw::Local, callback: raw::Local, arg_cb: *mut c_void, completion_cb: *mut c_void) {
    TaskContext::with(|mut cx: TaskContext| {
        let cb: Box<ArgCb> = Box::from_raw(mem::transmute(arg_cb));
        let args = cb(&mut cx);
        let this = JsValue::new_internal(this);
        let callback: JsFunction = JsFunction::from_raw(callback);
        let result = callback.call(&mut cx, this, args);
        let cb: Box<CompletionCb> = Box::from_raw(mem::transmute(completion_cb));
        cb(result);
    })
}
