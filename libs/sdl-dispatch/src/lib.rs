use sdl2::event::{Event, EventSender};
use futures::{
    channel::oneshot,
};
use std::{
    fmt::Debug,
    sync::Arc,
};

/// Task executor that receives task notifications from SDL messages.
pub struct SdlExecutor {

}

/// `SdlDispatcher` spawns new futures onto the SDL message pump.
#[derive(Clone)]
pub struct SdlDispatcher {
    event_sender: Arc<EventSender>,
}

/// A future that gets scheduled in SDL pump
pub struct SdlPumpTask<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone> {
    /// Input data
    input: TIn,

    /// Output sender
    shared_state: oneshot::Sender<TOut>,
}

impl<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone + Debug> SdlPumpTask<TIn, TOut> {
    pub fn input(&self) -> &TIn {
        &self.input
    }

    pub fn complete(self, result: TOut) {
        self.shared_state.send(result).unwrap();
    }
}

impl SdlDispatcher {
    fn new(sdl_events: &sdl2::EventSubsystem) -> Self {
        Self {
            event_sender: sdl_events.event_sender().into(),
        }
    }

    pub fn spawn<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone + Debug>(&self, input: TIn) -> oneshot::Receiver<TOut> {
        let (sender, receiver) = oneshot::channel::<TOut>();
        let task = SdlPumpTask {
            input: input,
            shared_state: sender,
        };

        self.event_sender.push_custom_event::<SdlPumpTask<TIn, TOut>>(task)
            .expect("Can't push on SDL pump");
        receiver
    }
}

impl SdlExecutor {
    /// Handles an SDL event, running a task if it is a Task Notification
    /// Returns `true` if the event was handled.
    pub fn handle_sdl_event<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone + Debug>(&self, event: &Event) -> Option<SdlPumpTask<TIn, TOut>> {
        if !event.is_user_event() {
            return None;
        }

        event.as_user_event_type::<SdlPumpTask<TIn, TOut>>()
   }
}

pub fn new_executor_and_dispatcher<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone + Debug>(sdl_events: &sdl2::EventSubsystem) -> (SdlExecutor, SdlDispatcher) {
    sdl_events.register_custom_event::<SdlPumpTask<TIn, TOut>>().expect("Types already registered");
    (SdlExecutor {}, SdlDispatcher::new(sdl_events))
}

