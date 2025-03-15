// Create a new file: notification_component.rs

use std::cell::RefCell;

use leptos::*;
use leptos::prelude::*;
use crate::connection_utils;
use wasm_bindgen_futures::spawn_local;

// Define a global storage for notifications
thread_local! {
    static NOTIFICATIONS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

// Function to add a new notification
pub fn add_notification(message: String) {
    NOTIFICATIONS.with(|notifications| {
        notifications.borrow_mut().push(message);
    });
    
    // Trigger re-render of any notification components
    trigger_notifications_change();
}

// Signal to track when notifications change
static NOTIFICATIONS_CHANGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn trigger_notifications_change() {
    let current = NOTIFICATIONS_CHANGED.load(std::sync::atomic::Ordering::SeqCst);
    NOTIFICATIONS_CHANGED.store(!current, std::sync::atomic::Ordering::SeqCst);
}

// Function to get all notifications
pub fn get_notifications() -> Vec<String> {
    NOTIFICATIONS.with(|notifications| {
        notifications.borrow().clone()
    })
}

// Function to clear all notifications
pub fn clear_notifications() {
    NOTIFICATIONS.with(|notifications| {
        notifications.borrow_mut().clear();
    });
    trigger_notifications_change();
}

// Component to display notifications
#[component]
pub fn NotificationList() -> impl IntoView {
    // Create a signal to track notifications
    let (notifications, set_notifications) = signal(get_notifications());
    
    // React to changes in the NOTIFICATIONS_CHANGED atomic
    create_effect(move |_| {
        // This is a bit of a hack, but it works to trigger re-renders
        let _ = NOTIFICATIONS_CHANGED.load(std::sync::atomic::Ordering::SeqCst);
        set_notifications.set(get_notifications());
    });
    
    // Function to dismiss notifications
    let dismiss_all = move |_| {
        clear_notifications();
    };
    
    view! {
        <div class="notification-container">
            {move || {
                let current_notifications = notifications.get();
                if current_notifications.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <div class="fixed bottom-4 right-4 max-w-md z-50">
                            <div class="bg-gray-800 border border-gray-700 rounded-lg shadow-lg p-4">
                                <div class="flex justify-between items-center mb-2">
                                    <h3 class="text-md font-bold text-gray-100">
                                        "Notifications"
                                    </h3>
                                    <button 
                                        class="text-gray-400 hover:text-gray-100"
                                        on:click=dismiss_all
                                    >
                                        "Dismiss All"
                                    </button>
                                </div>
                                <ul class="space-y-2">
                                    <For
                                        each=move || current_notifications.clone()
                                        key=|notification| notification.clone()
                                        let:notification
                                    >
                                        <li class="border-b border-gray-700 pb-2 last:border-0 last:pb-0">
                                            {notification}
                                        </li>
                                    </For>
                                </ul>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// Start polling for notifications
pub fn start_notification_polling(player_id: String) {
    spawn_local(async move {
        loop {
            // Poll for notifications every 5 seconds
            gloo_timers::future::TimeoutFuture::new(5000).await;
            
            match connection_utils::poll_notifications(&player_id).await {
                Ok(notifications) => {
                    if !notifications.is_empty() {
                        // Add each notification to our local store
                        for notification in notifications {
                            add_notification(notification);
                        }
                    }
                },
                Err(e) => {
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                        &format!("Error polling for notifications: {:?}", e)
                    ));
                }
            }
        }
    });
}