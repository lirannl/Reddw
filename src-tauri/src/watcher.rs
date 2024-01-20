use anyhow::Result;
use debounce::EventDebouncer;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, Stream, StreamExt,
};
use notify::{Config, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    pin::{pin, Pin},
    task::{Context, Poll},
    time::Duration,
};
use tauri::{async_runtime::spawn, AppHandle, Manager};
use tokio::sync::Mutex;

use crate::{app_handle_ext::AppHandleExt, log::LogLevel};

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

pub struct FileWatch {
    app: AppHandle,
    receiver: Receiver<Result<Event, notify::Error>>,
}
impl Stream for FileWatch {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let event = self.receiver.poll_next_unpin(cx);
        match event {
            Poll::Ready(Some(Ok(event))) => Poll::Ready(Some(event)),
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                self.app.log(&err, LogLevel::Error);
                Poll::Pending
            }
        }
    }
}
impl FileWatch {
    fn new(
        app: AppHandle,
        receiver: Receiver<Result<Event, notify::Error>>,
        debounce: Duration,
    ) -> Self {
        let receiver = if debounce.is_zero() {
            receiver
        } else {
            debounce_stream(receiver, debounce)
        };
        FileWatch { app, receiver }
    }
}
fn debounce_stream<
    Item: PartialEq + Send + 'static,
    E: Error + Send + 'static,
    S: Stream<Item = Result<Item, E>> + Send + 'static,
>(
    receiver: S,
    duration: Duration,
) -> Receiver<Result<Item, E>> {
    let (mut tx, rx) = channel(1);
    let debouncer = EventDebouncer::new(duration, move |i| {
        let _ = tx.try_send(Ok(i));
    });
    spawn(async move {
        let mut receiver = pin!(receiver);
        while let Some(val) = receiver.next().await {
            if let Ok(val) = val {
                debouncer.put(val);
            }
        }
        debouncer.stop().join().unwrap();
    });
    rx
}

pub type FileWatches = Mutex<HashMap<PathBuf, INotifyWatcher>>;

pub fn setup_file_watches(app: AppHandle) -> bool {
    app.manage(FileWatches::new(HashMap::new()))
}

pub fn watch_path_sync(
    app: AppHandle,
    target: &Path,
    recursiveness: RecursiveMode,
    debounce: Duration,
) -> Result<FileWatch> {
    let (mut watcher, receiver) = async_watcher()?;
    watcher.watch(target, recursiveness)?;
    app.state::<FileWatches>()
        .blocking_lock()
        .insert(target.to_owned(), watcher);
    Ok(FileWatch::new(app, receiver, debounce))
}

pub async fn watch_path(
    app: AppHandle,
    target: &Path,
    recursiveness: RecursiveMode,
    debounce: Duration,
) -> Result<FileWatch> {
    let (mut watcher, receiver) = async_watcher()?;
    watcher.watch(target, recursiveness)?;
    app.state::<FileWatches>()
        .lock()
        .await
        .insert(target.to_owned(), watcher);
    Ok(FileWatch::new(app, receiver, debounce))
}
