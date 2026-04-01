//! Custom SpacetimeDB WASM client connection module.
//!
//! This module implements a direct WebSocket client for SpacetimeDB's v2 BSATN
//! wire protocol, without relying on the SDK's codegen-based `SpacetimeModule`
//! trait. Instead, we use the lower-level `spacetimedb-client-api-messages` crate
//! for message types and `spacetimedb-lib::bsatn` for serialization.
//!
//! # Architecture
//!
//! - `StdbConnection` is the main handle, cheaply cloneable via `Arc`.
//! - On `connect()`, a WebSocket is opened with the `v2.bsatn.spacetimedb`
//!   subprotocol and a background task is spawned to read incoming messages.
//! - Outgoing messages (subscribe, call_reducer) are sent through an unbounded
//!   channel to a send loop.
//! - Incoming server messages are parsed (decompress + BSATN decode) and
//!   forwarded to a callback-based event system that downstream layers
//!   (e.g., Leptos signal integration in Task 3.2) can hook into.

use bytes::Bytes;
use futures::StreamExt;
use futures_channel::mpsc;
use spacetimedb_client_api_messages::websocket::{
    common::{self, QuerySetId},
    v2::{
        self, BIN_PROTOCOL, CallReducer, CallReducerFlags, ClientMessage, ServerMessage, Subscribe,
    },
};
use spacetimedb_lib::{bsatn, ConnectionId, Identity};
use std::borrow::Cow;
use std::io::Read as _;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use tokio_tungstenite_wasm::Message as WsMessage;
use wasm_bindgen_futures::spawn_local;

// ---------------------------------------------------------------------------
// Compression helpers (mirrors SDK's compression.rs)
// ---------------------------------------------------------------------------

fn decompress_server_message(raw: &[u8]) -> Result<Cow<'_, [u8]>, String> {
    match raw {
        [] => Err("empty server message".into()),
        [common::SERVER_MSG_COMPRESSION_TAG_NONE, bytes @ ..] => Ok(Cow::Borrowed(bytes)),
        [common::SERVER_MSG_COMPRESSION_TAG_BROTLI, bytes @ ..] => {
            let mut decompressed = Vec::new();
            brotli::BrotliDecompress(&mut &bytes[..], &mut decompressed)
                .map_err(|e| format!("brotli decompress: {e}"))?;
            Ok(Cow::Owned(decompressed))
        }
        [common::SERVER_MSG_COMPRESSION_TAG_GZIP, bytes @ ..] => {
            let mut decompressed = Vec::new();
            flate2::read::GzDecoder::new(bytes)
                .read_to_end(&mut decompressed)
                .map_err(|e| format!("gzip decompress: {e}"))?;
            Ok(Cow::Owned(decompressed))
        }
        [tag, ..] => Err(format!("unknown compression tag: {tag:#x}")),
    }
}

// ---------------------------------------------------------------------------
// Callbacks for downstream consumers (Task 3.2 Leptos signal integration)
// ---------------------------------------------------------------------------

/// A decoded server message ready for consumption by the signal layer.
#[derive(Debug)]
pub enum ServerEvent {
    /// Connection established; identity and token received.
    Connected {
        identity: Identity,
        connection_id: ConnectionId,
        token: String,
    },
    /// Initial subscription rows arrived.
    SubscribeApplied {
        query_set_id: QuerySetId,
        rows: v2::QueryRows,
    },
    /// A transaction committed — contains row inserts/deletes for subscribed tables.
    TransactionUpdate {
        updates: v2::TransactionUpdate,
    },
    /// Result of a reducer call we initiated.
    ReducerResult {
        request_id: u32,
        result: v2::ReducerResult,
    },
    /// Subscription error.
    SubscriptionError {
        query_set_id: QuerySetId,
        error: String,
    },
    /// One-off query response.
    OneOffQueryResult(v2::OneOffQueryResult),
    /// WebSocket disconnected.
    Disconnected {
        reason: String,
    },
}

/// Callback type for receiving server events.
pub type EventCallback = Box<dyn Fn(ServerEvent) + 'static>;

// ---------------------------------------------------------------------------
// Connection internals
// ---------------------------------------------------------------------------

struct StdbConnectionInner {
    /// Channel for sending outbound WebSocket messages (raw bytes).
    ws_send: mpsc::UnboundedSender<Vec<u8>>,

    /// Monotonically increasing request ID counter.
    next_request_id: AtomicU32,

