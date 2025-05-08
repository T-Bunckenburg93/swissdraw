use rusqlite::{Connection, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

#[component]
fn init_db() -> Result<Connection> {
    let db_path = get_db_path();
    Connection::open(db_path)
}

// fn ChildComponent(cx: Scope) -> Element { let conn = use_context::<Rc<DbConnection>>(cx).expect("DB not provided");

fn get_db_path() -> std::path::PathBuf {
    let proj_dirs = ProjectDirs::from(".config", "swissdraw", "data").unwrap();
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).unwrap();
    data_dir.join("data.db")
}
