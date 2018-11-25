use neon::prelude::*;

/*
test with emit:
JS:
const e = new Emitter()
extend with event emitter
e.connect()
e.on('progress', (p) => {
    console.log('progress:', `{p}%`)
})
e.on('end', (result) => {
    assert(result === 100)
    console.log('finished:', `{result}%`)
    done()
})
e.start() // do work async

RS:

Emitter(Option<ThreadSafeCb)

init => Emitter(None)
connect => self.cb = ThreadSafeCb::new(this, 'emit')
start => rayon progress cb.call('progress', p), end cb.call('end', 100)

*/
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
            thread::spawn(|| {
                for i in 1..10 {
                    cb.call(|cx| {
                        return vec![cx.string("progress").upcast(), cx.number(i).upcast()];
                    }, |_| {
                    });
                    thread::sleep(Duration::from_millis(500));
                }
                cb.call(|cx| {
                    return vec![cx.string("end").upcast(), cx.number(12).upcast()];
                }, |_| {
                });
            });
            /*cb.call(|cx| {
                return vec![cx.string("end").upcast(), cx.number(12).upcast()];
            }, |_| {
            });*/
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
