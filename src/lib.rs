use leptos::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[component]
pub fn App() -> impl IntoView {
    view! { 
        <div>"Hello"</div>
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