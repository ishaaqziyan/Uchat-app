#![allow(non_snake_case)]


use dioxus::prelude::*;

pub fn NewChat(cx: Scope) -> Element {
    cx.render(rsx! {
        form {
        class: "flex flex-col gap-4",
        onsubmit: move |_| (),
        prevent_default: "onsubmit",
        // Message Input
        // Headline Input
        button{
            class: "btn",
            r#type: "submit",
            disabled: true,
            "Post"
        }
    }
    
    })

}