use sdl2::event::{Event, EventSender};
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

/// Task executor that receives task notifications from SDL messages.
pub struct SdlExecutor/*<TIn, TOut>*/ {

}

/// `SdlDispatcher` spawns new futures onto the SDL message pump.
#[derive(Clone)]
pub struct SdlDispatcher/*<TIn, TOut>*/ {
    event_sender: Arc<EventSender>,
}

struct SharedState<TOut: 'static + Send + Clone> {
    completed: Poll<TOut>,
    waker: Option<Waker>,
}

/// A future that gets scheduled in SDL pump
pub struct SdlPumpTask<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone> {
    /// Event sender to reschedule the task
    event_sender: Arc<EventSender>,

    /// Input data
    input: TIn,

    shared_state: Arc<Mutex<SharedState<TOut>>>,
}

impl<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone> SdlPumpTask<TIn, TOut> {
    pub fn complete(&self, result: TOut) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.completed = Poll::Ready(result);
    }
}

/// A future for the awaiter side
pub struct Task<TOut: 'static + Send + Clone> {
    shared_state: Arc<Mutex<SharedState<TOut>>>,
}

impl SdlDispatcher/*<TIn, TOut>*/ {
    fn new(sdl_events: &sdl2::EventSubsystem) -> Self {
        Self {
            event_sender: sdl_events.event_sender().into(),
        }
    }

    pub fn spawn<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone>(&self, input: TIn) -> Task<TOut> {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: Poll::Pending,
            waker: None,
        }));
        let task = Arc::new(SdlPumpTask {
            event_sender: self.event_sender.clone(), // Here should be the SDL pump
            input: input,
            shared_state: shared_state.clone(),
        });

        self.event_sender.push_custom_event::<Arc<SdlPumpTask<TIn, TOut>>>(task)
            .expect("Can't push on SDL pump");
        Task::<TOut> {
            shared_state: shared_state.clone(),
        }
    }
}

/*
impl ArcWake for SdlPumpTask {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self.event_sender.push_custom_event::<Arc<SdlPumpTask>>(cloned)
            .expect("Can't push on SDL pump");
    }
}*/

impl SdlExecutor {
    /// Handles an SDL event, running a task if it is a Task Notification
    /// Returns `true` if the event was handled.
    pub fn handle_sdl_event<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone>(&self, event: &Event) -> Option<Arc<SdlPumpTask<TIn, TOut>>> {
        if !event.is_user_event() {
            return None;
        }

        event.as_user_event_type::<Arc<SdlPumpTask<TIn, TOut>>>()
        /*{
            Some(notification) => {
                let mut future_slot = notification.future.lock().unwrap();
                if let Some(mut future) = future_slot.take() {
                    let waker = waker_ref(&notification);
                    let context = &mut Context::from_waker(&waker);

                    if future.as_mut().poll(context).is_ready() {
                        let mut shared_state = notification.shared_state.lock().unwrap();
                        shared_state.completed = true;

                        if let Some(waker) = shared_state.waker.take() {
                            waker.wake();
                        }
                    }
                }
                true
            }
            None => false
        }*/
    }
}

impl<TOut: 'static + Send + Clone> Future for Task<TOut> {
    type Output = TOut;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed.is_pending() {
            shared_state.waker = Some(cx.waker().clone());
        }

        shared_state.completed.clone()
    }
}

pub fn new_executor_and_dispatcher<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone>(sdl_events: &sdl2::EventSubsystem) -> (SdlExecutor, SdlDispatcher) {
    sdl_events.register_custom_event::<Arc<SdlPumpTask<TIn, TOut>>>();
    (SdlExecutor {}, SdlDispatcher::new(sdl_events))
}

