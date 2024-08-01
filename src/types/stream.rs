use futures::StreamExt;
use surrealdb::{method::Stream as sdbStream, sql};
use tokio::runtime::Handle;

use crate::notification::Notification;

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
    #[export_name = "sr_stream_next"]
    pub extern "C" fn next(&mut self) -> Notification {
        self.rt.block_on(self.inner.next()).into()
    }

    #[export_name = "sr_stream_kill"]
    pub extern "C" fn kill(stream: *mut Stream) {
        let boxed = unsafe { Box::from_raw(stream) };
        let handle = boxed.rt.clone();
        handle.block_on(async { drop(boxed) });
    }
}
