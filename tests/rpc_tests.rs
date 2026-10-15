//! Tests for the CBOR RPC API (`sr_surreal_rpc_*`), which surrealdb.rb's embedded mode uses.
//!
//! Requests and responses are encoded with SurrealDB's own CBOR codec, so these tests check
//! that tagged values (record ids, datetimes, uuids, ...) survive the round trip and that
//! responses use the server's `{ id, result | error }` envelope.

use std::ffi::{c_int, CString};
use std::ptr;
use std::sync::mpsc;
use std::time::Duration;

use surrealdb::types::{Array, Datetime, Object, RecordId, RecordIdKey, Value};
use surrealdb_c::opts::Options;
use surrealdb_c::rpc::SurrealRpc;
use surrealdb_c::stream::RpcStream;
use surrealdb_c::string::string_t;
use surrealdb_c::types::bytes::Bytes;
use surrealdb_core::rpc::format::cbor;

struct Rpc(*mut SurrealRpc);

impl Rpc {
    fn new() -> Self {
        let mut err = string_t::null();
        let mut ctx: *mut SurrealRpc = ptr::null_mut();
        let endpoint = CString::new("memory").unwrap();
        let opts = Options {
            strict: false,
            query_timeout: 0,
            transaction_timeout: 0,
        };
        let res = SurrealRpc::new(&mut err, &mut ctx, endpoint.as_ptr(), opts);
        assert!(res >= 0, "rpc_new failed: {err:?}");
        Rpc(ctx)
    }

    /// Send raw request bytes and decode the response envelope
    fn send_raw(&self, bytes: &[u8]) -> Object {
        let mut err = string_t::null();
        let mut out: *mut u8 = ptr::null_mut();
        let len = SurrealRpc::execute(
            unsafe { &*self.0 },
            &mut err,
            &mut out,
            bytes.as_ptr(),
            bytes.len() as c_int,
        );
        assert!(len > 0, "rpc_execute failed ({len}): {err:?}");
        let response = unsafe { std::slice::from_raw_parts(out, len as usize) }.to_vec();
        Bytes::free_byte_arr(out, len);
        match cbor::decode(&response, 100).expect("response is valid CBOR") {
            Value::Object(obj) => obj,
            other => panic!("expected a response object, got {other:?}"),
        }
    }

    fn send(&self, id: i64, method: &str, params: Vec<Value>) -> Object {
        let mut req = Object::new();
        req.insert("id".to_string(), Value::from_t(id));
        req.insert("method".to_string(), Value::String(method.to_string()));
        req.insert("params".to_string(), Value::Array(Array::from(params)));
        let bytes = cbor::encode(Value::Object(req)).unwrap();
        let response = self.send_raw(&bytes);
        assert_eq!(response.get("id"), Some(&Value::from_t(id)), "id is echoed");
        response
    }

    /// Send a request and return its result, failing on an error response
    fn call(&self, method: &str, params: Vec<Value>) -> Value {
        let mut response = self.send(7, method, params);
        if let Some(err) = response.get("error") {
            panic!("{method} returned an error: {err:?}");
        }
        response.remove("result").expect("response has a result")
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        SurrealRpc::rpc_free(self.0);
    }
}

fn s(v: &str) -> Value {
    Value::String(v.to_string())
}

fn use_test_db(rpc: &Rpc) {
    rpc.call("use", vec![s("test"), s("test")]);
}

/// Unwrap the result of statement `index` from a `query` response
fn statement_result(result: &Value, index: usize) -> &Value {
    let Value::Array(statements) = result else {
        panic!("query result should be an array, got {result:?}");
    };
    let Value::Object(statement) = &statements[index] else {
        panic!("statement result should be an object");
    };
    assert_eq!(statement.get("status"), Some(&s("OK")), "{statement:?}");
    statement.get("result").expect("statement has a result")
}

#[test]
fn query_returns_tagged_results() {
    let rpc = Rpc::new();
    use_test_db(&rpc);

    let result = rpc.call(
        "query",
        vec![s(
            "CREATE person:alice SET born = d'1990-05-01T00:00:00Z', \
             token = u'0190a6d8-4b6c-7a3e-9b1f-1c2d3e4f5a6b', tags = [1, 2.5, 'x']; \
             SELECT * FROM person;",
        )],
    );

    let Value::Array(rows) = statement_result(&result, 1) else {
        panic!("SELECT should return an array");
    };
    let Value::Object(alice) = &rows[0] else {
        panic!("row should be an object");
    };
    assert_eq!(
        alice.get("id"),
        Some(&Value::RecordId(RecordId::new("person", "alice"))),
        "record id keeps its tag"
    );
    assert!(matches!(alice.get("born"), Some(Value::Datetime(_))), "datetime keeps its tag");
    assert!(matches!(alice.get("token"), Some(Value::Uuid(_))), "uuid keeps its tag");
}

