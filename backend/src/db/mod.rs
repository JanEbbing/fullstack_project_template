pub mod migrations;

use rusqlite::Connection;

pub fn init(database_url: &str) -> Result<Connection, rusqlite::Error> {
    if let Some(parent) = std::path::Path::new(database_url).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(database_url)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrations::run(&conn)?;
    Ok(conn)
}
