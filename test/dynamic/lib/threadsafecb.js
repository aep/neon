var addon = require('../native');
var events = require('events');
var util = require('util');

util.inherits(addon.Emitter, events.EventEmitter);

describe('ThreadSafeCb', function() {
  it('calls a cb async', function (done) {
    const e = new addon.Emitter();

    e.on('progress', (p) => {
        console.log('progress:', `${p}%`)
    })
    e.on('end', (result) => {
        console.log('finished:', `${result}%`)
        e.shutdown()
        done()
    })
    e.start()
  });
});
