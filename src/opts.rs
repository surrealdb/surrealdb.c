#![allow(non_camel_case_types)]

//! Connection options and the capability sandbox.

use std::collections::HashSet;
use std::ffi::{c_char, c_int, CStr};
use std::hash::Hash;

use surrealdb_core::dbs::capabilities::{
    ArbitraryQueryTarget, Capabilities, EvalQueryTarget, ExperimentalTarget, FuncTarget,
    MethodTarget, NetTarget, RouteTarget, Targets,
};

/// A three-state switch.
///
/// Zero means "leave the library default alone", so a zero-initialised
/// `Options` changes nothing -- which is the only safe reading for a
/// sandbox.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toggle {
    /// Keep SurrealDB's default.
    SR_TOGGLE_DEFAULT = 0,
    SR_TOGGLE_ON = 1,
    SR_TOGGLE_OFF = 2,
}

/// Which members of a capability set are allowed or denied.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetMode {
    /// Keep SurrealDB's default for this set.
    SR_TARGET_DEFAULT = 0,
    /// Empty set.
    SR_TARGET_NONE = 1,
    /// Exactly the names listed in `items`.
    SR_TARGET_SOME = 2,
    /// Everything.
    SR_TARGET_ALL = 3,
}

/// A capability target set.
///
/// `items` is read only when `mode` is `SR_TARGET_SOME`, and holds `len`
/// null-terminated strings. The accepted spellings are SurrealDB's own -- for
/// example `"http::get"` for a function, `"1.2.3.4/8"` or `"example.com:80"`
/// for a network target, `"select"` for an RPC method, `"gql"` for an
/// experimental feature. An unrecognised name is reported as an error rather
/// than ignored, so a typo cannot silently widen or narrow the sandbox.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TargetSet {
    pub mode: TargetMode,
    pub items: *const *const c_char,
    pub len: c_int,
}

/// The capability sandbox.
///
/// Every field defaults to "leave SurrealDB's default alone", so
/// `CapabilitySet caps = {0};` is a no-op. Where both an allow and a deny
/// set are given, deny wins -- that is SurrealDB's rule, not this library's.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CapabilitySet {
    /// Embedded scripting functions. Off by default.
    pub scripting: Toggle,
    /// Unauthenticated access. Off by default.
    pub guest_access: Toggle,
    /// Live query notifications. On by default.
    pub live_query_notifications: Toggle,

    /// Built-in and custom functions. Allowed by default.
    pub allow_functions: TargetSet,
    pub deny_functions: TargetSet,

    /// Outbound network access. **Denied by default**; this is the field to set
    /// before anything can reach the network.
    pub allow_network: TargetSet,
    pub deny_network: TargetSet,

    /// RPC methods. Allowed by default.
    pub allow_rpc_methods: TargetSet,
    pub deny_rpc_methods: TargetSet,

    /// HTTP routes. Allowed by default.
    pub allow_http_routes: TargetSet,
    pub deny_http_routes: TargetSet,

    /// Experimental features -- `"gql"`, `"files"`, `"surrealism"`. Denied by
    /// default. `gql` is required by the `gql`/`graphql` RPC methods, and
    /// `files` by `file://` values (`sr_value_file`).
    pub allow_experimental: TargetSet,
    pub deny_experimental: TargetSet,

    /// Which authentication levels may run arbitrary queries.
    /// Names are `"guest"`, `"record"`, `"system"`.
    pub allow_arbitrary_query: TargetSet,
    pub deny_arbitrary_query: TargetSet,

    /// Which authentication levels may run `eval`-style queries.
    pub allow_eval_query: TargetSet,
    pub deny_eval_query: TargetSet,
}

/// Connection options.
///
/// Zero-initialise and set only what you need: every field's zero value means
/// "SurrealDB's default".
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Query timeout in seconds. Zero leaves the default in place.
    pub query_timeout: u8,
    /// Transaction timeout in seconds. Zero leaves the default in place.
    pub transaction_timeout: u8,
    /// The capability sandbox.
    pub capabilities: CapabilitySet,
    /// Directory in which to persist RPC sessions, or null to disable.
    ///
    /// When set, a session attached to an RPC context is written here as JSON
    /// and is rehydrated on demand, so sessions survive the context being torn
    /// down and rebuilt. Ignored by `sr_connect_with_options`, which has no
    /// session map of its own.
    ///
    /// # These files hold credentials
    ///
    /// A session is serialised whole, and a session carries its authentication
    /// token, its record-authentication data and its variables. They are
    /// written as plain JSON: nothing here is encrypted or obfuscated, and
    /// anything that can read the file can replay the session.
    ///
    /// The library restricts the directory and its files to the current user
    /// (0700 / 0600 on unix; on other platforms the inherited ACL is all there
    /// is). That is sufficient on a server, which is what upstream built this
    /// for. It is not a disk-encryption scheme, so treat the directory as
    /// credential storage: keep it off shared or synced volumes, and think hard
    /// before enabling it on hardware the end user controls.
    pub session_dir: *const c_char,
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

