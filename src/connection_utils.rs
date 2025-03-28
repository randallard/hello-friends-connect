use web_sys::{window, UrlSearchParams, Request, RequestInit, RequestMode, Response, console};
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;
use wasm_bindgen_futures::JsFuture;
use serde::{Serialize, Deserialize};
use js_sys::{Promise, JSON, Object};
use crate::connect_component::{Connection, ConnectionStatus};
use crate::config::get_config; 
use uuid::Uuid;
use std::pin;
use std::future::Future;

#[cfg(test)]
use std::sync::Once;

#[cfg(test)]
static INIT: Once = Once::new();

#[cfg(test)]
pub static mut MOCK_CREATE_CONNECTION: Option<Box<dyn Fn(&str) -> pin::Pin<Box<dyn Future<Output = Result<Connection, JsValue>>>>>> = None;

#[cfg(test)]
pub static mut MOCK_JOIN_CONNECTION: Option<Box<dyn Fn(&str, &str) -> pin::Pin<Box<dyn Future<Output = Result<Connection, JsValue>>>>>> = None;

pub fn get_api_base() -> String {
    get_config().api_base_url.clone()
}

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

pub async fn create_connection(player_id: &str) -> Result<Connection, JsValue> {
    #[cfg(test)]
    {
        if let Some(mock) = unsafe { MOCK_CREATE_CONNECTION.as_ref() } {
            return mock(player_id).await;
        }
    }
    
    // Real implementation follows...
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
    let url = format!("{}/connections", get_api_base());
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
    
    // Check if response has a connection field
    if let Ok(response_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(json.clone()) {
        if let Some(connection_obj) = response_obj.get("connection") {
            // New format with nested connection object
            let connection_str = serde_json::to_string(connection_obj)
                .map_err(|e| JsValue::from_str(&format!("Error serializing connection: {:?}", e)))?;
            
            let connection_js = js_sys::JSON::parse(&connection_str)
                .map_err(|e| JsValue::from_str(&format!("Error parsing connection JSON: {:?}", e)))?;
            
            let connection_data: Connection = serde_wasm_bindgen::from_value(connection_js)?;
            
            // Optionally log the WebSocket URL
            if let Some(ws_url) = response_obj.get("websocket_url") {
                if let Some(url_str) = ws_url.as_str() {
                    console_log(&format!("Server provided WebSocket URL: {}", url_str));
                }
            }
            
            console_log(&format!("Connection created with ID: {}", connection_data.id));
            return Ok(connection_data);
        }
    }
    
    // Fallback to direct parsing (old format)
    let connection_data: Connection = serde_wasm_bindgen::from_value(json)?;
    console_log(&format!("Connection created with ID: {}", connection_data.id));
    Ok(connection_data)
}

pub async fn join_connection(link_id: &str, player_id: &str) -> Result<Connection, JsValue> {
    #[cfg(test)]
    {
        if let Some(mock) = unsafe { MOCK_JOIN_CONNECTION.as_ref() } {
            return mock(link_id, player_id).await;
        }
    }
    
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
    let url = format!("{}/connections/link/{}/join", get_api_base(), link_id);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    // Fetch the request
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    // Enhanced error handling
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        
        // Try to parse error response as JSON
        let error_text = JsFuture::from(resp.text()?).await?;
        let error_str = error_text.as_string().unwrap_or_default();
        
        console_log(&format!("Error response text: {}", error_str));
        
        // Try to extract the error message from JSON
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_str) {
            if let Some(error_msg) = error_json.get("error").and_then(|v| v.as_str()) {
                return Err(JsValue::from_str(error_msg));
            }
        }
        
        // Fallback to generic error if JSON parsing fails
        return Err(JsValue::from_str(&format!(
            "API error: {} {}", status, status_text
        )));
    }
    
    // Parse the response as JSON
    let json = JsFuture::from(resp.json()?).await?;
    
    // Check if response has a connection field
    if let Ok(response_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(json.clone()) {
        if let Some(connection_obj) = response_obj.get("connection") {
            // New format with nested connection object
            let connection_str = serde_json::to_string(connection_obj)
                .map_err(|e| JsValue::from_str(&format!("Error serializing connection: {:?}", e)))?;
            
            let connection_js = js_sys::JSON::parse(&connection_str)
                .map_err(|e| JsValue::from_str(&format!("Error parsing connection JSON: {:?}", e)))?;
            
            let connection_data: Connection = serde_wasm_bindgen::from_value(connection_js)?;
            
            // Optionally log the WebSocket URL
            if let Some(ws_url) = response_obj.get("websocket_url") {
                if let Some(url_str) = ws_url.as_str() {
                    console_log(&format!("Server provided WebSocket URL: {}", url_str));
                }
            }
            
            console_log(&format!("Joined connection with ID: {}", connection_data.id));
            return Ok(connection_data);
        }
    }
    
    // Fallback to direct parsing (old format)
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
    let url = format!("{}/connections/link/{}", get_api_base(), link_id);
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

pub fn save_connection_to_local_storage(connection: &Connection, friendly_name: &str) -> Option<()> {
    let window = window()?;
    let storage = window.local_storage().ok()??;

    console::log_1(&wasm_bindgen::JsValue::from_str(
        &format!("Saving connection to localStorage - ID: {}, link_id: {}", 
            connection.id, connection.link_id)
    ));
    
    // Create a structure to save
    #[derive(Serialize, Deserialize)]
    struct SavedConnection {
        id: String,
        link_id: String,
        friendly_name: String,
        created_at: i64,
        expires_at: i64, // Add expires_at field
    }
    
    let saved = SavedConnection {
        id: connection.id.clone(),
        link_id: connection.link_id.clone(),
        friendly_name: friendly_name.to_string(),
        created_at: connection.created_at,
        expires_at: connection.expires_at, // Save the expiration time
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
    let url = format!("{}/players/{}/notifications", get_api_base(), player_id);
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
    let url = format!("{}/players/{}/notifications/ack", get_api_base(), player_id);
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
    let url = format!("{}/connections/{}/messages", get_api_base(), connection_id);
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

