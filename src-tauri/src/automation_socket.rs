use std::{io::ErrorKind, process::exit};

use crate::{app_config::Source, main_window_setup, wallpaper_changer::update_wallpaper};
use anyhow::{anyhow, Result};
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{api::cli, async_runtime::spawn, AppHandle, Manager};
use tokio::io::{self, AsyncReadExt, AsyncWrite, AsyncWriteExt};
#[cfg(target_family = "windows")]
use tokio::net::windows::named_pipe;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    UpdateWallpaper,
    UpdateFromSource(Source),
    Show,
    FetchCache,
}

pub async fn connect(app: AppHandle, mut writer: impl AsyncWrite + Unpin) -> Result<()> {
    writer
        .write_all(&to_vec(&match &app.state::<cli::Matches>().args {
            args if args["update"].occurrences > 0 => match &args["update"].value {
                Value::Array(a) if let [Value::String(src), Value::String(subreddit)] = &a[..] && src == "subreddit" => {
                    Message::UpdateFromSource(Source::Subreddit(subreddit.clone()))
                },
                _ => Message::UpdateWallpaper,
            },
            args if args["fetch-cache"].occurrences > 0 => Message::FetchCache,
            _ => Message::Show,
        })?)
        .await?;
    Ok(())
}

pub async fn handle(app: AppHandle, message: Message) -> Result<()> {
    match message {
        Message::Show => {
            if app.get_window("main").is_none() {
                main_window_setup(app.app_handle())?;
            };
            Ok(())
        }
        Message::UpdateWallpaper => update_wallpaper(app.app_handle()).await,
        // Message::UpdateFromSource(source) => {
        //     update_wallpaper(app.handle()).await?;
        //     set_config(app.handle(), source).await?;
        // },
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

pub async fn participate(app: AppHandle) -> Result<()> {
    {
        #[cfg(target_family = "unix")]
        {
            match tokio::net::UnixListener::bind(SOCKET_ID) {
                Err(e) if e.kind() == AddrInUse => {
                    let stream = tokio::net::UnixStream::connect(SOCKET_ID).await?;
                    connect(app.app_handle(), stream).await?;
                    exit(0);
                }
                Ok(listener) => {
                    spawn(async move {
                        #[allow(unreachable_code)]
                        Result::<(), Error>::Ok({
                            loop {
                                let (mut socket, _) = listener.accept().await?;
                                let mut buf: Vec<u8> = vec![];
                                socket.read_to_end(&mut buf).await?;
                                handle(app.app_handle(), from_slice(&buf)?).await?;
                            }
                        })
                    });
                    Ok::<_, Error>(())
                }
                Err(e) => Err(anyhow!(e)),
            }?;
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
                    connect(app.app_handle(), &mut client).await?;
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