/// Turn a C target set into a `Targets<T>`, or `None` to leave the default.
///
/// # Safety
///
/// `items` must hold `len` valid C strings when `mode` is `SR_TARGET_SOME`.
fn to_targets<T>(t: &TargetSet, what: &str) -> Result<Option<Targets<T>>, String>
where
    T: std::str::FromStr + Hash + Eq + Ord,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match t.mode {
        TargetMode::SR_TARGET_DEFAULT => Ok(None),
        TargetMode::SR_TARGET_NONE => Ok(Some(Targets::None)),
        TargetMode::SR_TARGET_ALL => Ok(Some(Targets::All)),
        TargetMode::SR_TARGET_SOME => {
            if t.items.is_null() || t.len <= 0 {
                // An explicitly empty SOME is the same as NONE, and saying so
                // is friendlier than rejecting it.
                return Ok(Some(Targets::None));
            }
            let mut set: HashSet<T> = HashSet::new();
            for i in 0..t.len as usize {
                let raw = unsafe { *t.items.add(i) };
                let Some(s) = cstr(raw) else {
                    return Err(format!("{what}: entry {i} is null or not valid UTF-8"));
                };
                match s.parse::<T>() {
                    Ok(v) => {
                        set.insert(v);
                    }
                    // Reported rather than skipped: silently dropping a name
                    // the caller meant to allow or deny would change the
                    // sandbox without telling anyone.
                    Err(e) => return Err(format!("{what}: cannot parse '{s}': {e}")),
                }
            }
            Ok(Some(Targets::Some(set)))
        }
    }
}

fn apply_toggle(current: bool, t: Toggle) -> bool {
    match t {
        Toggle::SR_TOGGLE_DEFAULT => current,
        Toggle::SR_TOGGLE_ON => true,
        Toggle::SR_TOGGLE_OFF => false,
    }
}

impl CapabilitySet {
    /// Build a `Capabilities` from this description, or `None` if nothing was
    /// set (in which case the caller should not touch the builder at all).
    pub fn to_capabilities(&self) -> Result<Option<Capabilities>, String> {
        let mut touched = false;
        let d = Capabilities::default();
        let mut caps = Capabilities::default();

        let scripting = apply_toggle(false, self.scripting);
        let guest = apply_toggle(false, self.guest_access);
        let lqn = apply_toggle(true, self.live_query_notifications);
        if self.scripting != Toggle::SR_TOGGLE_DEFAULT
            || self.guest_access != Toggle::SR_TOGGLE_DEFAULT
            || self.live_query_notifications != Toggle::SR_TOGGLE_DEFAULT
        {
            touched = true;
        }
        caps = caps
            .with_scripting(scripting)
            .with_guest_access(guest)
            .with_live_query_notifications(lqn);

        macro_rules! set {
            ($field:ident, $setter:ident, $ty:ty, $label:expr) => {
                if let Some(t) = to_targets::<$ty>(&self.$field, $label)? {
                    caps = caps.$setter(t);
                    touched = true;
                }
            };
        }

        set!(allow_functions, with_functions, FuncTarget, "allow_functions");
        set!(deny_functions, without_functions, FuncTarget, "deny_functions");
        set!(allow_network, with_network_targets, NetTarget, "allow_network");
        set!(deny_network, without_network_targets, NetTarget, "deny_network");
        set!(allow_rpc_methods, with_rpc_methods, MethodTarget, "allow_rpc_methods");
        set!(deny_rpc_methods, without_rpc_methods, MethodTarget, "deny_rpc_methods");
        set!(allow_http_routes, with_http_routes, RouteTarget, "allow_http_routes");
        set!(deny_http_routes, without_http_routes, RouteTarget, "deny_http_routes");
        set!(allow_experimental, with_experimental, ExperimentalTarget, "allow_experimental");
        set!(deny_experimental, without_experimental, ExperimentalTarget, "deny_experimental");
        set!(
            allow_arbitrary_query,
            with_arbitrary_query,
            ArbitraryQueryTarget,
            "allow_arbitrary_query"
        );
        set!(
            deny_arbitrary_query,
            without_arbitrary_query,
            ArbitraryQueryTarget,
            "deny_arbitrary_query"
        );
        set!(allow_eval_query, with_eval_query, EvalQueryTarget, "allow_eval_query");
        set!(deny_eval_query, without_eval_query, EvalQueryTarget, "deny_eval_query");

        let _ = d;
        Ok(if touched { Some(caps) } else { None })
    }
}

impl Options {
    pub fn session_dir_path(&self) -> Option<std::path::PathBuf> {
        cstr(self.session_dir).map(std::path::PathBuf::from)
    }
}
