//! Bridges SpacetimeDB server events to the reactive store.

use std::collections::HashMap;
use leptos::prelude::*;
use stdb_client::{StdbStore, ElementData, CursorData};
use stdb_client::connection::{StdbConnection, ServerEvent};
use stdb_client::decode;
use spacetimedb_client_api_messages::websocket::v2::{
    TableUpdateRows, TransactionUpdate, ReducerOutcome,
};

/// Register an event handler that updates `store` signals in response to
/// server events from `conn`.
pub fn setup_event_handler(conn: &StdbConnection, store: StdbStore, selected_ids: RwSignal<Vec<u64>>) {
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
                    log::info!("SubscribeApplied: table={}", table_name);
                    match table_name {
                        "element" => {
                            store.elements.update(|elems: &mut HashMap<u64, ElementData>| {
                                for row_bytes in &table_rows.rows {
                                    match decode::decode_element(&row_bytes) {
                                        Ok(Some(elem)) => { elems.insert(elem.id, elem); }
                                        Ok(None) => {}
                                        Err(e) => log::error!("decode element: {e}"),
                                    }
                                }
                            });
                        }
                        "cursor" => {
                            store.cursors.update(|cursors: &mut HashMap<String, CursorData>| {
                                for row_bytes in &table_rows.rows {
                                    match decode::decode_cursor(&row_bytes) {
                                        Ok((key, cursor)) => { cursors.insert(key, cursor); }
                                        Err(e) => log::error!("decode cursor: {e}"),
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
            ServerEvent::TransactionUpdate { updates } => {
                process_transaction_update(&updates, &store, selected_ids);
            }
            ServerEvent::ReducerResult { request_id, result } => {
                log::debug!("Reducer result for request {}", request_id);
                // ReducerResult contains an embedded TransactionUpdate on success
                match result.result {
                    ReducerOutcome::Ok(ok) => {
                        process_transaction_update(&ok.transaction_update, &store, selected_ids);
                    }
                    ReducerOutcome::OkEmpty => {
                        // No updates
                    }
                    ReducerOutcome::Err(err_bytes) => {
                        log::error!("Reducer {} failed: {} bytes of error", request_id, err_bytes.len());
                    }
                    ReducerOutcome::InternalError(msg) => {
                        log::error!("Reducer {} internal error: {}", request_id, msg);
                    }
                }
            }
            ServerEvent::SubscriptionError { error, .. } => {
                log::error!("Subscription error: {}", error);
            }
            _ => {}
        }
    });
}

fn process_transaction_update(
    updates: &TransactionUpdate,
    store: &StdbStore,
    selected_ids: RwSignal<Vec<u64>>,
) {
    for query_set in updates.query_sets.iter() {
        for table_update in query_set.tables.iter() {
            let table_name = table_update.table_name.as_ref();

            for row_update in table_update.rows.iter() {
                match row_update {
                    TableUpdateRows::PersistentTable(persistent) => {
                        match table_name {
                            "element" => {
                                store.elements.update(|elems: &mut HashMap<u64, ElementData>| {
                                    // Process deletes
                                    for row_bytes in &persistent.deletes {
                                        match decode::decode_element_id(&row_bytes) {
                                            Ok(id) => { elems.remove(&id); }
                                            Err(e) => log::error!("decode element delete: {e}"),
                                        }
                                    }
                                    // Process inserts
                                    for row_bytes in &persistent.inserts {
                                        match decode::decode_element(&row_bytes) {
                                            Ok(Some(elem)) => {
                                                // Dedup: remove local temp elements that match
                                                let dup_key: Option<u64> = elems.iter()
                                                    .find(|(&id, e)| {
                                                        id != elem.id
                                                            && id > 1_000_000_000
                                                            && e.room_id == elem.room_id
                                                            && std::mem::discriminant(&e.kind) == std::mem::discriminant(&elem.kind)
                                                            && (e.x - elem.x).abs() < 1.0
                                                            && (e.y - elem.y).abs() < 1.0
                                                            && (e.width - elem.width).abs() < 1.0
                                                            && (e.height - elem.height).abs() < 1.0
                                                    })
                                                    .map(|(&id, _)| id);
                                                if let Some(old_id) = dup_key {
                                                    log::debug!("Dedup: {} -> {}", old_id, elem.id);
                                                    elems.remove(&old_id);
                                                    let new_id = elem.id;
                                                    selected_ids.update(|ids: &mut Vec<u64>| {
                                                        for id in ids.iter_mut() {
                                                            if *id == old_id { *id = new_id; }
                                                        }
                                                    });
                                                }
                                                elems.insert(elem.id, elem);
                                            }
                                            Ok(None) => {
                                                // deleted=true, remove
                                                if let Ok(id) = decode::decode_element_id(&row_bytes) {
                                                    elems.remove(&id);
                                                }
                                            }
                                            Err(e) => log::error!("decode element insert: {e}"),
                                        }
                                    }
                                });
                            }
                            "cursor" => {
                                store.cursors.update(|cursors: &mut HashMap<String, CursorData>| {
                                    for row_bytes in &persistent.deletes {
                                        if let Ok(key) = decode::decode_cursor_key(&row_bytes) {
                                            cursors.remove(&key);
                                        }
                                    }
                                    for row_bytes in &persistent.inserts {
                                        match decode::decode_cursor(&row_bytes) {
                                            Ok((key, cursor)) => { cursors.insert(key, cursor); }
                                            Err(e) => log::error!("decode cursor insert: {e}"),
                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
                    }
                    TableUpdateRows::EventTable(_) => {}
                }
            }
        }
    }
}
