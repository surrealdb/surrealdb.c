use crate::{uuid::Uuid, value::Value};
use surrealdb::types::{Action as sdbAction, Notification as sdbNotification};

#[derive(Debug)]
#[repr(C)]
pub struct Notification {
    pub query_id: Uuid,
    pub action: Action,
    pub data: Value,
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
}

impl From<sdbAction> for Action {
    fn from(value: sdbAction) -> Self {
        match value {
            sdbAction::Create => Action::SR_ACTION_CREATE,
            sdbAction::Update => Action::SR_ACTION_UPDATE,
            sdbAction::Delete => Action::SR_ACTION_DELETE,
            sdbAction::Killed => Action::SR_ACTION_KILLED,
            _ => Action::SR_ACTION_KILLED,
        }
    }
}
