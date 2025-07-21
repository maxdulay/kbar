use super::*;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_stream::try_stream;
use futures_lite::{Stream, StreamExt};

use neli::{
    consts::{genl::*, nl::*},
    err::RouterError,
    genl::Genlmsghdr,
    types::Buffer,
    utils::Groups,
};

use neli::consts::socket::NlFamily;
use neli::router::asynchronous::NlRouter;

use nl80211_stream::*;

pub enum NetError {
    RouterErrorU16Buffer(RouterError<u16, Buffer>),
    RouterErrorGenlIdGenlmsghdr(RouterError<GenlId, Genlmsghdr<CtrlCmd, CtrlAttr>>),
}

impl From<RouterError<u16, Buffer>> for NetError {
    fn from(e: RouterError<u16, Buffer>) -> Self {
        NetError::RouterErrorU16Buffer(e)
    }
}
impl From<RouterError<GenlId, Genlmsghdr<CtrlCmd, CtrlAttr>>> for NetError {
    fn from(e: RouterError<GenlId, Genlmsghdr<CtrlCmd, CtrlAttr>>) -> Self {
        NetError::RouterErrorGenlIdGenlmsghdr(e)
    }
}

pub type Result<T> = std::result::Result<T, NetError>;

pub struct EventStream {
    stream: Pin<Box<dyn Stream<Item = Result<Event>> + Send>>,
}

#[must_use = "streams nothing unless polled"]
impl EventStream {
    pub fn new() -> Self {
        let stream = try_stream! {
            let (socket, mut multicast) =
                NlRouter::connect(NlFamily::Generic, None, Groups::empty()).await?;
            let id = socket.resolve_nl_mcast_group("nl80211", "mlme").await?;
            socket.add_mcast_membership(Groups::new_groups(&[id]))?;
            loop {
                if let Some(Ok(msg)) = multicast.next::<GenlId, Genlmsghdr<Nl80211Command, Nl80211Attribute>>().await {
                    if let Ok(msg) = parse_event(msg) {
                        yield msg;
                    }
                }
            }
        };
        Self {
            stream: Box::pin(stream),
        }
    }
}

impl Stream for EventStream {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().stream.poll_next(cx)
    }

    type Item = Result<Event>;
}
