use futures::{
    channel::oneshot::{Receiver, Sender},
    Stream,
};
use std::pin::Pin;

/// Stream wrapper that sends a message to a oneshot reciever upon being dropped.
#[derive(Debug)]
pub struct DropStream<T>
where
    T: Stream + ?Sized,
{
    pub stream: Pin<Box<T>>,
    pub tx: Option<Sender<()>>,
}

impl<T> DropStream<T>
where
    T: Stream + ?Sized,
{
    pub fn new(stream: Pin<Box<T>>) -> (DropStream<T>, Receiver<()>) {
        let (tx, rx) = futures::channel::oneshot::channel();

        let myself = Self {
            stream,
            tx: Some(tx),
        };

        return (myself, rx);
    }

    ///Given an existing `Stream` and `futures::channel::oneshot::Sender`, create a `DropStream` wrapper.
    pub fn from_existing_tx(stream: Pin<Box<T>>, tx: Sender<()>) -> DropStream<T> {
        let myself = Self {
            stream,
            tx: Some(tx),
        };

        return myself;
    }
}

#[cfg(feature = "tokio")]
impl<T> DropStream<T>
where
    T: Stream + ?Sized,
{
    pub fn new_closure<Fut>(stream: Pin<Box<T>>, future: Fut) -> DropStream<T>
    where
        Fut: Future + 'static,
        Fut::Ouput: 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();

        let myself = Self {
            stream,
            tx: Some(tx),
        };

        tokio::spawn(async move {
            let _ = rx.await;
            let _ = future.await;
        });

        return myself;
    }
}

impl<T> Drop for DropStream<T>
where
    T: Stream + ?Sized,
{
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        };
    }
}

impl<T> Stream for DropStream<T>
where
    T: Stream + ?Sized,
{
    type Item = T::Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