    /// Monotonically increasing query set ID counter.
    next_query_set_id: AtomicU32,

    /// Event callback — set once and called from the recv loop.
    on_event: Mutex<Option<EventCallback>>,

    /// Stored identity (set after InitialConnection).
    identity: Mutex<Option<Identity>>,

    /// Stored connection ID (set after InitialConnection).
    connection_id: Mutex<Option<ConnectionId>>,

    /// Auth token received from server.
    token: Mutex<Option<String>>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Handle to an active SpacetimeDB connection.
///
/// Cheaply cloneable; all clones share the same underlying WebSocket.
#[derive(Clone)]
pub struct StdbConnection {
    inner: Arc<StdbConnectionInner>,
}

impl StdbConnection {
    /// Connect to a SpacetimeDB instance.
    ///
    /// `host` should be a WebSocket-capable URL, e.g. `ws://localhost:3000` or
    /// `wss://my-spacetimedb.example.com`.
    ///
    /// `db_name` is the name (or identity) of the database module.
    pub async fn connect(host: &str, db_name: &str) -> Result<Self, String> {
        // Build the WebSocket URI with the v2 BSATN subprotocol path.
        let uri = format!(
            "{host}/database/subscribe/{db_name}?client_address=0000000000000000"
        );

        log::info!("StdbConnection: connecting to {uri}");

        let ws_stream = tokio_tungstenite_wasm::connect_with_protocols(uri, &[BIN_PROTOCOL])
            .await
            .map_err(|e| format!("WebSocket connect failed: {e}"))?;

        let (ws_write, ws_read) = ws_stream.split();

        // Outbound channel: connection.rs callers push raw BSATN bytes here,
        // and the send loop forwards them to the WebSocket.
        let (send_tx, send_rx) = mpsc::unbounded::<Vec<u8>>();

        let inner = Arc::new(StdbConnectionInner {
            ws_send: send_tx,
            next_request_id: AtomicU32::new(1),
            next_query_set_id: AtomicU32::new(1),
            on_event: Mutex::new(None),
            identity: Mutex::new(None),
            connection_id: Mutex::new(None),
            token: Mutex::new(None),
        });

        // Spawn the send loop: forward outbound messages to the WebSocket sink.
        {
            use futures::SinkExt;
            let mut ws_write = ws_write;
            let mut send_rx = send_rx;
            spawn_local(async move {
                while let Some(msg) = send_rx.next().await {
                    if let Err(e) = ws_write.send(WsMessage::Binary(msg.into())).await {
                        log::error!("StdbConnection send error: {e}");
                        break;
                    }
                }
            });
        }

        // Spawn the receive loop: read WebSocket messages, decode, dispatch.
        {
            let inner = Arc::clone(&inner);
            let mut ws_read = ws_read;
            spawn_local(async move {
                while let Some(msg_result) = ws_read.next().await {
                    match msg_result {
                        Ok(WsMessage::Binary(data)) => {
                            if let Err(e) = handle_raw_message(&inner, &data) {
                                log::error!("StdbConnection message error: {e}");
                            }
                        }
                        Ok(WsMessage::Close(_)) => {
                            dispatch_event(
                                &inner,
                                ServerEvent::Disconnected {
                                    reason: "server closed connection".into(),
                                },
                            );
                            break;
                        }
                        Err(e) => {
                            dispatch_event(
                                &inner,
                                ServerEvent::Disconnected {
                                    reason: format!("WebSocket error: {e}"),
                                },
                            );
                            break;
                        }
                        // Ignore text/ping/pong frames.
                        _ => {}
                    }
                }
            });
        }

        Ok(StdbConnection { inner })
    }

    /// Register a callback to receive all server events.
    ///
    /// Only one callback can be active at a time; calling this replaces any
    /// previously registered callback.
    pub fn on_event(&self, callback: impl Fn(ServerEvent) + 'static) {
        let mut guard = self.inner.on_event.lock().unwrap();
        *guard = Some(Box::new(callback));
    }

