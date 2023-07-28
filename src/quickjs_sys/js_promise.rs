use std::{future::Future, sync::atomic::Ordering, task::Poll};

use crate::{quickjs_sys::qjs::JS_ExecutePendingJob, Context, EventLoop, JsValue, Runtime};

use super::{
    qjs::{js_std_dump_error, JSContext, JS_GetRuntimeOpaque},
    RuntimeResult,
};

impl Context {
    pub fn future_to_promise(
        &mut self,
        f: impl Future<Output = Result<JsValue, JsValue>> + std::marker::Send + 'static,
    ) -> JsValue {
        let waker = self
            .event_loop()
            .and_then(|event_loop| event_loop.waker.clone());

        println!("future_to_promise {waker:?}");

        let (promise, resolve, reject) = self.new_promise();

        let handle = tokio::task::spawn(async move {
            println!("future_to_promise start");
            match f.await {
                Ok(value) => {
                    if let JsValue::Function(f) = resolve {
                        f.call(&[value]);
                    }
                }
                Err(err) => {
                    if let JsValue::Function(f) = reject {
                        f.call(&[err]);
                    }
                }
            }
            println!("rt {:?} wake", waker);
            log::trace!("rt {:?} wake", waker);
            waker.map(|waker| waker.wake());
        });

        self.event_loop().map(|event_loop| {
            event_loop.sub_tasks.push_back(handle);
        });
        promise
    }
}
impl Future for Runtime {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        println!("runtime poll");
        unsafe {
            let rt = self.rt.0;
            let event_loop = { (JS_GetRuntimeOpaque(rt) as *mut EventLoop).as_mut() };
            if let Some(event_loop) = event_loop {
                let waker = cx.waker().clone();
                log::info!("insert rt waker {waker:?}");
                event_loop.waker.insert(waker);

                // println!("1 sub_tasks size = {}", event_loop.sub_tasks.len());
                // loop {
                //     match event_loop.sub_tasks.pop_front() {
                //         Some(task) => {
                //             if task.is_finished() {
                //                 continue;
                //             } else {
                //                 event_loop.sub_tasks.push_front(task);
                //                 log::trace!("Runtime Pending 1");
                //                 println!("Runtime pending 1");
                //                 return Poll::Pending;
                //             }
                //         }
                //         None => {
                //             println!("Runtime Ready 1");
                //             break;
                //         }
                //     }
                // }

                if self.run_loop_without_io() < 0 {
                    println!("Runtime io error");
                    return Poll::Ready(());
                }
                println!("2 sub_tasks size = {}", event_loop.sub_tasks.len());
                loop {
                    match event_loop.sub_tasks.pop_front() {
                        Some(task) => {
                            if task.is_finished() {
                                continue;
                            } else {
                                event_loop.sub_tasks.push_front(task);
                                log::trace!("Runtime Pending");
                                println!("Runtime pending");
                                return Poll::Pending;
                            }
                        }
                        None => {
                            println!("Runtime Ready");
                            return Poll::Ready(());
                        }
                    }
                }
            } else {
                Poll::Ready(())
            }
        }
    }
}

impl<'rt> Future for RuntimeResult<'rt> {
    type Output = Result<JsValue, ()>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        if me.result.is_none() && me.box_fn.is_some() {
            unsafe {
                let rt = me.rt.rt.0;
                let event_loop = { (JS_GetRuntimeOpaque(rt) as *mut EventLoop).as_mut() };
                if let Some(event_loop) = event_loop {
                    event_loop.waker.insert(cx.waker().clone());
                } else {
                    return Poll::Ready(Err(()));
                }
                let f = me.box_fn.take().unwrap();
                me.result = Some(f(&mut me.rt.ctx));
            }
        }
        let rt = &mut me.rt;
        tokio::pin!(rt);
        std::task::ready!(rt.poll(cx));
        Poll::Ready(me.result.take().ok_or(()))
    }
}
