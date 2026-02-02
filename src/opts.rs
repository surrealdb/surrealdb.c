/// Connection options for SurrealDB
///
/// Configures various settings for the database connection.
#[repr(C)]
pub struct Options {
    /// Enable strict mode for queries
    pub strict: bool,
    /// Query timeout in seconds
    pub query_timeout: u8,
    /// Transaction timeout in seconds
    pub transaction_timeout: u8,
}
