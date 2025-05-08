use rusqlite::{Connection, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;



// #[component]
pub fn Init_DB() -> Result<Connection> {
    let db_path = get_db_path();
    Connection::open(db_path)
}

// fn ChildComponent(cx: Scope) -> Element { let conn = use_context::<Rc<DbConnection>>(cx).expect("DB not provided");

fn get_db_path() -> std::path::PathBuf {
    let proj_dirs = ProjectDirs::from(".config", "swissdraw", "data").unwrap();
    println!("Data dir: {:?}", proj_dirs.data_dir());
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).unwrap();
    data_dir.join("data.db")
}
