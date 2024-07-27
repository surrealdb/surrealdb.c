use crate::{uuid::Uuid, value::Value};
use surrealdb::{sql, Notification as sdbNotification};

#[derive(Debug)]
#[repr(C)]
pub struct Notification {
    pub some: bool,
    pub query_id: Uuid,
    pub action: Action,
    pub data: Value,
}

impl From<sdbNotification<sql::Value>> for Notification {
    fn from(value: sdbNotification<sql::Value>) -> Self {
        Notification {
            some: true,
            query_id: value.query_id.into(),
            action: value.action.into(),
            data: value.data.into(),
        }
    }
}

impl From<Option<sdbNotification<sql::Value>>> for Notification {
    fn from(value: Option<sdbNotification<sql::Value>>) -> Self {
        match value {
            Some(n) => n.into(),
            None => Notification {
                some: false,
                query_id: Uuid([0; 16]),
                action: Action::SR_ACTION_CREATE,
                data: Default::default(),
            },
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
}

impl From<surrealdb::Action> for Action {
    fn from(value: surrealdb::Action) -> Self {
        match value {
            surrealdb::Action::Create => Action::SR_ACTION_CREATE,
            surrealdb::Action::Update => Action::SR_ACTION_UPDATE,
            surrealdb::Action::Delete => Action::SR_ACTION_DELETE,
            _ => todo!(),
        }
    }
}
