use crate::{app_config::Source, main_window_setup, wallpaper_changer::update_wallpaper};
use anyhow::{anyhow, Result};
#[cfg(target_family = "unix")]
use fs::remove_file;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, process::exit};
use tauri::{async_runtime::spawn, AppHandle, Manager};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
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
    UpdateFromSource(Source),
    Show,
    FetchCache,
    Quit,
}

pub async fn connect(args: &Args, mut writer: impl AsyncWrite + Unpin) -> Result<()> {
    writer
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
    Ok(())
}

pub async fn handle(app: AppHandle, message: Message) -> Result<()> {
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

static SOCKET_ID: &str = include_str!("../../automation_socket.txt");

pub async fn participate(args: &Args, app: AppHandle) -> Result<()> {
    {
        #[cfg(target_family = "unix")]
        {
            let socket_path = format!(
                "/tmp/reddw-{SOCKET_ID}-{}.sock",
                hex::encode(whoami::username())
            );
            let mut listener = tokio::net::UnixListener::bind(socket_path.clone());
            if let Err(e) = &listener && e.kind() == ErrorKind::AddrInUse {
                let stream_result = tokio::net::UnixStream::connect(socket_path.clone()).await;
                if let Ok(mut stream) = stream_result
                {
                    connect(args, &mut stream).await?;
                    exit(0);
                }
                else if let Err(e) = stream_result && e.kind() == ErrorKind::ConnectionRefused {
                    remove_file(socket_path.clone())?;
                    listener = tokio::net::UnixListener::bind(socket_path.clone());
                }
            }
            let listener = listener?;
            spawn(async move {
                loop {
                    let (mut stream, _) = listener.accept().await.unwrap();
                    let mut buf = vec![];
                    stream.read_buf(&mut buf).await.unwrap();

                    let app = app.app_handle();
                    tokio::spawn(async move { handle(app, from_slice(&buf).unwrap()).await });
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
                    connect(args, &mut client).await?;
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
                                tokio::spawn(async move { handle(app, from_slice(&buf)?).await });
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
