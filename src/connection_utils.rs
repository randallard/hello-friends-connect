use web_sys::{window, UrlSearchParams};
use crate::connect_component::{Connection, ConnectionStatus};
use uuid::Uuid;
use js_sys;
#[cfg(test)]
use wasm_bindgen_test::*;

pub fn extract_link_id_from_search(search: &str) -> Option<String> {
    if search.is_empty() {
        return None;
    }
    
    match UrlSearchParams::new_with_str(search) {
        Ok(params) => params.get("link"),
        Err(_) => None
    }
}

pub fn get_link_id_from_url() -> Option<String> {
    let window = window()?;
    let location = window.location();
    let search = location.search().ok()?;
    
    extract_link_id_from_search(&search)
}

/// Create a new connection with a unique link ID
pub fn create_connection(player_id: &str, name: &str) -> Option<Connection> {
    let link_id = Uuid::new_v4().to_string();
    
    let connection = Connection {
        id: Uuid::new_v4().to_string(),
        link_id,
        players: vec![player_id.to_string()],
        created_at: js_sys::Date::now() as i64,
        status: ConnectionStatus::Pending,
        expires_at: js_sys::Date::now() as i64 + 86400000, // 24 hours
    };
    
    // Save to localStorage
    let window = window()?;
    let storage = window.local_storage().ok()??;
    
    // Store the connection
    let conn_key = format!("connection-{}", connection.id);
    storage.set_item(&conn_key, &serde_json::to_string(&connection).ok()?).ok()?;
    
    // Store connection name
    storage.set_item(&format!("conn-name-{}", connection.id), name).ok()?;
    
    // Update connection index
    let index_key = "connection-index";
    let mut connection_ids = Vec::new();
    
    if let Ok(Some(index_json)) = storage.get_item(index_key) {
        if let Ok(mut ids) = serde_json::from_str::<Vec<String>>(&index_json) {
            connection_ids = ids;
        }
    }
    
    if !connection_ids.contains(&connection.id) {
        connection_ids.push(connection.id.clone());
        if let Ok(index_json) = serde_json::to_string(&connection_ids) {
            let _ = storage.set_item(index_key, &index_json);
        }
    }
    
    Some(connection)
}

/// Join an existing connection using its link ID
pub fn join_connection(link_id: &str, player_id: &str) -> Option<Connection> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    
    // In a real app, this would be an API call
    // For this demo, we'll just iterate through connection keys
    // Since we can't use storage.keys() directly, we'll check specific keys
    
    // Try to load from saved connections first
    if let Some(connections) = load_all_connections() {
        for mut connection in connections {
            if connection.link_id == link_id {
                // Add player to connection if not already present
                if !connection.players.contains(&player_id.to_string()) {
                    connection.players.push(player_id.to_string());
                    connection.status = ConnectionStatus::Active;
                    
                    // Update in localStorage
                    let conn_key = format!("connection-{}", connection.id);
                    storage.set_item(&conn_key, &serde_json::to_string(&connection).ok()?).ok()?;
                }
                
                return Some(connection);
            }
        }
    }
    
    None
}

/// Helper function to load all connections from localStorage
fn load_all_connections() -> Option<Vec<Connection>> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    
    // Since we can't enumerate keys, we'll use a known prefix pattern
    let mut connections = Vec::new();
    
    // Check for saved connections collection first
    if let Ok(Some(saved_json)) = storage.get_item("saved-connections") {
        if let Ok(saved_conns) = serde_json::from_str::<Vec<Connection>>(&saved_json) {
            connections.extend(saved_conns);
        }
    }
    
    // Try to find individual connections (we'd need to know the IDs in a real app)
    // For this demo, let's assume we know connection IDs are in a specific format
    // In a real app, you might store an index of IDs
    if let Ok(Some(index_json)) = storage.get_item("connection-index") {
        if let Ok(connection_ids) = serde_json::from_str::<Vec<String>>(&index_json) {
            for id in connection_ids {
                let conn_key = format!("connection-{}", id);
                if let Ok(Some(conn_json)) = storage.get_item(&conn_key) {
                    if let Ok(conn) = serde_json::from_str::<Connection>(&conn_json) {
                        connections.push(conn);
                    }
                }
            }
        }
    }
    
    Some(connections)
}

/// Save a connection to the list of saved connections
pub fn save_connection(connection: &Connection) -> Option<()> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    
    // Get existing saved connections
    let saved_key = "saved-connections";
    let existing = storage.get_item(saved_key).ok()?;
    
    let mut connections: Vec<Connection> = existing
        .map(|json| serde_json::from_str(&json).unwrap_or_else(|_| Vec::new()))
        .unwrap_or_else(|| Vec::new());
    
    // Add new connection if not already present
    if !connections.iter().any(|c| c.id == connection.id) {
        connections.push(connection.clone());
        
        // Save back to localStorage
        storage.set_item(saved_key, &serde_json::to_string(&connections).ok()?).ok()?;
        
        // Also make sure it's in the connection index
        let index_key = "connection-index";
        let mut connection_ids = Vec::new();
        
        if let Ok(Some(index_json)) = storage.get_item(index_key) {
            if let Ok(mut ids) = serde_json::from_str::<Vec<String>>(&index_json) {
                connection_ids = ids;
            }
        }
        
        if !connection_ids.contains(&connection.id) {
            connection_ids.push(connection.id.clone());
            if let Ok(index_json) = serde_json::to_string(&connection_ids) {
                let _ = storage.set_item(index_key, &index_json);
            }
        }
    }
    
    Some(())
}

/// Load all saved connections
pub fn load_saved_connections() -> Vec<Connection> {
    let window = match window() {
        Some(w) => w,
        None => return Vec::new(),
    };
    
    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        _ => return Vec::new(),
    };
    
    let saved_key = "saved-connections";
    let existing = match storage.get_item(saved_key) {
        Ok(Some(json)) => json,
        _ => return Vec::new(),
    };
    
    match serde_json::from_str(&existing) {
        Ok(connections) => connections,
        Err(_) => Vec::new(),
    }
}