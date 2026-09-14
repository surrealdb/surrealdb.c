use std::fmt::Debug;
use std::fmt::Display;
use crate::{
    string::string_t,
};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum credentials_scope {
    // Prefixed like every other constant this header exports. Unprefixed
    // ROOT / NAMESPACE / DATABASE / RECORD were the only four of 52 that were
    // not, and they are exactly the kind of generic identifier a consumer's
    // own headers already define -- NAMESPACE and RECORD collide with macros
    // in clang's own headers.
    SR_SCOPE_ROOT,
    SR_SCOPE_NAMESPACE,
    SR_SCOPE_DATABASE,
    SR_SCOPE_RECORD,
}

impl Display for credentials_scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            credentials_scope::SR_SCOPE_ROOT => write!(f, "root"),
            credentials_scope::SR_SCOPE_NAMESPACE => write!(f, "namespace"),
            credentials_scope::SR_SCOPE_DATABASE => write!(f, "database"),
            credentials_scope::SR_SCOPE_RECORD => write!(f, "record"),
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct credentials {
    pub username: string_t,
    pub password: string_t,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct credentials_access {
    pub namespace: string_t,
    pub database: string_t,
    pub access: string_t,
}
