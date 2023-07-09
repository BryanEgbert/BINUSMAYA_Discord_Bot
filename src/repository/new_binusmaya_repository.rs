use chrono::{NaiveDateTime, Local, TimeZone};
use rusqlite;


use crate::models;

#[derive(Debug)]
pub struct NewBinusmayaRepository {
    connection: rusqlite::Connection
}

unsafe impl Send for NewBinusmayaRepository {}
unsafe impl Sync for NewBinusmayaRepository {}

impl NewBinusmayaRepository {
    pub fn new(conn: rusqlite::Connection) -> Self {
        conn.execute("CREATE TABLE IF NOT EXISTS new_binusmaya_user(
            member_id INTEGER PRIMARY KEY NOT NULL,
            auth TEXT NOT NULL,
            last_registered INTEGER NOT NULL
        );", []).unwrap();

        NewBinusmayaRepository { connection: conn }
    }

    pub fn insert(&self, user: &models::user::NewBinusmayaUser) -> Result<usize, rusqlite::Error> {
        self.connection.execute(
            "INSERT INTO new_binusmaya_user (member_id, auth, last_registered) VALUES (?1, ?2, ?3);", 
            rusqlite::params![user.member_id, user.auth, user.last_registered.timestamp()],
        )
    }

    pub fn get_by_id(&self, member_id: &u64) -> Option<Result<models::user::NewBinusmayaUser, rusqlite::Error>> {
        let mut stmt = self.connection.prepare(
            "SELECT member_id, auth, last_registered FROM new_binusmaya_user WHERE member_id = :member_id;"
        ).unwrap();

        let data = stmt.query_map(&[(":member_id", &member_id)], |row| {
            Ok(models::user::NewBinusmayaUser {
                member_id: row.get(0)?,
                auth: row.get(1)?,
                last_registered: Local.from_local_datetime(&NaiveDateTime::from_timestamp_opt(row.get(2)?, 0).unwrap()).unwrap(),
            })
        });

        match data {
            Ok(d) => d.last(),
            Err(e) => Some(Err(e))
        }
    }

    pub fn get_all(&self) -> Result<Vec<models::user::NewBinusmayaUser>, rusqlite::Error> {
        let mut users: Vec<models::user::NewBinusmayaUser> = std::vec![];
        let mut stmt = self.connection.prepare(
            "SELECT member_id, auth, last_registered FROM new_binusmaya_user;"
        ).unwrap();

        let data = stmt.query_map([], |row| {
            Ok(models::user::NewBinusmayaUser {
                member_id: row.get(0)?,
                auth: row.get(1)?,
                last_registered: Local.from_local_datetime(&NaiveDateTime::from_timestamp_opt(row.get(2)?, 0).unwrap()).unwrap(),
            })
        });

        match data {
            Ok(d) => {
                for user in d {
                    users.push(user.expect("something's wrong in getting user data"))
                }

                Ok(users)
            },
            Err(e) => Err(e),            
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_by_id() {
        let conn = rusqlite::Connection::open("test.db").unwrap();
        let sut = NewBinusmayaRepository::new(conn);

        println!("{:?}", sut.get_by_id(&1));
    }
}