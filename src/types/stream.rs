use std::ffi::c_int;

use futures::StreamExt;
use surrealdb::{method::Stream as sdbStream, sql};
use tokio::runtime::Handle;

use crate::{notification::Notification, SR_NONE};

/// may be sent across threads, but must not be aliased
pub struct Stream {
    inner: sdbStream<sql::Value>,
    rt: Handle,
}

impl Stream {
    pub fn new(inner: sdbStream<sql::Value>, rt: Handle) -> Stream {
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
