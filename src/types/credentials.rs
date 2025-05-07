use std::{ffi::CString, fmt::Debug};
use std::fmt::Display;
use crate::{
    string::string_t,
    object::Object
};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum credentials_scope {
    ROOT,
    NAMESPACE,
    DATABASE,
    RECORD
}

impl Display for credentials_scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            credentials_scope::ROOT => write!(f, "root"),
            credentials_scope::NAMESPACE => write!(f, "namespace"),
            credentials_scope::DATABASE => write!(f, "database"),
            credentials_scope::RECORD => write!(f, "record"),
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
