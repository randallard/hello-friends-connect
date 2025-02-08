use leptos::*;
use leptos::prelude::*;
use wasm_bindgen_test::*;

pub mod connect_component;
use connect_component::FriendsConnect;

#[component]
pub fn App() -> impl IntoView {
    view! { 
        <FriendsConnect />
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_app_contains_friends_connect() {
        mount_to_body(|| view! { <App/> });
        
        // Look for the container by ID
        let friends_connect = document()
            .query_selector("#friends-connect-container")
            .unwrap()
            .expect("Should find FriendsConnect container");
            
        let heading = friends_connect
            .query_selector("h2")
            .unwrap()
            .expect("Should find heading inside FriendsConnect");
            
        assert_eq!(
            heading.text_content().unwrap(),
            "Connect with Friends"
        );
    }
}