use crate::{app_handle_ext::AppHandleExt, main_window_setup, wallpaper_changer::update_wallpaper};
use anyhow::{anyhow, Result};
use reddw_ipc::{IPCData, IPCMessage, SOCKET_PATH};
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};
#[cfg(target_family = "unix")]
use std::fs::remove_file;
use std::{io::ErrorKind, process::exit};
use tauri::{async_runtime::spawn, AppHandle, Manager};
use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::watch::{Receiver, Sender},
};
#[cfg(target_family = "windows")]
use {std::io, tokio::net::windows::named_pipe};

#[derive(clap::Parser, Debug, Clone)]
#[clap(version = "1.0", author)]
pub struct Args {
    #[arg(short, long)]
    pub background: bool,
    #[arg(short, long)]
    pub quit: bool,
    #[arg(short, long)]
    pub update: bool,
    #[arg(short, long)]
    pub fetch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    UpdateWallpaper,
    UpdateFromSource(String),
    Show,
    FetchCache,
    Quit,
}

pub async fn handle_automation(app: AppHandle, message: Message) -> Result<()> {
    match message {
        Message::Show => {
            if app.get_window("main").is_none() {
                main_window_setup(app.app_handle())?.show()?;
            };
            Ok(())
        }
        Message::UpdateWallpaper => update_wallpaper(app.app_handle()).await,
        // Message::UpdateFromSource(source) => {
        //     update_wallpaper(app.handle()).await?;
        //     set_config(app.handle(), source).await?;
        // },
        Message::Quit => Ok(app.exit(0)),
        _ => Ok(()),
    }
    .map_err(|e| anyhow!(e))?;

    #[cfg(debug_assertions)]
    if let Some(main_window) = app.get_window("main") {
        main_window.emit("print", format!("{message:#?}"))?;
    }

    Ok(())
}

pub async fn initiate_ipc(args: &Args, app: AppHandle) -> Result<()> {
    {
        let (broadcaster, receiver) =
            tokio::sync::watch::channel::<IPCData<Vec<u8>>>((IPCMessage::Init, Vec::new()));
        app.manage(broadcaster);
        app.manage(receiver);
        #[cfg(target_family = "unix")]
        {
            let mut listener = tokio::net::UnixListener::bind(SOCKET_PATH.as_path());
            if let Err(e) = &listener
                && e.kind() == ErrorKind::AddrInUse
            {
                let stream_result = tokio::net::UnixStream::connect(SOCKET_PATH.as_path()).await;
                if let Ok(mut stream) = stream_result {
                    let writer = &mut stream;
                    writer
                        .write_all(&to_vec(&(
                            IPCMessage::AutomationSocket,
                            to_vec(&if args.quit {
                                Message::Quit
                            } else if args.fetch {
                                Message::FetchCache
                            } else if args.update {
                                Message::UpdateWallpaper
                            } else {
                                Message::Show
                            })?,
                        ))?)
                        .await?;
                    exit(0);
                } else if let Err(e) = stream_result
                    && e.kind() == ErrorKind::ConnectionRefused
                {
                    remove_file(SOCKET_PATH.as_path())?;
                    listener = tokio::net::UnixListener::bind(SOCKET_PATH.as_path());
                }
            }
            let listener = listener?;
            let app_clone = app.app_handle();
            spawn(async move {
                loop {
                    let (mut stream, _) = listener.accept().await.unwrap();
                    let mut buf = vec![];
                    stream.read_buf(&mut buf).await.unwrap();

                    let app = app.app_handle();
                    tokio::spawn(async move {
                        let message = from_slice::<IPCData<Vec<u8>>>(&buf).unwrap();
                        let _ = app.state::<Sender<IPCData<Vec<u8>>>>().send(message);
                    });
                }
            });
            spawn(async move {
                loop {
                    let message = app_clone
                        .listen_ipc::<Message>(|t: &IPCMessage| match t {
                            IPCMessage::AutomationSocket => true,
                            _ => false,
                        })
                        .await
                        .map_err(|e| eprintln!("{:#?}", e))
                        .unwrap();
                    eprintln!("Recieved {:#?}", message);
                    let _ = handle_automation(app_clone.app_handle(), message).await;
                }
            });
        }
        #[cfg(target_family = "windows")]
        {
            let socket_id = format!("\\\\.\\pipe\\{SOCKET_ID}");
            match named_pipe::ServerOptions::new()
                .first_pipe_instance(true)
                .create(&socket_id)
            {
                Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                    let mut client = named_pipe::ClientOptions::new().open(&socket_id)?;
                    client
                        .write_all(&to_vec(&{
                            if args.quit {
                                Message::Quit
                            } else if args.fetch {
                                Message::FetchCache
                            } else if args.update {
                                Message::UpdateWallpaper
                            } else {
                                Message::Show
                            }
                        })?)
                        .await?;
                    #[allow(unreachable_code)]
                    return Ok(exit(0));
                }
                Ok(mut server) => {
                    spawn(async move {
                        loop {
                            async {
                                // Wait for a client to connect.
                                server.connect().await?;
                                let mut buf = vec![];
                                server.read_buf(&mut buf).await?;
                                server = named_pipe::ServerOptions::new().create(&socket_id)?;
                                let app = app.app_handle();
                                tokio::spawn(async move {
                                    let message = from_slice::<IPCData<Vec<u8>>>(&buf).unwrap();
                                    let _ = app.state::<Sender<IPCData<Vec<u8>>>>().send(message);
                                });
                                Ok::<(), io::Error>(())
                            }
                            .await
                            .unwrap_or_else(|e| eprintln!("Error: {}", e));
                        }
                    });
                    Ok(())
                }
                Err(e) => Err(anyhow!(e)),
            }?;
        }
    }
    Ok(())
}
