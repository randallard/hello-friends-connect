use hello_leptos::App;
use leptos::*;
use leptos::prelude::*;

fn main() {
    // Add CSS to document head for default dark mode
    let document = web_sys::window()
        .expect("no global window")
        .document()
        .expect("no document on window");
    
    let head = document.head().expect("no head on document");
    let style = document
        .create_element("style")
        .expect("couldn't create style element");
    
    style.set_text_content(Some("
        body { 
            background-color: #111827; 
            color: #f3f4f6;
            margin: 0;
            font-family: system-ui, -apple-system, sans-serif;
        }
    "));
    
    head.append_child(&style).expect("couldn't append style");

    mount_to_body(|| view! { <App/> });
}