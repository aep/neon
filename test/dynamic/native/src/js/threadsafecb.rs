use neon::prelude::*;

use std::thread;
use std::time::Duration;

pub struct Emitter {
  cb: Option<ThreadSafeCB>,
}

declare_types! {
  pub class JsEmitter for Emitter {
    init(_) {
      Ok(Emitter {
        cb: None
      })
    }

    constructor(mut cx) {
      let mut this = cx.this();
      let f = this.get(&mut cx, "emit")?.downcast::<JsFunction>().or_throw(&mut cx)?;
      let cb = ThreadSafeCB::new(this, f);
      {
        let guard = cx.lock();
        let mut callback = this.borrow_mut(&guard);
        callback.cb = Some(cb);
      }
      Ok(None)
    }

    method start(mut cx) {
        let this = cx.this();
        let cb = {
            let guard = cx.lock();
            let callback = this.borrow(&guard);
            callback.cb.clone()
        };
        if let Some(cb) = cb {
            thread::spawn(move || {
                for i in 1..10 {
                    cb.call(|cx, this, callback| {
                        let args : Vec<Handle<JsValue>> = vec![cx.string("progress").upcast(), cx.number(i).upcast()];
                        let result = callback.call(cx, this, args);
                        match(result) {
                          Ok(v) => println!("{}", v.to_string(cx).unwrap().value()),
                          Err(e) => println!("{}", e),
                        }
                    });
                    thread::sleep(Duration::from_millis(40));
                }
                cb.call(|cx, this, callback| {
                    let args : Vec<Handle<JsValue>> = vec![cx.string("end").upcast(), cx.number(100).upcast()];
                    let _result = callback.call(cx, this, args);
                });
            });
        }
        Ok(cx.undefined().upcast())
    }

    method shutdown(mut cx) {
      let mut this = cx.this();
      {
        let guard = cx.lock();
        let mut callback = this.borrow_mut(&guard);
        callback.cb = None;
      }
      Ok(cx.undefined().upcast())
    }
  }
}
