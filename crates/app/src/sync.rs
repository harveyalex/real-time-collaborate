//! Bridges SpacetimeDB server events to the reactive store.
//!
//! The `setup_event_handler` function registers a callback on the connection
//! that translates `ServerEvent`s into updates on the `StdbStore` signals.
//! Full BSATN row decoding is left as a TODO — for the PoC, only connection
//! status and identity are wired up; element creation still works locally.

use leptos::prelude::Set;
use stdb_client::StdbStore;
use stdb_client::connection::{StdbConnection, ServerEvent};

/// Register an event handler that updates `store` signals in response to
/// server events from `conn`.
pub fn setup_event_handler(conn: &StdbConnection, store: StdbStore) {
    conn.on_event(move |event| {
        match event {
            ServerEvent::Connected { identity, .. } => {
                store.connected.set(true);
                store.my_identity.set(Some(identity.to_string()));
                log::info!("Connected as {}", identity);
            }
            ServerEvent::Disconnected { reason } => {
                store.connected.set(false);
                log::warn!("Disconnected: {}", reason);
            }
            ServerEvent::SubscribeApplied { rows, .. } => {
                // TODO: decode BSATN rows from rows.tables into ElementData/CursorData
                // For now, log the table names we received
                for table_rows in rows.tables.iter() {
                    log::info!(
                        "Subscription data for table: {:?}",
                        table_rows.table
                    );
                }
            }
            ServerEvent::TransactionUpdate { .. } => {
                // TODO: decode row inserts/deletes from updates
                log::debug!("Transaction update received");
            }
            ServerEvent::ReducerResult { request_id, .. } => {
                log::debug!("Reducer result for request {}", request_id);
            }
            ServerEvent::SubscriptionError { error, .. } => {
                log::error!("Subscription error: {}", error);
            }
            _ => {}
        }
    });
}
