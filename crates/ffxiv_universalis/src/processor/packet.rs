use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_processor::AsyncProcessorHandle;
use futures::{Future, FutureExt};

use crate::universalis::{RequestHandle, RequestResult};

////////////////////////////////////////////////////////////

pub struct RequestPacket(pub RequestHandle, pub RequestHandle);

pub struct PacketRequestResult(pub RequestResult, pub RequestResult);

pub struct AsyncPacket(PacketData, PacketData);

enum PacketData {
    Handle(AsyncProcessorHandle<RequestResult>),
    Result(RequestResult),
    Done,
}

////////////////////////////////////////////////////////////

impl AsyncPacket {
    pub fn new(
        listings_async: AsyncProcessorHandle<RequestResult>,
        history_async: AsyncProcessorHandle<RequestResult>,
    ) -> Self {
        Self(
            PacketData::Handle(listings_async),
            PacketData::Handle(history_async),
        )
    }
}

fn poll_handle_to_result(data: &mut PacketData, cx: &mut Context<'_>) {
    if let PacketData::Handle(ref mut handle) = data {
        if let Poll::Ready(result) = handle.poll_unpin(cx) {
            *data = PacketData::Result(result)
        }
    }
}

fn extract_result(data: &mut PacketData) -> RequestResult {
    let PacketData::Result(res) = std::mem::replace(data, PacketData::Done) else {
        unreachable!()
    };
    res
}

impl Future for AsyncPacket {
    type Output = PacketRequestResult;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_handle_to_result(&mut self.0, cx);
        poll_handle_to_result(&mut self.1, cx);

        match (&self.0, &self.1) {
            (PacketData::Result(_), PacketData::Result(_)) => {}
            _ => return Poll::Pending,
        }

        // They're both results, so let's migrate them to done, and return the result
        let listings_result = extract_result(&mut self.0);
        let history_result = extract_result(&mut self.1);
        Poll::Ready(PacketRequestResult(listings_result, history_result))
    }
}
