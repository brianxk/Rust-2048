use yew::services::Task;
use yew::prelude::*;
use std::sync::mpsc::{self, Receiver};
use yew::services::ConsoleService;

fn process_keydown_messages(keydown_rx: Receiver<String>) {
    // Process keydown messages in a loop
    loop {
        if let Ok(key_code) = keydown_rx.recv() {
            // Process the key_code here
            ConsoleService::log(&format!("Received key code: {}", key_code));
        } else {
            // The channel is closed, break the loop
            break;
        }
    }
}

fn your_component() -> Html {
    let (keydown_tx, keydown_rx) = mpsc::channel::<String>();
    
    use_effect(move || {
        let document = gloo::utils::document();
        
        let listener = EventListener::new(&document, "keydown", move |event| {
            let key_code = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw().code();
            keydown_tx.send(key_code).unwrap();
        });

        // Spawn a background task to process keydown messages
        let task: Task = Box::new(move |_| {
            process_keydown_messages(keydown_rx);
        });

        spawn_local(task);

        || drop(listener)
    });

    html! {
        // Your component's HTML here
    }
}