#[test]
fn tagged_params_are_decoded() {
    let rpc = Rpc::new();
    use_test_db(&rpc);
    rpc.call("query", vec![s("CREATE person:1 SET name = 'one'")]);

    // A record id parameter must arrive as a record id, not as a string or a table name
    let mut vars = Object::new();
    vars.insert("rid".to_string(), Value::RecordId(RecordId::new("person", 1i64)));
    let when: Datetime = "2024-01-02T03:04:05Z".parse().unwrap();
    vars.insert("when".to_string(), Value::Datetime(when.clone()));
    let result = rpc.call(
        "query",
        vec![s("SELECT VALUE name FROM $rid; RETURN $when;"), Value::Object(vars)],
    );
    assert_eq!(statement_result(&result, 0), &Value::Array(Array::from(vec![s("one")])));
    assert_eq!(statement_result(&result, 1), &Value::Datetime(when));

    // CRUD methods take tagged targets too
    let selected = rpc.call("select", vec![Value::RecordId(RecordId::new("person", 1i64))]);
    let Value::Object(obj) = selected else {
        panic!("select of one record should return an object, got {selected:?}");
    };
    assert_eq!(obj.get("id"), Some(&Value::RecordId(RecordId::new("person", 1i64))));
    assert!(matches!(
        obj.get("id"),
        Some(Value::RecordId(RecordId {
            key: RecordIdKey::Number(1),
            ..
        }))
    ));
}

#[test]
fn errors_use_the_error_envelope() {
    let rpc = Rpc::new();

    let response = rpc.send(3, "no_such_method", vec![]);
    assert!(response.get("result").is_none());
    assert!(matches!(response.get("error"), Some(Value::Object(_))), "{response:?}");

    // A request that can't be decoded still gets an error response
    let response = rpc.send_raw(&[0xff]);
    assert!(matches!(response.get("error"), Some(Value::Object(_))), "{response:?}");
}

#[test]
fn ping_and_version() {
    let rpc = Rpc::new();
    assert_eq!(rpc.call("ping", vec![]), Value::None);
    assert!(matches!(rpc.call("version", vec![]), Value::String(_)));
}

#[test]
fn notifications_are_tagged() {
    let rpc = Rpc::new();
    use_test_db(&rpc);

    let mut err = string_t::null();
    let mut stream: *mut RpcStream = ptr::null_mut();
    let res = SurrealRpc::notifications(unsafe { &*rpc.0 }, &mut err, &mut stream);
    assert!(res >= 0, "notifications failed: {err:?}");

    let result = rpc.call("query", vec![s("DEFINE TABLE person; LIVE SELECT * FROM person")]);
    let Value::Uuid(live_id) = statement_result(&result, 1).clone() else {
        panic!("LIVE SELECT should return a uuid");
    };

    // Read the notification on another thread so a missing notification fails instead of hanging
    let stream_addr = stream as usize;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut out: *mut u8 = ptr::null_mut();
        let len = RpcStream::next(unsafe { &mut *(stream_addr as *mut RpcStream) }, &mut out);
        let bytes = if len > 0 {
            let bytes = unsafe { std::slice::from_raw_parts(out, len as usize) }.to_vec();
            Bytes::free_byte_arr(out, len);
            bytes
        } else {
            Vec::new()
        };
        tx.send(bytes).ok();
    });

    rpc.call("query", vec![s("CREATE person:bob SET name = 'bob'")]);

    let bytes = rx.recv_timeout(Duration::from_secs(10)).expect("notification arrives");
    let Value::Object(mut envelope) = cbor::decode(&bytes, 100).unwrap() else {
        panic!("notification should be an object");
    };
    let Some(Value::Object(notification)) = envelope.remove("result") else {
        panic!("notification should be wrapped in result: {envelope:?}");
    };
    assert_eq!(notification.get("id"), Some(&Value::Uuid(live_id)));
    assert_eq!(notification.get("action"), Some(&s("CREATE")));
    assert_eq!(
        notification.get("record"),
        Some(&Value::RecordId(RecordId::new("person", "bob")))
    );

    RpcStream::free(stream);
}
