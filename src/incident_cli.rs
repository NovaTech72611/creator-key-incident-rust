use creator_key_incident::{handle_leaked_creator_key, plan_creator_incident, InfraiClient};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = std::env::args().nth(1).unwrap_or_else(|| "subscriber-demo".to_owned());
    let asset = std::env::args().nth(2).unwrap_or_else(|| "creator-asset-demo".to_owned());
    let plan = plan_creator_incident(subscriber, asset);
    let client = InfraiClient::from_env()?;
    let result = block_on(handle_leaked_creator_key(&client, &plan))?;
    println!("temporary key {} reviewed; delivery={} processing={}", result.temporary_key_id, plan.subscriber_update.delivery_state, plan.content_process.operation);
    println!("blast-radius search envelope: {}", result.log_envelope);
    Ok(())
}

fn block_on<F: Future>(future: F) -> F::Output {
    let waker = unsafe { Waker::from_raw(raw_waker()) };
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match Pin::as_mut(&mut future).poll(&mut context) { Poll::Ready(value) => return value, Poll::Pending => std::thread::yield_now() }
    }
}

fn raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker { raw_waker() }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    RawWaker::new(std::ptr::null(), &RawWakerVTable::new(clone, wake, wake_by_ref, drop))
}