    /// Subscribe to SQL queries for a room.
    ///
    /// Sends a `Subscribe` message with queries that fetch all rows relevant to
    /// the given `room_id` from the element, cursor, and undo_entry tables.
    /// Returns the `QuerySetId` assigned to this subscription.
    pub fn subscribe_room(&self, room_id: u64) -> QuerySetId {
        let query_set_id = QuerySetId::new(
            self.inner
                .next_query_set_id
                .fetch_add(1, Ordering::Relaxed),
        );
        let request_id = self.inner.next_request_id.fetch_add(1, Ordering::Relaxed);

        let queries: Box<[Box<str>]> = vec![
            format!("SELECT * FROM element WHERE room_id = {room_id}").into(),
            format!("SELECT * FROM cursor WHERE room_id = {room_id}").into(),
            format!("SELECT * FROM room WHERE id = {room_id}").into(),
        ]
        .into();

        let msg = ClientMessage::Subscribe(Subscribe {
            request_id,
            query_set_id,
            query_strings: queries,
        });

        self.send_client_message(&msg);
        query_set_id
    }

    /// Call a reducer by name with BSATN-encoded arguments.
    ///
    /// Returns the `request_id` assigned to this call, which can be correlated
    /// with the `ReducerResult` event.
    pub fn call_reducer(&self, name: &str, args: Vec<u8>) -> u32 {
        let request_id = self.inner.next_request_id.fetch_add(1, Ordering::Relaxed);

        let msg = ClientMessage::CallReducer(CallReducer {
            request_id,
            flags: CallReducerFlags::Default,
            reducer: name.into(),
            args: Bytes::from(args),
        });

        self.send_client_message(&msg);
        request_id
    }

    /// Get the identity assigned to this connection, if the InitialConnection
    /// message has been received.
    pub fn identity(&self) -> Option<Identity> {
        *self.inner.identity.lock().unwrap()
    }

    /// Get the connection ID assigned by the server, if available.
    pub fn connection_id(&self) -> Option<ConnectionId> {
        *self.inner.connection_id.lock().unwrap()
    }

    /// Get the auth token received from the server, if available.
    pub fn token(&self) -> Option<String> {
        self.inner.token.lock().unwrap().clone()
    }

    /// Check if the send channel is still open.
    pub fn is_active(&self) -> bool {
        !self.inner.ws_send.is_closed()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn send_client_message(&self, msg: &ClientMessage) {
        let encoded = bsatn::to_vec(msg).expect("BSATN encode ClientMessage");
        if let Err(e) = self.inner.ws_send.unbounded_send(encoded) {
            log::error!("StdbConnection: failed to queue message: {e}");
        }
    }
}

// ---------------------------------------------------------------------------
// Internal message handling
// ---------------------------------------------------------------------------

fn dispatch_event(inner: &StdbConnectionInner, event: ServerEvent) {
    let guard = inner.on_event.lock().unwrap();
    if let Some(cb) = guard.as_ref() {
        cb(event);
    }
}

fn handle_raw_message(inner: &StdbConnectionInner, raw: &[u8]) -> Result<(), String> {
    let decompressed = decompress_server_message(raw)?;
    let server_msg: ServerMessage =
        bsatn::from_slice(&decompressed).map_err(|e| format!("BSATN decode error: {e}"))?;

    match server_msg {
        ServerMessage::InitialConnection(init) => {
            *inner.identity.lock().unwrap() = Some(init.identity);
            *inner.connection_id.lock().unwrap() = Some(init.connection_id);
            *inner.token.lock().unwrap() = Some(init.token.to_string());
            dispatch_event(
                inner,
                ServerEvent::Connected {
                    identity: init.identity,
                    connection_id: init.connection_id,
                    token: init.token.to_string(),
                },
            );
        }
        ServerMessage::SubscribeApplied(applied) => {
            dispatch_event(
                inner,
                ServerEvent::SubscribeApplied {
                    query_set_id: applied.query_set_id,
                    rows: applied.rows,
                },
            );
        }
        ServerMessage::UnsubscribeApplied(_) => {
            // Not actively used yet; will be handled in Task 8.1 if needed.
        }
        ServerMessage::SubscriptionError(err) => {
            dispatch_event(
                inner,
                ServerEvent::SubscriptionError {
                    query_set_id: err.query_set_id,
                    error: err.error.to_string(),
                },
            );
        }
        ServerMessage::TransactionUpdate(update) => {
            dispatch_event(
                inner,
                ServerEvent::TransactionUpdate { updates: update },
            );
        }
        ServerMessage::OneOffQueryResult(result) => {
            dispatch_event(inner, ServerEvent::OneOffQueryResult(result));
        }
        ServerMessage::ReducerResult(result) => {
            dispatch_event(
                inner,
                ServerEvent::ReducerResult {
                    request_id: result.request_id,
                    result,
                },
            );
        }
        ServerMessage::ProcedureResult(_) => {
            // Not used in this application.
        }
    }

    Ok(())
}
