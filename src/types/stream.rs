use std::ffi::c_int;

use async_channel::Receiver;
use futures::StreamExt;
use surrealdb::method::Stream as sdbStream;
use surrealdb::types::{Value as sdbValue, Notification as PublicNotification};
use tokio::runtime::Handle;

use crate::SR_ERROR;
use crate::{notification::Notification, SR_CLOSED, SR_NONE};

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
    /// Blocks until next item is received on stream
    /// will return 1 and write notification to notification_ptr if received
    /// will return SR_NONE if the stream is closed
    ///
    /// sr_stream_t *stream;
    /// if (sr_select_live(db, &err, &stream, "foo") < 0)
    /// {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    ///
    /// sr_notification_t not ;
    /// if (sr_stream_next(stream, &not ) > 0)
    /// {
    ///     sr_print_notification(&not );
    /// }
    /// sr_stream_kill(stream);
    #[export_name = "sr_stream_next"]
    pub extern "C" fn next(&mut self, notification_ptr: *mut Notification) -> c_int {
        match self.rt.block_on(self.inner.next()) {
            Some(Ok(n)) => {
                let notif = Notification {
                    query_id: crate::uuid::Uuid::from(n.query_id),
                    action: crate::notification::Action::from(n.action),
                    data: crate::value::Value::from(n.data),
                };
                unsafe { notification_ptr.write(notif) }
                1
            }
            Some(Err(_)) => SR_ERROR,
            None => SR_NONE,
        }
    }

    /// Kill and free a stream
    ///
    /// Closes the stream and releases all associated resources.
    /// The stream must not be used after calling this function.
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

    /// Get the next notification from the stream
    ///
    /// Returns the length of the CBOR-encoded notification, or SR_CLOSED if the
    /// channel is closed. The CBOR-encoded bytes are written to *res_ptr.
    ///
    /// Free the result with sr_free_byte_arr.
    #[export_name = "sr_rpc_stream_next"]
    pub extern "C" fn next(&mut self, res_ptr: *mut *mut u8) -> c_int {
        let notification = match self.rx.recv_blocking() {
            Ok(n) => n,
            Err(_) => return SR_CLOSED,
        };

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
