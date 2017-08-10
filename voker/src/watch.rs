use notify;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, RawEvent};
use std::sync::mpsc::{channel, Receiver};
use std::path::{PathBuf, Path};

pub type ChangeEvent = notify::RawEvent;

pub struct FileWatcher {
    pub watcher : RecommendedWatcher,
    pub change_events: Receiver<RawEvent>,
}

pub fn watch(path:&Path) -> FileWatcher {
    let (tx, notifier_rx) = channel::<RawEvent>();
    let mut resource_file_watcher : RecommendedWatcher = Watcher::new_raw(tx).expect("a watcher");
    resource_file_watcher.watch(path, RecursiveMode::Recursive).expect("watching resources path");

    FileWatcher {
        watcher: resource_file_watcher,
        change_events: notifier_rx,
    }
}

pub fn watch_example() {
    let this_directory = PathBuf::from(".");
    let watcher = watch(this_directory.as_path());

    'fs: loop {
        match watcher.change_events.recv() {
            Ok(RawEvent { path, op:_, cookie:_ }) => {
                if let Some(p) = path {
                    use std::path;
                    let components: Vec<path::Component> = p.components().collect();
                    println!("fs event {:?} -> {:?}", p, components);

                }
            },
            Err(_) => break 'fs,
        }
    }




}