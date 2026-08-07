use futures_util::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::Receiver;

/// Minimal zero-dependency adapter converting `tokio::sync::mpsc::Receiver` into a `futures::Stream`.
pub struct ChannelStream<T> {
    receiver: Receiver<T>,
}

impl<T> ChannelStream<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        Self { receiver }
    }
}

impl<T> Stream for ChannelStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
