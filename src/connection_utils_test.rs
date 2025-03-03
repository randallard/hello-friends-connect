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