// Auth: fuer das Laden von Nutzerdaten, die Authentifizierung, und die Defintion der Berechtigungen

use std::convert::TryInto;
use sha2::{ Sha256, Sha512, Digest };
use rand::rngs::StdRng;
use rand::{ Rng, SeedableRng };

// Enhaelt eine Liste von Benutzern, mit nuetzlichen Funktionen
#[derive(Debug)]
pub struct UserDataCache {
    list: Vec<UserData>
}

#[derive(Debug, Clone)]
pub struct UserData {
    username: String,
    password_hash: String,
    hash_salt: String,
    permissions: Vec<UserPermission>
}

#[derive(Debug, Clone)]
pub enum UserPermission {
    Upload,
    Admin,
    Review
}

impl UserDataCache {
    pub fn new() -> Self {
        Self {
            list: vec![]
        }
    }

    pub fn add(&mut self, user: UserData) {
        self.list.push(user);
    }

    pub fn clear(&mut self) {
        self.list = vec![];
    }

    pub fn get_user<'a>(&'a self, username: &str) -> Option<&'a UserData> {
        for user in &self.list {
            if user.get_username() == username {
                return Some(user);
            }
        }

        None
    }
}

impl UserData {
    // Gibt wieder, ob das Passwort fuer den Benutzer richtig war
    pub fn authenticate(&self, password: &str) -> bool {
        let auth_attempt_hash = get_hash(&(password.to_string() + &self.hash_salt));
        
        &auth_attempt_hash == &self.password_hash
    }

    // Erstellt einen neuen Benutzer
    pub fn new(username: String, password: String, permissions: Vec<UserPermission>) -> Self {
        let hash_salt = generate_salt(&username); // Salt mit dem Benutzernamen erstellen, sodass dieser fuer jeden Benutzer anders ist
        let password_hash = get_hash(&(password.to_string() + &hash_salt));

        Self {
            username,
            password_hash,
            hash_salt,
            permissions
        }
    }

    pub fn get_username<'a>(&'a self) -> &'a str {
        &self.username
    }

    pub fn get_permissions<'a>(&'a self) -> &'a Vec<UserPermission> {
        &self.permissions
    }
}

// Erstellt einen zufaelligen Salt aus einer Eingabe
fn generate_salt(seed_input: &str) -> String {
    // Erstelle Seed als Hash der Eingabe
    let mut hasher = Sha256::new();
    hasher.update(&seed_input.as_bytes());
    let hash = hasher.finalize();
    let seed: [u8; 32] = hash.as_slice().try_into().unwrap(); // Seed ist immer 32 Bytes lang, da Sha256 verwendet wird


    // Erstelle Salt aus zufaelligen Buchstaben
    let mut rng = StdRng::from_seed(seed);
    let mut salt = String::new();
    for _ in 0..32 { // 32 Buchstaben von A bis Z
        salt.push(rng.gen_range(65..91) as u8 as char);
    }

    salt
}

fn get_hash(input: &str) -> String {
    // Erstelle Hash
    let mut hasher = Sha512::new();
    hasher.update(&input.as_bytes());
    let hash = hasher.finalize();

    // Konvertiere Hash zu der Hex-Darstellung
    let mut hex = String::new();
    for byte in hash {
        hex += &format!("{:02x}", byte);
    }

    hex
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_hash() {
        let input = "test_sha512";

        let expected = "e335ec8aa0e729469a06c50fe8f93621b544970ebdb99ab6351368f3541f63fc37ed92bb2fee40549de8ebfeb167386859391866541d9578684ec06ea7a70cea";
        let actual = super::get_hash(&input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_generate_salt() {
        let input1 = "username1";
        let input2 = "username2";

        assert!(super::generate_salt(input1) != super::generate_salt(input2));
    }

    #[test]
    fn test_authenticate() {
        use super::UserData;

        let user = UserData {
            username: String::from("username"),
            password_hash: String::from("8e3955db72f80547d9f27ebaa278be711dc13993e4cf516b8a0709a772b2490c3d083f7099ee2b64966f1597fccc94cdf21cb45102ad45210e832f95ab22ceab"),
            hash_salt: String::from("TOIEIDPGRCIOEHNCPDHOUEGRYPTDUTDA"),
            permissions: vec![]
        };

        assert!(user.authenticate("password"));
    }

    #[test]
    fn test_user_data_new() {
        use super::UserData;
        
        let user = UserData::new("username".to_string(), "password".to_string(), vec![]);

        assert!(user.authenticate("password"));
    }
}