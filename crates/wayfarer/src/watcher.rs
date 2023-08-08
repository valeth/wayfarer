#![cfg(feature = "watch")]

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{self, spawn};
use std::time::Duration;

use notify::event::{AccessKind, AccessMode};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, Watcher};


pub struct FileWatcher {
    exit_signal: mpsc::SyncSender<()>,
}

impl FileWatcher {
    pub fn new<P, F>(path: P, callback: F) -> Self
    where
        P: Into<PathBuf>,
        F: Fn() -> () + Send + 'static,
    {
        let (exit_signal, exit) = mpsc::sync_channel(0);
        let (ev_tx, ev_rx) = mpsc::channel();

        let path = path.into();

        spawn(move || {
            let mut watcher = RecommendedWatcher::new(ev_tx, NotifyConfig::default()).unwrap();
            watcher
                .watch(path.as_ref(), notify::RecursiveMode::NonRecursive)
                .unwrap();

            let written_and_closed = EventKind::Access(AccessKind::Close(AccessMode::Write));

            loop {
                match ev_rx.try_recv() {
                    Ok(Ok(event)) => {
                        if event.kind == written_and_closed {
                            callback();
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => (),
                    Err(_) | Ok(Err(_)) => break,
                }

                if exit.try_recv().is_ok() {
                    break;
                }

                thread::sleep(Duration::from_millis(500));
            }
        });

        Self { exit_signal }
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        let _ = self.exit_signal.send(());
    }
}
