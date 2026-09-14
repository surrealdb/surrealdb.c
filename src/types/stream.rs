use std::ffi::c_int;
use std::time::{Duration, Instant};

use async_channel::Receiver;
use futures::StreamExt;
use surrealdb::method::Stream as sdbStream;
use surrealdb::types::{Value as sdbValue, Notification as PublicNotification};
use tokio::runtime::Handle;

use crate::SR_ERROR;
use crate::{notification::Notification, SR_CLOSED, SR_NONE, SR_TIMEOUT};

use super::array::MakeArray;

/// Stream for receiving live query notifications
///
/// May be sent across threads, but must not be aliased.
/// Use `sr_stream_next` to receive notifications and `sr_stream_kill` to close.
pub struct Stream {
    inner: sdbStream<sdbValue>,
    rt: Handle,
}

impl Stream {
    pub fn new(inner: sdbStream<sdbValue>, rt: Handle) -> Stream {
        Stream { inner, rt }
    }
}

impl Stream {
    /// Get the next notification, blocking until one arrives
    ///
    /// Returns 1 and writes to `notification_ptr` when a notification is received,
    /// SR_NONE when the stream has ended, and SR_ERROR on a stream error.
    ///
    /// # Blocking and shutdown
    ///
    /// This call blocks until a notification is available. It is intended to be
    /// driven from a dedicated thread rather than a latency-sensitive one; see
    /// `sr_stream_next_timeout` for a bounded or non-blocking wait.
    ///
    /// A stream borrows the runtime owned by the connection it was opened on, so
    /// teardown is ordered: call `sr_stream_kill` first, then
    /// `sr_surreal_disconnect`. Disconnecting first drops that runtime out from
    /// under the stream, and the kill then runs against a runtime that is already
    /// shut down.
    ///
    /// There is no way to release a reader already parked in this call from another
    /// thread; `sr_stream_kill` frees the very stream that reader is borrowing. Call
    /// this only when an event is expected, and prefer `sr_stream_next_timeout`
    /// for a reader that has to be able to give up.
    #[export_name = "sr_stream_next"]
    pub extern "C" fn next(&mut self, notification_ptr: *mut Notification) -> c_int {
        match self.rt.block_on(self.inner.next()) {
            Some(Ok(n)) => Self::deliver(n, notification_ptr),
            Some(Err(_)) => SR_ERROR,
            None => SR_NONE,
        }
    }

    /// Get the next notification, waiting no longer than `timeout_ms`
    ///
    /// Returns 1 and writes to `notification_ptr` when a notification is
    /// received, SR_TIMEOUT when the wait expired with the stream still open,
    /// SR_NONE when the stream has ended, and SR_ERROR on a stream error.
    ///
    /// `timeout_ms` follows `poll(2)`: negative waits indefinitely and is exactly
    /// `sr_stream_next`, zero polls once and returns immediately, and a positive
    /// value waits up to that many milliseconds.
    ///
    /// # SR_TIMEOUT is not SR_NONE
    ///
    /// A timeout means nothing has arrived yet and the stream is still live, so
    /// call again. SR_NONE means the stream has ended and calling again is
    /// pointless. Collapsing the two turns a merely slow notification into an
    /// abandoned stream, or an ended stream into a spin.
    ///
    /// An expired wait consumes nothing. Notifications queue in a channel, and a
    /// poll that finds it empty leaves any later arrival in place, so polling in
    /// a loop cannot lose an event.
    ///
    /// This is the call for a reader that must stay responsive to anything other
    /// than the stream -- a shutdown flag, usually, since `sr_stream_kill` cannot
    /// release a reader already parked inside `sr_stream_next`.
    #[export_name = "sr_stream_next_timeout"]
    pub extern "C" fn next_timeout(
        &mut self,
        notification_ptr: *mut Notification,
        timeout_ms: c_int,
    ) -> c_int {
        if timeout_ms < 0 {
            return self.next(notification_ptr);
        }
        // Cloned so the future below can borrow `inner` mutably without also
        // holding a borrow of `rt`.
        let rt = self.rt.clone();
        let wait = Duration::from_millis(timeout_ms as u64);
        // The timeout is constructed inside `block_on`, not outside it: building
        // one registers with the runtime's timer driver, and doing that from a
        // plain C thread panics with "there is no reactor running".
        //
        // tokio polls the inner future before the timer, so a zero duration is
        // still one real poll rather than an unconditional expiry.
        let inner = &mut self.inner;
        match rt.block_on(async { tokio::time::timeout(wait, inner.next()).await }) {
            Ok(Some(Ok(n))) => Self::deliver(n, notification_ptr),
            Ok(Some(Err(_))) => SR_ERROR,
            Ok(None) => SR_NONE,
            Err(_elapsed) => SR_TIMEOUT,
        }
    }

    /// Hand one notification to C. Shared by both `next` variants.
    fn deliver(
        n: surrealdb::Notification<sdbValue>,
        notification_ptr: *mut Notification,
    ) -> c_int {
        let notif = Notification {
            query_id: crate::uuid::Uuid::from(n.query_id),
            action: crate::notification::Action::from(n.action),
            data: crate::value::Value::from(n.data),
        };
        unsafe { notification_ptr.write(notif) }
        1
    }

