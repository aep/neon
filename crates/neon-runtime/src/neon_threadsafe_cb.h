#ifndef NEON_THREADSAFE_CALLBACK_H_
#define NEON_THREADSAFE_CALLBACK_H_

#include <uv.h>
#include "neon.h"
#include "v8.h"
#include <mutex>

namespace neon {

    class ThreadSafeCallback {
    public:
        ThreadSafeCallback(v8::Isolate *isolate, v8::Local<v8::Value> self, v8::Local<v8::Function> callback): isolate_(isolate), close_(false)
        {
            async_.data = this;
            uv_async_init(uv_default_loop(), &async_, async_callback);
            // Save the this argument and the callback to be invoked.
            self_.Reset(isolate, self);
            callback_.Reset(isolate, callback);
            // Save the context (aka realm) to be used when invoking the callback.
            context_.Reset(isolate, isolate->GetCurrentContext());
        }

        void call(void *rust_callback, Neon_ThreadSafeCallbackHandler handler) {
            printf("before call\n");
            {
                std::lock_guard<std::mutex> lock(mutex_);
                printf("after call %d\n", close_);
                if (close_) {
                    return;
                }
                function_pairs_.push_back({ rust_callback, handler });
            }
            uv_async_send(&async_);
        }

        void close() {
            printf("before close\n");
            {
                std::lock_guard<std::mutex> lock(mutex_);
                printf("after close %d\n", close_);
                if (close_) {
                    return;
                }
                close_ = true;
            }
            uv_async_send(&async_);
        }

        void async_call() {
            printf("before async_call\n");
            std::lock_guard<std::mutex> lock(mutex_);
            printf("after async_call %d\n", close_);
            if (close_) {
                uv_close(reinterpret_cast<uv_handle_t*>(&async_), [](uv_handle_t* handle) {
                    delete static_cast<ThreadSafeCallback*>(handle->data);
                });
                return;
            }
            // Ensure that we have all the proper scopes installed on the C++ stack before
            // invoking the callback, and use the context (i.e. realm) we saved with the task.
            v8::Isolate::Scope isolate_scope(isolate_);
            v8::HandleScope handle_scope(isolate_);
            v8::Local<v8::Context> context = v8::Local<v8::Context>::New(isolate_, context_);
            v8::Context::Scope context_scope(context);

            v8::Local<v8::Value> self = v8::Local<v8::Value>::New(isolate_, self_);
            v8::Local<v8::Function> callback = v8::Local<v8::Function>::New(isolate_, callback_);
            for (const CbData &data : function_pairs_)
            {
                data.handler(self, callback, data.rust_callback);
            }
        }

    private:
        static void async_callback(uv_async_t* handle) {
            ThreadSafeCallback* cb = static_cast<ThreadSafeCallback*>(handle->data);
            cb->async_call();
        }

        uv_async_t async_;
        v8::Isolate *isolate_;
        v8::Persistent<v8::Value> self_;
        v8::Persistent<v8::Function> callback_;
        v8::Persistent<v8::Context> context_;

        struct CbData {
            void *rust_callback;
            Neon_ThreadSafeCallbackHandler handler;
        };

        std::mutex mutex_;
        std::vector<CbData> function_pairs_;
        bool close_;
    };
}

#endif
