use leptos::*;
use wasm_bindgen_test::*;

pub mod connect_component;
use connect_component::FriendsConnect;

wasm_bindgen_test_configure!(run_in_browser);

#[component]
pub fn App() -> impl IntoView {
    view! { 
        <div id="app-greeting">"Hello"</div>
        <FriendsConnect />
    }
}

#[wasm_bindgen_test]
fn test_app_says_hello() {
    mount_to_body(|| view! { <App/> });
    
    // Debug: Log the entire document body content
    let body = document().body().unwrap();
    web_sys::console::log_1(&format!("Body content: {}", body.inner_html()).into());
    
    // Try getting all divs first
    let divs = document().query_selector_all("div").unwrap();
    web_sys::console::log_1(&format!("Number of divs: {}", divs.length()).into());
    
    // Now try to find our specific div
    let div = document()
        .query_selector("#app-greeting")
        .unwrap()
        .expect("Could not find any div elements");
        
    assert_eq!(div.text_content().unwrap(), "Hello");
}

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