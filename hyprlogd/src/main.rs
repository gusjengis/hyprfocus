mod log_writer;
mod shutdown;
mod socket;

use std::{env, time::Duration};

use hyprland::event_listener::{AsyncEventListener, WindowEventData};
use log_writer::{LogMsg, log_error, run_log_writer};
use mosaic_model::log::Log;
use shutdown::{try_spawn_logind_shutdown_watcher, wait_for_shutdown_signal};
use socket::start_socket_listener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    let mut settings = Settings::new();

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("snitch") => settings.snitch = true,
        _ => {}
    };

    // setup mpsc channel for sending messages to the log writer
    let (sender_handle, receiver_handle) = mpsc::channel::<LogMsg>(1024);

    // start the log writer, this handles all writing to log files from one thread to avoid conflicts
    let writer_jh = tokio::spawn(run_log_writer(receiver_handle, settings));

    // log boot
    let _ = sender_handle
        .send(LogMsg::Line {
            ts: chrono::Utc::now().timestamp_millis(),
            class: "SYSTEM".into(),
            title: "boot".into(),
        })
        .await;

    // listen for focus events from hyprland, the core of this program's utility
    {
        let sender_handle_static: &'static mpsc::Sender<LogMsg> =
            Box::leak(Box::new(sender_handle.clone()));

        tokio::spawn(async move {
            loop {
                let mut event_listener = AsyncEventListener::new();

                event_listener.add_active_window_changed_handler(
                    hyprland::async_closure! { move |window_data: Option<WindowEventData>| {
                        if let Some(ref data) = window_data {
                            let class = data.class.clone();
                            let title = data.title.clone();

                            let _ = sender_handle_static.try_send(LogMsg::Line {
                                ts: chrono::Utc::now().timestamp_millis(),
                                class,
                                title,
                            });
                        }
                    }},
                );

                if let Err(e) = event_listener.start_listener_async().await {
                    let ts = chrono::Utc::now().timestamp_millis();
                    log_error(format!("{ts}, [hypr] listener ended: {e}; retrying in 1s")); // output to file
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });
    }

    // listen for signals from the hyprlog CLI, this is used to get idle and resume signals
    {
        let sender_handle_sock = sender_handle.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = start_socket_listener(sender_handle_sock.clone()).await {
                    let ts = chrono::Utc::now().timestamp_millis();
                    log_error(format!("{ts}, [sock] listener failed: {e}; retrying in 3s",)); // output to file
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        });
    }

    // Listen to logind for shutdown signal, this gives enough time to reliably log shutdowns
    if let Some(jhandle) = try_spawn_logind_shutdown_watcher(sender_handle.clone()).await {
        jhandle.await;
    } else {
        // As a fallback for systems that don't have systemd, listen for the signals that come
        // during shutdown. This only works half the time in my testing. If someone who isn't on
        // systemd wants to handle shutdown properly for their system that would be awesome.
        wait_for_shutdown_signal().await;

        let _ = sender_handle
            .send(LogMsg::Line {
                ts: chrono::Utc::now().timestamp_millis(),
                class: String::from("SYSTEM"),
                title: String::from("shutdown"),
            })
            .await;
    }

    // wrap up
    drop(sender_handle);
    let _ = writer_jh.await;

    Ok(())
}

#[derive(Clone)]
pub struct Settings {
    pub snitch: bool,
}

impl Settings {
    fn new() -> Self {
        Self { snitch: false }
    }
}
