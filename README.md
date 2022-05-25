# drop-stream
Do you need to know when your stream is dropped, after you pass it off?

`DropStream` will send a message to a `futures::channel::oneshot::Reciever` upon being dropped. You can run cleanup code and close connections when your stream's consumer drops it.
This is especially useful when using frameworks that consume data streams.

See on [crates.io](https://crates.io/crates/drop-stream)

## Example
```rust
struct MyDataService {
  db_ref: Arc<SqlDatabase> //Example database type here
}

impl MyDataService {
  async get_change_feed(&self, user_id: String) -> Pin<Box<dyn Stream<Item = User>>> {

    // I need to close this session when the consumer stops listening to my stream
    let session = self.db_ref.new_session(user_id).await;
    
    let data_stream = session.changes();
    
    // This new wrapped stream will send a message to `rx` when dropped
    let (wrapped_stream, rx) = DropStream::new(data_stream);
    
    // Since using futures::channel::oneshot::Reciever, I can `await` the message
    tokio::spawn(async move {
      let _ = rx.await; // No actual value is passed as the message. We only care that a send was intended
      log::debug!("Closing stream...");
      let _close_success = session.close().await;
    });
    
    // Now I know I will close the session when the consumer is done with this :)
    wrapped_stream.boxed()
    
  }
}
```
