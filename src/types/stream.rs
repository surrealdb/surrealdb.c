use std::{fmt::Display, ptr};

use futures::StreamExt;
use surrealdb::{engine::any::Any, method::Stream as sdbStream, sql};
use tokio::runtime::Handle;

use crate::{notification::Notification, result::SurrealError, Surreal};

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
    #[no_mangle]
    pub extern "C" fn next(&mut self) -> *mut Notification {
        match self.rt.block_on(self.inner.next()) {
            Some(n) => {
                let mut n = n.into();
                &mut n
            }
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn kill(stream: *mut Stream) {
        let boxed = unsafe { Box::from_raw(stream) };
        let handle = boxed.rt.clone();
        handle.block_on(async { drop(boxed) });
    }
}

#[repr(C)]
pub struct StreamResult {
    pub ok: *mut Stream,
    pub err: SurrealError,
}

impl StreamResult {
    pub fn err(msg: impl Display) -> Self {
        Self {
            ok: ptr::null_mut(),
            err: SurrealError::from_msg(msg),
        }
    }
    pub fn ok(ok: &mut Stream) -> Self {
        Self {
            ok,
            err: SurrealError::empty(),
        }
    }
}
