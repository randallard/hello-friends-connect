use leptos::*;
use wasm_bindgen_test::*;

pub mod connect_component;
use connect_component::FriendsConnect;

wasm_bindgen_test_configure!(run_in_browser);

#[component]
pub fn App() -> impl IntoView {
    view! { 
        <div>"Hello"</div>
        <FriendsConnect />
    }
}

#[wasm_bindgen_test]
fn test_app_says_hello() {
    mount_to_body(|| view! { <App/> });
    
    // Get the div element
    let div = document()
        .query_selector("div")
        .unwrap()
        .unwrap();
        
    assert_eq!(div.text_content().unwrap(), "Hello");
}

#[wasm_bindgen_test]
fn test_app_contains_friends_connect() {
    mount_to_body(|| view! { <App/> });
    
    // Look for the heading that should be inside FriendsConnect
    let friends_connect_heading = document()
        .query_selector("h2")
        .unwrap()
        .expect("Should find FriendsConnect heading");
        
    assert_eq!(
        friends_connect_heading.text_content().unwrap(),
        "Connect with Friends"
    );
}