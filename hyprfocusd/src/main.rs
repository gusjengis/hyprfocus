mod log_writer;
mod socket;

use std::time::Duration;

use hyprland::event_listener::{AsyncEventListener, WindowEventData};
use log_writer::{LogMsg, log_error, run_log_writer};
use socket::start_socket_listener;
use tokio::{signal, sync::mpsc};

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    let (sender_handle, receiver_handle) = mpsc::channel::<LogMsg>(1024);
    tokio::spawn(run_log_writer(receiver_handle));

    // boot marker
    let _ = sender_handle
        .send(LogMsg::Line {
            ts: chrono::Local::now().timestamp_millis(),
            class: "SYSTEM".into(),
            title: "boot".into(),
        })
        .await;

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
                                ts: chrono::Local::now().timestamp_millis(),
                                class,
                                title,
                            });
                        }
                    }},
                );

                if let Err(e) = event_listener.start_listener_async().await {
                    log_error(format!("[hypr] listener ended: {e}; retrying in 1s")); // output to file
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });
    }

    {
        let sender_handle_sock = sender_handle.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = start_socket_listener(sender_handle_sock.clone()).await {
                    log_error(format!("[sock] listener failed: {e}; retrying in 3s",)); // output to file
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        });
    }

    wait_for_shutdown_signal().await;

    let _ = sender_handle
        .send(LogMsg::Line {
            ts: chrono::Local::now().timestamp_millis(),
            class: "SYSTEM".into(),
            title: "shutdown".into(),
        })
        .await;
    let _ = sender_handle.send(LogMsg::Shutdown).await;

    Ok(())
}

async fn wait_for_shutdown_signal() {
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = sigterm.recv() => {},
    }
}
