mod rest;
mod utils;

use crate::rest::ui::{handle_ui_files, index};
use crate::rest::yubikey::submit_key;
use crate::utils::image::load_icon;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use log::info;
use nix::unistd::Uid;
use std::process::{Command, Stdio};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use tokio::runtime::Runtime;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;
use wry::WebViewBuilder;

fn main() -> Result<()> {
    get_root_privs();
    println!("All good.");

    let web_server_port = 55584;

    // Create a one-shot channel for shutdown signal
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Create an event loop for Tao (webview)
    let event_loop = EventLoop::new();
    let event_loop_proxy = event_loop.create_proxy();
    let icon = load_icon(&*include_bytes!("../icon.png").to_vec())?;

    let window = WindowBuilder::new()
        .with_title("Yubi Goblin")
        .with_maximized(true)
        .with_window_icon(Some(icon))
        .build(&event_loop)?;

    // Build the WebView
    let builder = WebViewBuilder::new().with_url(&format!("http://localhost:{}/", web_server_port));
    let _webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        builder.build_gtk(vbox)?
    };

    // Register Ctrl+C handler to trigger a user event, which in turn will trigger shutdown
    let event_loop_proxy_ctrlc = event_loop_proxy.clone();
    ctrlc::set_handler(move || {
        // Trigger a user event to notify the event loop
        event_loop_proxy_ctrlc.send_event(()).ok();
    })
        .expect("Error setting Ctrl-C handler");

    let addr = format!("0.0.0.0:{}", web_server_port);
    info!("Starting web server at '{}'", addr);

    // Spawn the web server in a separate thread
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            let server = HttpServer::new(move || {
                App::new()
                    .route("/", web::get().to(index))
                    .route("/api/v1/yubikey", web::get().to(submit_key))
                    .route("/{path:.*}", web::get().to(handle_ui_files))
            })
                .bind(addr)
                .expect("Failed to bind address");

            let server = server.run();
            tokio::pin!(server);

            // Clone handle for stopping the server
            let server_handle = server.handle();

            // Spawn a task that listens for SIGTERM
            {
                let event_loop_proxy_unix = event_loop_proxy.clone();
                tokio::spawn(async move {
                    let mut sigterm = signal(SignalKind::terminate())
                        .expect("Failed to register SIGTERM handler");

                    // Wait for SIGTERM
                    sigterm.recv().await;
                    println!("Received SIGTERM signal");
                    // Send user event to trigger shutdown
                    event_loop_proxy_unix.send_event(()).ok();
                });
            }

            tokio::select! {
                // If the server finishes first
                res = &mut server => {
                    if let Err(e) = res {
                        eprintln!("Server error: {}", e);
                    }
                }

                // Wait for a shutdown signal from the event loop
                _ = shutdown_rx => {
                    println!("Shutdown signal received. Exiting loop...");
                    server_handle.stop(true).await;
                }
            }
        })
    });
    // Wrap shutdown_tx in an Option before entering the event loop
    let mut shutdown_tx = Some(shutdown_tx);
    
    // Run the event loop for the webview
    // When `Event::UserEvent(())` is triggered (e.g., by Ctrl+C or SIGTERM),
    // we send a shutdown signal through the one-shot channel.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(()) => {
                println!("Shutdown requested, exiting event loop");
                *control_flow = ControlFlow::Exit;

                // Send shutdown signal via the one-shot channel (ignoring if receiver is dropped)
                if let Some(tx) = shutdown_tx.take() {
                    let _ = tx.send(()); // Send the shutdown signal once
                }
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Window close requested");
                *control_flow = ControlFlow::Exit;
                // Send shutdown signal if window is closed
                if let Some(tx) = shutdown_tx.take() {
                    let _ = tx.send(()); // Send the shutdown signal once
                }
            },
            _ => (),
        }
    });
    // Ok(())
}

fn get_root_privs() {
    if Uid::current().is_root() {
        println!("We are root.");
        return;
    }
    let current_exe = std::env::current_exe().expect("Unable to get current executable path");
    let display = std::env::var("DISPLAY").unwrap_or_default();
    let xauthority = std::env::var("XAUTHORITY").unwrap_or_default();

    let args_str = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let command_line = format!(
        "DISPLAY={} XAUTHORITY={} {} {}",
        display,
        xauthority,
        current_exe.display(),
        args_str
    );

    let mut child = Command::new("pkexec")
        .arg("bash")
        .arg("-c")
        .arg(&command_line)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start pkexec.");

    let status = child.wait().expect("Failed to wait on pkexec child process");

    if status.success() {
        std::process::exit(0);
    } else {
        eprintln!(
            "Failed to acquire root privileges via pkexec. Status: {:?}",
            status
        );
        std::process::exit(1);
    }
}
