use notify::{
    event::{ModifyKind, RemoveKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::path::Path;
use tokio;
use tokio::sync::mpsc::{self, channel, Receiver};
// FIXME: Those are prototyped-required to fixed.

/// Listen files worker
/// This worker listens to the data directory and sends the file path to the given sender.
pub async fn file_listen_worker(path: &str, tx: mpsc::Sender<String>) {
    let path = Path::new(path);

    let (mut watcher, mut rx) = async_watcher().unwrap();
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

    loop {
        let res = rx.recv().await;
        match res {
            Some(event) => match event {
                Ok(event) => match event.kind {
                    /*
                    EventKind::Create(kind) => {
                        if kind == CreateKind::File {
                            let path = event.paths.last().unwrap().to_str();
                            // parse path and extract "file" from "/./data/file.json"
                            let path = parse_path(path.unwrap());
                            println!("file created: {:?}", path);

                        }
                    }*/
                    EventKind::Modify(kind) => match kind {
                            ModifyKind::Data(_) | ModifyKind::Any /* Windows OS somehow return this as any.  */=> {
                                let path = event.paths.last().unwrap();
                                // parse path and extract "file" from "/./data/file.json"
                                let path = parse_path(path);
                                println!("file modified: {:?}", path);
                                tx.send(path).await.unwrap();
                            }
                            _ => {}
                        },
                    EventKind::Remove(kind) => {
                        // some ide's using interesting mechanism to remove files so it wouldn't be detected or so.
                        if kind == RemoveKind::File {
                            let path = event.paths.last().unwrap();
                            // parse path and extract "file" from "/./data/file.json"
                            let path = parse_path(path);
                            println!("file removed: {:?}", path);
                            tx.send(path).await.unwrap();
                        }
                    }

                    _ => {}
                },
                Err(e) => {
                    println!("watch error: {:?}", e);
                }
            },
            None => {
                return;
            }
        }
    }
}

fn async_watcher() -> Result<(RecommendedWatcher, Receiver<Result<Event>>)> {
    let (tx, rx) = channel(16);

    let watcher = RecommendedWatcher::new(
        move |res| {
            event_fn(res, tx.clone()); // FIXME: Is this valid?
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

fn event_fn(res: Result<Event>, tx: mpsc::Sender<Result<Event>>) {
    tx.blocking_send(res).unwrap();
}

fn parse_path(path: &Path) -> String {
    /*
    path.split("/")
        .last()
        .unwrap()
        .split(".json")
        .next()
        .unwrap()
        .to_string()
     */
    path.file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .split(".json")
        .next()
        .unwrap()
        .to_string()
}
