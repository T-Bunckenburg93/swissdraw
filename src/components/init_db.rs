use rusqlite::{Connection, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;



// #[component]
pub fn Init_DB() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;
    init_db(&conn).expect("Failed to initialize DB");
    Ok(conn)
}

// fn ChildComponent(cx: Scope) -> Element { let conn = use_context::<Rc<DbConnection>>(cx).expect("DB not provided");

fn get_db_path() -> std::path::PathBuf {
    let proj_dirs = ProjectDirs::from(".config", "swissdraw", "data").unwrap(); // this is the path for irl
    // let proj_dirs = ProjectDirs::from("com", "swissdraw", "swissdraw").unwrap();
    println!("Data dir: {:?}", proj_dirs.data_dir());
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).unwrap();
    data_dir.join("data.db")
}


// init the tables if they don't exist.
fn init_db(conn: &Connection) -> Result<()> {

    conn.execute(
        "CREATE TABLE IF NOT EXISTS draw (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            last_modified DateTime DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS teams (
            sd_id INTEGER,
            id INTEGER PRIMARY KEY,
            name TEXT,
            rank REAL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            sd_id INTEGER,
            id INTEGER PRIMARY KEY,
            round INTEGER,
            team_a_id TEXT NOT NULL,
            team_b_id TEXT NOT NULL,
            team_a_score INTEGER,
            team_b_score INTEGER,
            field INTEGER,
            played BOOLEAN DEFAULT FALSE,
            streamed BOOLEAN DEFAULT FALSE,
            _meta__is_current BOOLEAN DEFAULT TRUE,
            _meta__is_deleted BOOLEAN DEFAULT FALSE,
            _meta__last_modified DateTime DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    println!("DB tables created");
    Ok(())
}