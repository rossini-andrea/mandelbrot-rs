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
pub struct SdlExecutor {

}

/// `SdlDispatcher` spawns new futures onto the SDL message pump.
//#[derive(Clone)]
pub struct SdlDispatcher {
    event_sender: Arc<EventSender>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

/// A future that gets scheduled in SDL pump
struct SdlPumpTask {
    /// In-progress future that should be pushed to completion.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Event sender to reschedule the task
    event_sender: Arc<EventSender>,

    shared_state: Arc<Mutex<SharedState>>,
}

/// A future for the awaiter side
pub struct Task {
    shared_state: Arc<Mutex<SharedState>>,
}

impl SdlDispatcher {
    fn new(sdl_events: &sdl2::EventSubsystem) -> Self {
        Self {
            event_sender: sdl_events.event_sender().into(),
        }
    }

    pub fn spawn(&self, future: impl Future<Output=()> + 'static + Send) -> Task {
        let boxed = future.boxed();
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));
        let task = Arc::new(SdlPumpTask {
            future: Mutex::new(Some(boxed)),
            event_sender: self.event_sender.clone(), // Here should be the SDL pump
            shared_state: shared_state.clone(),
        });

        self.event_sender.push_custom_event::<Arc<SdlPumpTask>>(task)
            .expect("Can't push on SDL pump");
        Task {
            shared_state: shared_state.clone(),
        }
    }
}

impl ArcWake for SdlPumpTask {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self.event_sender.push_custom_event::<Arc<SdlPumpTask>>(cloned)
            .expect("Can't push on SDL pump");
    }
}

impl SdlExecutor {
    /// Handles an SDL event, running a task if it is a Task Notification
    /// Returns `true` if the event was handled.
    pub fn handle_sdl_event(&self, event: &Event) -> bool {
        if !event.is_user_event() {
            return false
        }

        match event.as_user_event_type::<Arc<SdlPumpTask>>() {
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
        }
    }
}

impl Future for Task {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `TimerFuture` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `TimerFuture` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn new_executor_and_dispatcher(sdl_events: &sdl2::EventSubsystem) -> (SdlExecutor, SdlDispatcher) {
    sdl_events.register_custom_event::<Arc<SdlPumpTask>>();
    (SdlExecutor {}, SdlDispatcher::new(sdl_events))
}

