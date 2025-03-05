use web_sys::{window, UrlSearchParams, Request, RequestInit, RequestMode, Response, console};
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;
use wasm_bindgen_futures::JsFuture;
use serde::{Serialize, Deserialize};
use js_sys::{Promise, JSON, Object};
use crate::connect_component::{Connection, ConnectionStatus};
use uuid::Uuid;

// API constants
const API_BASE: &str = "http://64.181.233.1/friends";

// Structs for API requests and responses
#[derive(Serialize, Deserialize)]
struct CreateConnectionRequest {
    player_id: String,
}

#[derive(Serialize, Deserialize)]
struct JoinConnectionRequest {
    player_id: String,
}

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

// Helper function for logging
fn console_log(msg: &str) {
    console::log_1(&JsValue::from_str(msg));
}

// Create a new connection with the API service
pub async fn create_connection(player_id: &str) -> Result<Connection, JsValue> {
    console_log(&format!("Creating connection for player: {}", player_id));
    
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    
    // Create the request body
    let request_data = CreateConnectionRequest {
        player_id: player_id.to_string(),
    };
    
    let request_json = JSON::stringify(&serde_wasm_bindgen::to_value(&request_data)?)?;
    opts.body(Some(&request_json));
    
    // Set headers
    let headers = web_sys::Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    headers.append("Accept", "application/json")?;
    opts.headers(&headers);
    
    // Create the request
    let url = format!("{}/connections", API_BASE);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    // Parse the response as JSON
    let json = JsFuture::from(resp.json()?).await?;
    let connection_data: Connection = serde_wasm_bindgen::from_value(json)?;
    
    console_log(&format!("Connection created with ID: {}", connection_data.id));
    
    Ok(connection_data)
}

// Join an existing connection using the API service
pub async fn join_connection(link_id: &str, player_id: &str) -> Result<Connection, JsValue> {
    console_log(&format!("Joining connection with link ID: {} for player: {}", link_id, player_id));
    
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    
    // Create the request body
    let request_data = JoinConnectionRequest {
        player_id: player_id.to_string(),
    };
    
    let request_json = JSON::stringify(&serde_wasm_bindgen::to_value(&request_data)?)?;
    opts.body(Some(&request_json));
    
    // Set headers
    let headers = web_sys::Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    opts.headers(&headers);
    
    // Create the request
    let url = format!("{}/connections/link/{}/join", API_BASE, link_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    // Parse the response as JSON
    let json = JsFuture::from(resp.json()?).await?;
    let connection_data: Connection = serde_wasm_bindgen::from_value(json)?;
    
    console_log(&format!("Joined connection with ID: {}", connection_data.id));
    
    Ok(connection_data)
}

// Helper function to save connection name in localStorage
fn save_connection_name(connection_id: &str, name: &str) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let key = format!("conn-name-{}", connection_id);
            let _ = storage.set_item(&key, name);
            
            // Also save in the connection names map
            if let Ok(Some(names_json)) = storage.get_item("connection-names") {
                if let Ok(mut names_map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&names_json) {
                    names_map.insert(connection_id.to_string(), serde_json::Value::String(name.to_string()));
                    if let Ok(updated_json) = serde_json::to_string(&names_map) {
                        let _ = storage.set_item("connection-names", &updated_json);
                    }
                }
            } else {
                // Create new map
                let mut names_map = serde_json::Map::new();
                names_map.insert(connection_id.to_string(), serde_json::Value::String(name.to_string()));
                if let Ok(json) = serde_json::to_string(&names_map) {
                    let _ = storage.set_item("connection-names", &json);
                }
            }
        }
    }
}

// Get a connection by its link ID
pub async fn get_connection_by_link_id(link_id: &str) -> Result<Connection, JsValue> {
    console_log(&format!("Getting connection with link ID: {}", link_id));
    
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    
    // Set headers
    let headers = web_sys::Headers::new()?;
    headers.append("Accept", "application/json")?;
    opts.headers(&headers);
    
    // Create the request
    let url = format!("{}/connections/link/{}", API_BASE, link_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        if status == 404 {
            return Err(JsValue::from_str("Connection not found"));
        }
        let status_text = resp.status_text();
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    // Parse the response as JSON
    let json = JsFuture::from(resp.json()?).await?;
    let connection_data: Connection = serde_wasm_bindgen::from_value(json)?;
    
    console_log(&format!("Retrieved connection with ID: {}", connection_data.id));
    
    Ok(connection_data)
}

// Save a connection in local storage for later reference
pub fn save_connection_to_local_storage(connection: &Connection, friendly_name: &str) -> Option<()> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    
    // Create a structure to save
    #[derive(Serialize, Deserialize)]
    struct SavedConnection {
        id: String,
        link_id: String,
        friendly_name: String,
        created_at: i64,
    }
    
    let saved = SavedConnection {
        id: connection.id.clone(),
        link_id: connection.link_id.clone(),
        friendly_name: friendly_name.to_string(),
        created_at: connection.created_at,
    };
    
    // Save in saved connections collection
    let saved_key = "saved-connections";
    let existing = storage.get_item(saved_key).ok()?;
    
    let mut connections: Vec<SavedConnection> = existing
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_else(|| Vec::new());
    
    // Add if not already present
    if !connections.iter().any(|c| c.id == connection.id) {
        connections.push(saved);
        
        // Save back to localStorage
        let json = serde_json::to_string(&connections).ok()?;
        storage.set_item(saved_key, &json).ok()?;
    }
    
    Some(())
}

// Load saved connections from local storage
pub fn load_saved_connections() -> Vec<serde_json::Value> {
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

// Poll for notifications
pub async fn poll_notifications(player_id: &str) -> Result<Vec<String>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    
    // Set headers
    let headers = web_sys::Headers::new()?;
    headers.append("Accept", "application/json")?;
    opts.headers(&headers);
    
    // Create the request
    let url = format!("{}/players/{}/notifications", API_BASE, player_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    // Parse the response as JSON
    let json = JsFuture::from(resp.json()?).await?;
    let notifications: Vec<String> = serde_wasm_bindgen::from_value(json)?;
    
    if !notifications.is_empty() {
        // Acknowledge notifications
        acknowledge_notifications(player_id).await?;
    }
    
    Ok(notifications)
}

// Acknowledge notifications
async fn acknowledge_notifications(player_id: &str) -> Result<(), JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    
    // Set headers
    let headers = web_sys::Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    opts.headers(&headers);
    
    // Create the request
    let url = format!("{}/players/{}/notifications/ack", API_BASE, player_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    Ok(())
}

// Send a message to a connection
pub async fn send_message(connection_id: &str, player_id: &str, content: &str) -> Result<(), JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    
    // Create the request body
    #[derive(Serialize)]
    struct MessageRequest {
        player_id: String,
        content: String,
    }
    
    let request_data = MessageRequest {
        player_id: player_id.to_string(),
        content: content.to_string(),
    };
    
    let request_json = JSON::stringify(&serde_wasm_bindgen::to_value(&request_data)?)?;
    opts.body(Some(&request_json));
    
    // Set headers
    let headers = web_sys::Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    opts.headers(&headers);
    
    // Create the request
    let url = format!("{}/connections/{}/messages", API_BASE, connection_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    Ok(())
}