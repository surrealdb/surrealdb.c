use std::ffi::c_int;

use async_channel::Receiver;
use futures::StreamExt;
use surrealdb::method::Stream as sdbStream;
use surrealdb::rpc::format::cbor::Cbor;
use surrealdb::Value as apiValue;
use surrealdb::{dbs, sql};
use tokio::runtime::Handle;

use crate::SR_ERROR;
use crate::{notification::Notification, SR_CLOSED, SR_NONE};

use super::array::MakeArray;

/// may be sent across threads, but must not be aliased
pub struct Stream {
    inner: sdbStream<apiValue>,
    rt: Handle,
}

impl Stream {
    pub fn new(inner: sdbStream<apiValue>, rt: Handle) -> Stream {
        Stream { inner, rt }
    }
}

impl Stream {
    // TODO: add try catch here, and add poison?

    /// blocks until next item is recieved on stream
    /// will return 1 and write notification to notification_ptr is recieved
    /// will return SR_NONE if the stream is closed
    #[export_name = "sr_stream_next"]
    pub extern "C" fn next(&mut self, notification_ptr: *mut Notification) -> c_int {
        match self.rt.block_on(self.inner.next()) {
            Some(n) => {
                unsafe { notification_ptr.write(n.into()) }
                1
            }
            None => SR_NONE,
        }
    }

    #[export_name = "sr_stream_kill"]
    pub extern "C" fn kill(stream: *mut Stream) {
        let boxed = unsafe { Box::from_raw(stream) };
        let handle = boxed.rt.clone();
        handle.block_on(async { drop(boxed) });
    }
}

// TODO: check if this needs to be dropped async
pub struct RpcStream {
    rx: Receiver<dbs::Notification>,
}

impl RpcStream {
    pub extern "C" fn next(&mut self, res_ptr: *mut *mut u8) -> c_int {
        let not = match self.rx.recv_blocking() {
            Ok(n) => n,
            Err(_) => return SR_CLOSED,
        };

        // Construct live message
        let mut message = sql::Object::default();
        message.insert("id".to_string(), not.id.into());
        message.insert("action".to_string(), not.action.to_string().into());
        message.insert("result".to_string(), not.result);

        // Into CBOR value
        let cbor: Cbor = match sql::Value::Object(message).try_into() {
            Ok(c) => c,
            Err(_) => return SR_ERROR,
        };

        let mut res = Vec::new();
        ciborium::into_writer(&cbor.0, &mut res).unwrap();
        let out = res.make_array();

        unsafe { res_ptr.write(out.ptr) }

        out.len
    }
}
