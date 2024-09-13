#[repr(C)]
pub struct Options {
    pub strict: bool,
    pub query_timeout: u8,
    pub transaction_timeout: u8,
    // TODO: capabilities
}
