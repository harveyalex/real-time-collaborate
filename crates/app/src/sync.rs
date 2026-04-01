//! Bridges SpacetimeDB server events to the reactive store.
//!
//! The `setup_event_handler` function registers a callback on the connection
//! that translates `ServerEvent`s into updates on the `StdbStore` signals.
//! BSATN row decoding is performed via `stdb_client::decode`.

use std::collections::HashMap;
use leptos::prelude::{Set, Update};
use stdb_client::{StdbStore, ElementData, CursorData};
use stdb_client::connection::{StdbConnection, ServerEvent};
use stdb_client::decode;
use spacetimedb_client_api_messages::websocket::v2::TableUpdateRows;

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
                for table_rows in rows.tables.iter() {
                    let table_name = table_rows.table.as_ref();
                    log::info!(
                        "Subscription data for table: {:?} ({} rows)",
                        table_name,
                        table_rows.rows.into_iter().count(),
                    );

                    match table_name {
                        "element" => {
                            store.elements.update(|elems: &mut HashMap<u64, ElementData>| {
                                for row_bytes in &table_rows.rows {
                                    match decode::decode_element(&row_bytes) {
                                        Ok(Some(elem)) => {
                                            elems.insert(elem.id, elem);
                                        }
                                        Ok(None) => {
                                            // deleted element, skip
                                        }
                                        Err(e) => {
                                            log::error!("Failed to decode element row: {e}");
                                        }
                                    }
                                }
                            });
                        }
                        "cursor" => {
                            store.cursors.update(|cursors: &mut HashMap<String, CursorData>| {
                                for row_bytes in &table_rows.rows {
                                    match decode::decode_cursor(&row_bytes) {
                                        Ok((key, cursor)) => {
                                            cursors.insert(key, cursor);
                                        }
                                        Err(e) => {
                                            log::error!("Failed to decode cursor row: {e}");
                                        }
                                    }
                                }
                            });
                        }
                        _ => {
                            log::debug!("Ignoring subscription data for table: {table_name}");
                        }
                    }
                }
            }
            ServerEvent::TransactionUpdate { updates } => {
                for query_set in updates.query_sets.iter() {
                    for table_update in query_set.tables.iter() {
                        let table_name = table_update.table_name.as_ref();

                        for row_update in table_update.rows.iter() {
                            match row_update {
                                TableUpdateRows::PersistentTable(persistent) => {
                                    // Process deletes first, then inserts
                                    match table_name {
                                        "element" => {
                                            store.elements.update(|elems: &mut HashMap<u64, ElementData>| {
                                                for row_bytes in &persistent.deletes {
                                                    match decode::decode_element_id(&row_bytes) {
                                                        Ok(id) => {
                                                            elems.remove(&id);
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to decode element delete key: {e}");
                                                        }
                                                    }
                                                }
                                                for row_bytes in &persistent.inserts {
                                                    match decode::decode_element(&row_bytes) {
                                                        Ok(Some(elem)) => {
                                                            // Dedup: remove any local temp element that matches the
                                                            // server element by (room_id, kind, x, y, width, height).
                                                            let dup_key: Option<u64> = elems.iter()
                                                                .find(|(&id, e)| {
                                                                    id != elem.id
                                                                        && e.room_id == elem.room_id
                                                                        && std::mem::discriminant(&e.kind) == std::mem::discriminant(&elem.kind)
                                                                        && e.x == elem.x
                                                                        && e.y == elem.y
                                                                        && e.width == elem.width
                                                                        && e.height == elem.height
                                                                })
                                                                .map(|(&id, _)| id);
                                                            if let Some(old_id) = dup_key {
                                                                log::debug!("Dedup: removing temp element {} in favour of server element {}", old_id, elem.id);
                                                                elems.remove(&old_id);
                                                            }
                                                            elems.insert(elem.id, elem);
                                                        }
                                                        Ok(None) => {
                                                            // deleted flag set, remove if present
                                                            if let Ok(id) = decode::decode_element_id(&row_bytes) {
                                                                elems.remove(&id);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to decode element insert: {e}");
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        "cursor" => {
                                            store.cursors.update(|cursors: &mut HashMap<String, CursorData>| {
                                                for row_bytes in &persistent.deletes {
                                                    match decode::decode_cursor_key(&row_bytes) {
                                                        Ok(key) => {
                                                            cursors.remove(&key);
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to decode cursor delete key: {e}");
                                                        }
                                                    }
                                                }
                                                for row_bytes in &persistent.inserts {
                                                    match decode::decode_cursor(&row_bytes) {
                                                        Ok((key, cursor)) => {
                                                            cursors.insert(key, cursor);
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to decode cursor insert: {e}");
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        _ => {
                                            log::debug!("Ignoring transaction update for table: {table_name}");
                                        }
                                    }
                                }
                                TableUpdateRows::EventTable(_) => {
                                    // Event tables are not used in this application
                                }
                            }
                        }
                    }
                }
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
