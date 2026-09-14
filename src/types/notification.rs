use crate::{uuid::Uuid, value::Value};
use surrealdb::types::{Action as sdbAction, Notification as sdbNotification};

#[derive(Debug)]
#[repr(C)]
pub struct Notification {
    pub query_id: Uuid,
    pub action: Action,
    pub data: Value,
}

impl Notification {
    /// Release a notification filled in by [`sr_stream_next`].
    ///
    /// The notification's `data` is an owned Value living in the caller's own
    /// storage, not a boxed one, so `sr_value_free` must not be used on it:
    /// that function reclaims a Box and would be handed a pointer it did not
    /// allocate. Without this entry point a caller had no way to release the
    /// value at all, and every notification leaked its contents.
    #[export_name = "sr_notification_free"]
    pub extern "C" fn notification_free(notification: Notification) {
        drop(notification)
    }
}

/// Convert from the protocol-level Notification (surrealdb_types::Notification)
/// Used by the RPC notification stream path.
impl From<sdbNotification> for Notification {
    fn from(value: sdbNotification) -> Self {
        Notification {
            query_id: value.id.into(),
            action: value.action.into(),
            data: Value::from(value.result),
        }
    }
}

impl Notification {
    #[export_name = "sr_print_notification"]
    pub extern "C" fn print_notification(notification: &Notification) {
        println!("{notification:?}");
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub enum Action {
    SR_ACTION_CREATE,
    SR_ACTION_UPDATE,
    SR_ACTION_DELETE,
    SR_ACTION_KILLED,
    /// The live query failed. Distinct from SR_ACTION_KILLED, which reports an
    /// ordinary termination.
    SR_ACTION_ERROR,
}

impl From<sdbAction> for Action {
    fn from(value: sdbAction) -> Self {
        match value {
            sdbAction::Create => Action::SR_ACTION_CREATE,
            sdbAction::Update => Action::SR_ACTION_UPDATE,
            sdbAction::Delete => Action::SR_ACTION_DELETE,
            sdbAction::Killed => Action::SR_ACTION_KILLED,
            sdbAction::Error => Action::SR_ACTION_ERROR,
        }
    }
}
