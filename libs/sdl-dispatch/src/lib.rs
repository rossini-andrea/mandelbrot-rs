use sdl2::event::{Event, EventSender};
use futures::{
    channel::oneshot,
};
use std::{
    fmt::Debug,
};

/// `SdlDispatcher` spawns new futures onto the SDL message pump.
pub struct SdlDispatcher {
    event_sender: EventSender,
}

/// A future that gets scheduled in SDL pump
pub struct SdlPumpTask<TIn: 'static + Send, TOut: 'static + Send> {
    /// Input data
    input: TIn,

    /// Output sender
    shared_state: oneshot::Sender<TOut>,
}

impl<TIn: 'static + Send, TOut: 'static + Send + Debug> SdlPumpTask<TIn, TOut> {
    pub fn input(&self) -> &TIn {
        &self.input
    }

    pub fn complete(self, result: TOut) {
        self.shared_state.send(result).unwrap();
    }
}

impl SdlDispatcher {
    pub fn from_eventsubsystem(sdl_events: &sdl2::EventSubsystem) -> Self {
        Self {
            event_sender: sdl_events.event_sender().into(),
        }
    }

    pub fn spawn<TIn: 'static + Send, TOut: 'static + Send + Debug>(&self, input: TIn) -> oneshot::Receiver<TOut> {
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

/// Inspects an SDL event, returning a task if it is a Task Notification.
/// Returns `None` if the event can't be converted to .
pub fn handle_sdl_event<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone + Debug>(event: &Event) -> Option<SdlPumpTask<TIn, TOut>> {
    if !event.is_user_event() {
        return None;
    }

    event.as_user_event_type::<SdlPumpTask<TIn, TOut>>()
}

pub fn register_task_type<TIn: 'static + Send + Clone, TOut: 'static + Send + Clone + Debug>(sdl_events: &sdl2::EventSubsystem) {
    sdl_events.register_custom_event::<SdlPumpTask<TIn, TOut>>().expect("Types already registered");
}

