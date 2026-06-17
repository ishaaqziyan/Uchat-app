#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use std::collections::{hash_map::Values, HashMap};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct KeyedNotifications {
    pub inner: HashMap<String, String>,
}

impl KeyedNotifications {
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.insert(key.into(), value.into());
    }

    pub fn remove<K: AsRef<str>>(&mut self, key: K) {
        self.inner.remove(key.as_ref());
    }

    pub fn messages(&self) -> Values<'_, String, String> {
        self.inner.values()
    }

    pub fn has_messages(&self) -> bool {
        !self.inner.is_empty()
    }
}

#[component]
pub fn KeyedNotificationBox(legend: Option<String>, notifications: KeyedNotifications) -> Element {
    let notifications_elements = notifications.messages().map(|msg| {
        rsx! { li { "{msg}" } }
    });

    let legend_str = legend.unwrap_or_else(|| "Errors".to_string());

    if notifications.has_messages() {
        rsx! {
            fieldset {
                class: "fieldset border-red-300 dark:border-red-800 rounded transition-colors duration-300",
                legend {
                    class: "bg-red-300 dark:bg-red-800 dark:text-white px-4 transition-colors duration-300",
                    "{legend_str}"
                }
                ul {
                    class: "list-disc ml-4 dark:text-red-200",
                    {notifications_elements}
                }
            }
        }
    } else {
        rsx! {}
    }
}
