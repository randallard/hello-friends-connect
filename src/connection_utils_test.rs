#[cfg(test)]
mod connection_utils_tests {
    use leptos::*;
    use leptos::prelude::*;    
    use std::{pin, sync::Once};
    use std::future::Future;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;
    use web_sys::{window, UrlSearchParams};
    
    use crate::{
        connect_component::{Connection, ConnectionStatus}, 
        connection_modal::ConnectionModal, 
        connection_utils::{
            extract_link_id_from_search, 
            get_link_id_from_url,
            MOCK_CREATE_CONNECTION
        }
    };

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_create_connection_returns_link_id() {
        unsafe {
            // Create a mock that returns a successful connection
            MOCK_CREATE_CONNECTION = Some(Box::new(|player_id| {
                let pid_owned = player_id.to_string();
                Box::pin(async move {
                    // Create a mock connection that matches your Connection struct
                    let connection = Connection {
                        id: "mock-id".to_string(),
                        link_id: "mock-link-123".to_string(),
                        status: ConnectionStatus::Pending,
                        created_at: 0,
                        expires_at: 0, 
                        players: vec![pid_owned],
                    };
                    Ok(connection)
                })
            }));
        }
    
        // Get or create a player ID
        let player_id = crate::connect_component::get_stored_player_id()
            .unwrap_or_else(|| {
                let new_id = uuid::Uuid::new_v4().to_string();
                let window = web_sys::window().unwrap();
                let storage = window.local_storage().unwrap().unwrap();
                storage.set_item("player-id", &new_id).unwrap();
                new_id
            });
        
        // Call the API to create a connection, which will use our mock
        match crate::connection_utils::create_connection(&player_id).await {
            Ok(connection) => {
                // Verify we got a link_id back
                assert!(!connection.link_id.is_empty(), "Connection should have a link_id");
                assert!(!connection.id.is_empty(), "Connection should have an id");
                
                // Log success for debugging
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                    &format!("Successfully created connection with link_id: {}", connection.link_id)
                ));
            },
            Err(err) => {
                // Convert JsValue error to string for assertion message
                let error_msg = format!("Failed to create connection: {:?}", err);
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&error_msg));
                assert!(false, "{}", error_msg);
            }
        }
        
        // Clean up the mock after the test
        unsafe {
            MOCK_CREATE_CONNECTION = None;
        }
    }
    
    #[wasm_bindgen_test]
    fn test_extract_link_id_from_search() {
        // Test with a valid search parameter
        let search = "?link=test-link-123";
        let result = extract_link_id_from_search(search);
        assert!(result.is_some(), "Link ID should be extracted from search string");
        assert_eq!(result.unwrap(), "test-link-123", "Extracted link ID should match test ID");
    }
    
    #[wasm_bindgen_test]
    async fn test_basic_localStorage_operations() {
        // A simple test to make sure we can use localStorage
        let window = window().expect("window should exist");
        let storage = window.local_storage().unwrap().unwrap();
        
        // Clear any existing test data
        storage.remove_item("test-key").unwrap();
        
        // Set and get a value
        let test_value = "test-value-123";
        storage.set_item("test-key", test_value).unwrap();
        
        let retrieved = storage.get_item("test-key").unwrap().unwrap();
        assert_eq!(retrieved, test_value, "Retrieved value should match what was stored");
        
        // Clean up
        storage.remove_item("test-key").unwrap();
    }
}