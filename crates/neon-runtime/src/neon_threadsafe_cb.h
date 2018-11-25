#ifndef NEON_THREADSAFE_CB_H_
#define NEON_THREADSAFE_CB_H_

#include <uv.h>
#include "neon.h"
#include "v8.h"

namespace neon {

    class ThreadSafeCb {
    public:
        ThreadSafeCb(v8::Isolate *isolate,
            v8::Local<v8::Value> self,
            v8::Local<v8::Function> callback)
            : isolate_(isolate)
        {
            async_.data = this;
            uv_async_init(uv_default_loop(), &async_, async_callback);
            // Save the callback to be invoked when the task completes.
            self_.Reset(isolate, self);
            // Save the callback to be invoked when the task completes.
            callback_.Reset(isolate, callback);
            // Save the context (aka realm) to be used when invoking the callback.
            context_.Reset(isolate, isolate->GetCurrentContext());
        }

        void call(void *arg_cb_raw, void *completion_cb_raw, Neon_ThreadSafeCbCallback complete) {
            // TODO: lock mutex
            // std::lock_guard<std::mutex> lock(mutex_);
            function_pairs_.push_back({ arg_cb_raw, completion_cb_raw, complete });
            uv_async_send(&async_);
        }

        void async_call() {

            // TODO: lock mutex
            // std::lock_guard<std::mutex> lock(mutex_);
            if (close_) {
                uv_close(reinterpret_cast<uv_handle_t *>(&async_), [](uv_handle_t *handle) {
                    delete static_cast<ThreadSafeCb *>(handle->data);
                });
                return;
            }
            printf("closed? %d\n", close_);
            {
                // Ensure that we have all the proper scopes installed on the C++ stack before
                // invoking the callback, and use the context (i.e. realm) we saved with the task.
                v8::Isolate::Scope isolate_scope(isolate_);
                v8::HandleScope handle_scope(isolate_);
                v8::Local<v8::Context> context = v8::Local<v8::Context>::New(isolate_, context_);
                v8::Context::Scope context_scope(context);
                while (true)
                {
                    std::vector<CbData> func_pairs;
                    {
                        // TODO: lock mutex
                        // std::lock_guard<std::mutex> lock(mutex_);
                        if (function_pairs_.empty())
                            break;
                        else
                            func_pairs.swap(function_pairs_);
                    }
                    v8::Local<v8::Value> self = v8::Local<v8::Value>::New(isolate_, self_);
                    v8::Local<v8::Function> callback = v8::Local<v8::Function>::New(isolate_, callback_);
                    for (const CbData &data : func_pairs)
                    {
                        data.complete(self, callback, data.arg_cb, data.completion_cb);
                    }
                }
            }
        }

        void close() {
            // TODO: lock mutex
            // std::lock_guard<std::mutex> lock(mutex_);
            close_ = true;
            uv_async_send(&async_);
        }

    private:
        static void async_callback(uv_async_t *handle) {
            ThreadSafeCb *cb = static_cast<ThreadSafeCb*>(handle->data);
            cb->async_call();
        }

        uv_async_t async_;
        v8::Isolate *isolate_;
        v8::Persistent<v8::Value> self_;
        v8::Persistent<v8::Function> callback_;
        v8::Persistent<v8::Context> context_;

        struct CbData {
            void *arg_cb;
            void *completion_cb;
            Neon_ThreadSafeCbCallback complete;
        };
        std::vector<CbData> function_pairs_;

        bool close_;
    };
}

#endif
