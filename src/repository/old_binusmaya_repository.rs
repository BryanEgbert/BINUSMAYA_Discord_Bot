use rusqlite;

use crate::models;

const OLD_BINUSMAYA_USER_TABLE_NAME: &str = "old_binusmaya_user";

pub struct OldBinusmayaRepository {
    connection: rusqlite::Connection
}

unsafe impl Send for OldBinusmayaRepository {}
unsafe impl Sync for OldBinusmayaRepository {}

impl OldBinusmayaRepository {
    pub fn new(conn: rusqlite::Connection) -> Self {
        conn.execute("CREATE TABLE IF NOT EXISTS old_binusmaya_user(
            member_id INTEGER PRIMARY KEY NOT NULL,
            email VARCHAR(255) NOT NULL,
            password VARCHAR(255) NOT NULL,
            binusian_id INTEGER NOT NULL,
            display_name VARCHAR(255) NOT NULL,
            user_id INTEGER NOT NULL,
            role_id INTEGER NOT NULL,
            specific_role_id INTEGER NOT NULL,
            cookie VARCHAR(255)
        );", []).unwrap();
        OldBinusmayaRepository { connection: conn }
    }

    pub fn insert(&self, user: &models::user::OldBinusmayaUser) -> Result<usize, rusqlite::Error> {
        self.connection.execute(
            "INSERT into old_binusmaya_user (member_id, email, password, binusian_id, display_name, user_id, role_id, specific_role_id) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
            rusqlite::params![user.member_id, user.user_credential.email, user.user_credential.password, user.binusian_data.binusian_id, user.binusian_data.display_name, user.binusian_data.user_id, user.binusian_data.role_id, user.binusian_data.specific_role_id],
        )
    }

    pub fn get_all(&self) -> Result<Vec<models::user::OldBinusmayaUser>, rusqlite::Error> {
        let mut users: Vec<models::user::OldBinusmayaUser> = std::vec![];
        let mut stmt = self.connection.prepare(
            "SELECT member_id, email, password, binusian_id, display_name, user_id, role_id, specific_role_id, cookie 
            FROM old_binusmaya_user;"
        ).unwrap();

        let data = stmt.query_map([], |row| {
            Ok(models::user::OldBinusmayaUser {
                member_id: row.get(0)?,
                user_credential: models::user::UserCredential {
                    email: row.get(1)?,
                    password: row.get(2)?,
                },
                binusian_data: models::user::UserBinusianData {
                    binusian_id: row.get(3)?,
                    display_name: row.get(4)?,
                    user_id: row.get(5)?,
                    role_id: row.get(6)?,
                    specific_role_id: row.get(7)?,
                },
                cookie: row.get(8).unwrap_or_default()
            })
        });

        match data {
            Ok(d) => {
                for user in d {
                    users.push(user.expect("something's wrong in getting users data"));
                }

                Ok(users)
            },
            Err(e) => Err(e)
        }
    }

    pub fn get_by_id(&self, member_id: &u64) -> Option<Result<models::user::OldBinusmayaUser, rusqlite::Error>> {
        let mut stmt = self.connection.prepare(
            "SELECT member_id, email, password, binusian_id, display_name, user_id, role_id, specific_role_id, cookie 
            FROM old_binusmaya_user
            WHERE member_id = :member_id;"
        ).unwrap();

        let data = stmt.query_map(&[(":member_id", &member_id)], |row| {
            Ok(models::user::OldBinusmayaUser {
                member_id: row.get(0)?,
                user_credential: models::user::UserCredential {
                    email: row.get(1)?,
                    password: row.get(2)?,
                },
                binusian_data: models::user::UserBinusianData {
                    binusian_id: row.get(3)?,
                    display_name: row.get(4)?,
                    user_id: row.get(5)?,
                    role_id: row.get(6)?,
                    specific_role_id: row.get(7)?,
                },
                cookie: row.get(8).unwrap_or_default()
            })
        });

        match data {
            Ok(d) => d.last(),
            Err(e) => Some(Err(e))
        }
    }

    pub fn update_cookie_by_id(&self, member_id: &u64, cookie: String) -> Result<usize, rusqlite::Error> {
        self.connection.execute(
            "UPDATE old_binusmaya_user SET cookie = ?1 WHERE member_id = ?2", 
            rusqlite::params![member_id, cookie])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_by_id() {
        let conn = rusqlite::Connection::open("test.db").unwrap();
        let sut = OldBinusmayaRepository::new(conn);

        println!("{:?}", sut.get_by_id(&1));
    }
}