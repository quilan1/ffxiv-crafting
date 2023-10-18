#![warn(unused_crate_dependencies)]
#![allow(clippy::module_inception)]

mod processor;
mod universalis;

////////////////////////////////////////////////////////////

pub use processor::{
    FetchState, Processor, ProcessorHandle, ProcessorHandleOutput, Status, StatusController,
};
pub use universalis::ListingsMap;
pub type Signal<T> = misc::Signal<T>;
pub mod json {
    pub use crate::universalis::json_types::*;
}

////////////////////////////////////////////////////////////

mod misc {
    use futures::channel::oneshot::Receiver;
    use futures::future::Shared;
    pub type Signal<T> = Shared<Receiver<T>>;
}
