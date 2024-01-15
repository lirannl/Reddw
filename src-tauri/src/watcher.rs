use crate::{app_handle_ext::AppHandleExt, log::LogLevel};
use anyhow::Result;
use futures::{
    channel::mpsc::{channel, Receiver},
    Future, SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::Path, pin::Pin};
use tauri::{
    async_runtime::{spawn, JoinHandle},
    AppHandle, Manager,
};

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub fn watch_path(
    target: &Path,
    action: impl 'static
        + Send
        + Sync
        + Fn(Event, AppHandle) -> Pin<Box<dyn Send + Sync + Future<Output = Result<()>>>>,
    app: AppHandle,
    recursiveness: RecursiveMode,
) -> Result<JoinHandle<Result<()>>> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(target, recursiveness)?;

    let handle = spawn(
        (|| -> Pin<Box<dyn Send + Sync + Future<Output = Result<_>>>> {
            Box::pin(async move {
                let mut action = action;
                let action = &mut action;
                let _watcher = watcher;
                while let Some(Ok(event)) = rx.next().await {
                    action(event, app.app_handle())
                        .await
                        .unwrap_or_else(|err| app.log(&err.to_string(), LogLevel::Error));
                }
                Ok(())
            })
        })(),
    );
    Ok(handle)
}
