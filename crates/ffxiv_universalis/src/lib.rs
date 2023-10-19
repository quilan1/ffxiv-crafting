#![warn(unused_crate_dependencies)]
#![allow(clippy::module_inception)]

mod multi_signal;
mod processor;
mod universalis;

////////////////////////////////////////////////////////////

use multi_signal::{multi_signal, MSender};

pub use multi_signal::MReceiver;
pub use processor::{
    FetchState, Processor, ProcessorHandle, ProcessorHandleOutput, Status, StatusController,
};
pub use universalis::{ListingsMap, RequestState};
pub mod json {
    pub use crate::universalis::json_types::*;
}

pub type Signal<T> = misc::Signal<T>;

////////////////////////////////////////////////////////////

mod misc {
    use futures::future::Shared;
    pub type Signal<T> = Shared<futures::channel::oneshot::Receiver<T>>;
}
