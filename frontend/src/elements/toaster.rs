#![allow(non_snake_case)]

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;

pub fn use_toaster() -> Signal<Toaster> {
    *crate::app::TOASTER
}

pub enum ToastKind {
    Error,
    Info,
    Success,
}

pub struct Toast {
    pub message: String,
    pub expires: DateTime<Utc>,
    pub kind: ToastKind,
}

#[derive(Default, Clone)]
pub struct Toaster {
    toasts: HashMap<usize, Toast>,
    next_id: usize,
}

impl Toaster {
    fn increment_id(&mut self) {
        self.next_id += 1;
    }

    pub fn push(&mut self, toast: Toast) {
        self.toasts.insert(self.next_id, toast);
        self.increment_id();
    }

    pub fn remove(&mut self, id: usize) {
        self.toasts.remove(&id);
    }

    pub fn success<T: Into<String>>(&mut self, message: T, duration: Duration) {
        let toast = Toast {
            message: message.into(),
            expires: Utc::now() + duration,
            kind: ToastKind::Success,
        };
        self.push(toast);
    }

    pub fn info<T: Into<String>>(&mut self, message: T, duration: Duration) {
        let toast = Toast {
            message: message.into(),
            expires: Utc::now() + duration,
            kind: ToastKind::Info,
        };
        self.push(toast);
    }

    pub fn error<T: Into<String>>(&mut self, message: T, duration: Duration) {
        let toast = Toast {
            message: message.into(),
            expires: Utc::now() + duration,
            kind: ToastKind::Error,
        };
        self.push(toast);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, usize, Toast> {
        self.toasts.iter()
    }
}

// Need to manually implement Clone for Toast since ToastKind doesn't derive it
impl Clone for Toast {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            expires: self.expires,
            kind: match self.kind {
                ToastKind::Error => ToastKind::Error,
                ToastKind::Info => ToastKind::Info,
                ToastKind::Success => ToastKind::Success,
            },
        }
    }
}

#[component]
pub fn ToastRoot() -> Element {
    let toaster = use_toaster();

    let toasts = toaster.read();
    let total_toasts = toasts.toasts.len();

    // Collect toast data for rendering
    let toast_data: Vec<_> = toasts
        .iter()
        .map(|(&id, toast)| (id, toast.clone()))
        .collect();

    drop(toasts); // Release the read lock

    use_future(move || {
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(200_u32).await;

                let expired_ids: Vec<usize> = toaster
                    .read()
                    .iter()
                    .filter_map(|(&id, toast)| {
                        if Utc::now() > toast.expires {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .collect();

                if !expired_ids.is_empty() {
                    let mut toaster_write = toaster.write();
                    for id in expired_ids {
                        toaster_write.remove(id);
                    }
                }

                if toaster.read().toasts.is_empty() {
                    break;
                }
            }
        }
    });

    rsx! {
        div {
            class: "fixed bottom-[var(--navbar-height)]
                    w-screen
                    max-w-[var(--content-max-width)]",
            div {
                class: "flex flex-col gap-5 px-5 mb-5",
                for (id, toast) in toast_data {
                    {
                        let toast_style = match toast.kind {
                            ToastKind::Info => "bg-slate-200 border-slate-300",
                            ToastKind::Error => "bg-rose-300 border-rose-400",
                            ToastKind::Success => "bg-emerald-200 border-emerald-300",
                        };
                        
                        rsx! {
                            div {
                                key: "{id}",
                                class: "{toast_style} p-3 cursor-pointer border-solid border rounded",
                                onclick: move |_| {
                                    toaster.write().remove(id);
                                },
                                "{toast.message}"
                            }
                        }
                    }
                }
            }
        }
    }
}
