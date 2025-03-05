#[cfg(test)]
mod connection_utils_tests {
    use leptos::*;
    use leptos::prelude::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;
    use web_sys::{window, UrlSearchParams};
    use crate::{connection_modal::ConnectionModal, connection_utils::{extract_link_id_from_search, get_link_id_from_url}};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_create_connection_returns_link_id() {
        // Get or create a player ID
        let player_id = crate::connect_component::get_stored_player_id()
            .unwrap_or_else(|| {
                let new_id = uuid::Uuid::new_v4().to_string();
                let window = web_sys::window().unwrap();
                let storage = window.local_storage().unwrap().unwrap();
                storage.set_item("player-id", &new_id).unwrap();
                new_id
            });
        
        // Call the API to create a connection
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