    /// Kill and free a stream
    ///
    /// Closes the stream and releases all associated resources.
    /// The stream must not be used after calling this function.
    ///
    /// This runs on the runtime owned by the connection the stream was opened on,
    /// so it must be called before `sr_surreal_disconnect` on that connection.
    #[export_name = "sr_stream_kill"]
    pub extern "C" fn kill(stream: *mut Stream) {
        let boxed = unsafe { Box::from_raw(stream) };
        let handle = boxed.rt.clone();
        handle.block_on(async { drop(boxed) });
    }
}

/// Stream for receiving RPC live query notifications
///
/// Wraps a `Receiver<PublicNotification>` from the datastore's notification channel.
/// Uses synchronous blocking receives, so no async drop is required.
pub struct RpcStream {
    rx: Receiver<PublicNotification>,
}

impl RpcStream {
    /// Create a new RpcStream from a notification receiver
    pub fn new(rx: Receiver<PublicNotification>) -> Self {
        RpcStream { rx }
    }
    /// Get the next notification, blocking until one arrives
    ///
    /// # Blocking and shutdown
    ///
    /// This call blocks until a notification is available. It is intended to be
    /// driven from a dedicated thread rather than a latency-sensitive one; see
    /// `sr_rpc_stream_next_timeout` for a bounded or non-blocking wait.
    ///
    /// To retire that thread, free the connection the stream came from. Doing so
    /// drops the sending half of the notification channel, and a reader parked
    /// inside this function returns SR_CLOSED. The stream itself stays valid and
    /// must still be freed.
    #[export_name = "sr_rpc_stream_next"]
    pub extern "C" fn next(&mut self, res_ptr: *mut *mut u8) -> c_int {
        let notification = match self.rx.recv_blocking() {
            Ok(n) => n,
            Err(_) => return SR_CLOSED,
        };
        Self::encode(notification, res_ptr)
    }

    /// Get the next notification, waiting no longer than `timeout_ms`
    ///
    /// Returns the payload length and writes to `res_ptr` on success, SR_TIMEOUT
    /// when the wait expired with the channel still open, SR_CLOSED when the
    /// sending half is gone, and SR_ERROR if the payload could not be encoded.
    ///
    /// `timeout_ms` follows `poll(2)`: negative waits indefinitely and is exactly
    /// `sr_rpc_stream_next`, zero polls once and returns immediately, and a
    /// positive value waits up to that many milliseconds.
    ///
    /// SR_TIMEOUT means call again; SR_CLOSED means stop. An expired wait takes
    /// nothing off the channel, so a later notification is still delivered.
    ///
    /// Unlike `sr_stream_next_timeout` this does not touch a tokio runtime, which
    /// is deliberate: an RpcStream outlives the connection it came from, so it
    /// must not hold a handle to a runtime that may already be shut down. The
    /// wait is therefore a poll at millisecond granularity rather than a timer.
    #[export_name = "sr_rpc_stream_next_timeout"]
    pub extern "C" fn next_timeout(&mut self, res_ptr: *mut *mut u8, timeout_ms: c_int) -> c_int {
        if timeout_ms < 0 {
            return self.next(res_ptr);
        }
        let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
        let notification = loop {
            match self.rx.try_recv() {
                Ok(n) => break n,
                // Checked every pass rather than only on expiry, so freeing the
                // connection still releases this reader promptly.
                Err(async_channel::TryRecvError::Closed) => return SR_CLOSED,
                Err(async_channel::TryRecvError::Empty) => {}
            }
            let now = Instant::now();
            if now >= deadline {
                return SR_TIMEOUT;
            }
            std::thread::sleep(std::cmp::min(deadline - now, Duration::from_millis(1)));
        };
        Self::encode(notification, res_ptr)
    }

    /// CBOR-encode one notification into `res_ptr`. Shared by both variants.
    fn encode(notification: PublicNotification, res_ptr: *mut *mut u8) -> c_int {
        let mut obj = surrealdb::types::Object::new();
        obj.insert(
            "id".to_string(),
            sdbValue::Uuid(notification.id),
        );
        obj.insert(
            "action".to_string(),
            sdbValue::String(format!("{}", notification.action)),
        );
        obj.insert("result".to_string(), notification.result);

        let cbor_val = crate::rpc::value_to_cbor(&sdbValue::Object(obj));
        let mut res = Vec::new();
        if ciborium::into_writer(&cbor_val, &mut res).is_err() {
            return SR_ERROR;
        }
        let out = res.make_array();

        unsafe { res_ptr.write(out.ptr) }

        out.len
    }

    /// Free an RpcStream
    #[export_name = "sr_rpc_stream_free"]
    pub extern "C" fn free(stream: *mut RpcStream) {
        if !stream.is_null() {
            let _ = unsafe { Box::from_raw(stream) };
        }
    }
}
