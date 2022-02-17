// Protected: fuer Seiten, die nur mit bestimmten Berechtigungen zugaenglich sein sollen (html und interne Ressourcen), und die dazugehoeringen Authentifikationsfunktionen

use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use std::time::SystemTime;
use rand::rngs::StdRng;
use rand::{ Rng, SeedableRng };
use rand::distributions::Alphanumeric;
use crate::auth::{ UserData, UserDataCache };

const COOKIE_LIVE_TIME: u64 = 60 * 60; // Zeit (in s), bevor ein Cookie ungueltig wird

// TODO: Cookie::new() so aendern, dass nicht zwei Cookies, die in derselben Milisekunde erstellt werden, gleich sind

// Enthaelt die Liste von aktiven Cookies, die fuer die Authentifikation verwendet werden
#[derive(Debug)]
pub struct AuthCookieMap {
    user_cache: UserDataCache,
    cookie_map: HashMap<Cookie, (AuthenticatedUser, Timestamp)>
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Cookie(String);
#[derive(Debug)]
struct Timestamp(u64); // in s (UNIX Zeit)
#[derive(Debug)]
pub struct AuthenticatedUser(UserData);

impl<'a> AuthCookieMap {
    // Cookie fuer einen Nutzer hinzufuegen
    pub fn add_new_cookie(&mut self, username: &str, password: &str) -> Option<Cookie> {
        let user = self.user_cache.get_user(username)?;
        let user = AuthenticatedUser::authenticate_new_user(user, password)?;
        
        let cookie = Cookie::new();
        self.cookie_map.insert(cookie.clone(), (user, Timestamp::now()));

        Some(cookie)
    }

    // Alle Cookies, die gespeichert sind nach gueltigkeit ueberpruefen, und, falls noetig, entfernen
    pub fn cleanup(&mut self) {
        self.cookie_map.retain(|_, (_, timestamp)| timestamp.describes_valid_cookie());
    }

    // Gibt die Nutzerdaten wieder, die mit dem gegebenen Cookie gespeichert sind
    pub fn lookup_cookie(&'a self, cookie: Cookie) -> Option<&'a AuthenticatedUser> {
        if let Some((user_data, timestamp)) = self.cookie_map.get(&cookie) {
            if timestamp.describes_valid_cookie() {
                return Some(user_data);
            }
        }

        None
    }

    pub fn new(user_cache: UserDataCache) -> Self {
        Self {
            user_cache,
            cookie_map: HashMap::new()
        }
    }
}

impl AuthenticatedUser {
    fn authenticate_new_user(user_data: &UserData, password: &str) -> Option<Self> {
        if user_data.authenticate(password) {
            return Some(Self(user_data.clone()));
        }

        None
    }
}

// Von SystemTime ausgehend; in ms
fn get_current_time() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 // TODO: nachgucken, ob unwrap hier sicher ist
}

impl Cookie {
    fn new() -> Self {
        let rng = StdRng::seed_from_u64(get_current_time());
        let s: String = rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        Cookie(s)
    }
}

impl Timestamp {
    fn now() -> Self {
        Timestamp(get_current_time() / 1000) // Zeit in Sekunden angeben
    }

    fn describes_valid_cookie(&self) -> bool {
        self.0 - Self::now().0 <= COOKIE_LIVE_TIME
    }
}

#[cfg(test)]
mod tests {
    use super::{ Timestamp, Cookie, COOKIE_LIVE_TIME, AuthenticatedUser, AuthCookieMap };
    use crate::auth::{ UserDataCache, UserData };

    fn helper_create_user_data_cache() -> UserDataCache {
        let mut users = UserDataCache::new();
        users.add(UserData::new("user1".to_string(), "password1".to_string(), vec!()));
        users.add(UserData::new("user2".to_string(), "password2".to_string(), vec!()));
        users
    }

    #[test]
    fn test_timestamp_describes_valid_cookie_decayed() {
        let now_unix = Timestamp::now().0;
        let decayed_timestamp = Timestamp(now_unix + COOKIE_LIVE_TIME + 1);

        assert!(!decayed_timestamp.describes_valid_cookie())
    }

    #[test]
    fn test_timestamp_describes_valid_cookie_alive() {
        let now_unix = Timestamp::now().0;
        let alive_timestamp = Timestamp(now_unix + COOKIE_LIVE_TIME / 2);

        assert!(alive_timestamp.describes_valid_cookie())
    }

    #[test]
    fn test_authenticated_user_authenticate_new_user_wrong_password() {
        let users = helper_create_user_data_cache();
        let result = AuthenticatedUser::authenticate_new_user(users.get_user("user1").unwrap(), "password2");

        assert!(result.is_none());
    }

    #[test]
    fn test_authenticated_user_authenticate_new_user_correct_password() {
        let users = helper_create_user_data_cache();
        let result = AuthenticatedUser::authenticate_new_user(users.get_user("user2").unwrap(), "password2");

        assert!(result.is_some());
    }

    #[test]
    fn test_auth_cookie_map_lookup_cookie_found() {
        let users = helper_create_user_data_cache();
        let mut cookie_map = AuthCookieMap::new(users);
        
        let cookie = cookie_map.add_new_cookie("user1", "password1").unwrap();

        assert!(cookie_map.lookup_cookie(cookie).is_some());
    }

    #[test]
    fn test_auth_cookie_map_lookup_cookie_missing() {
        let users = helper_create_user_data_cache();
        let cookie_map = AuthCookieMap::new(users);

        let cookie = Cookie::new();

        assert!(cookie_map.lookup_cookie(cookie).is_none());
    }

    #[test]
    fn test_auth_cookie_map_cleanup() {
        let users = helper_create_user_data_cache();
        let mut cookie_map = AuthCookieMap::new(users);

        // Fuege Cookies fuer beide Nutzer hinzu
        let user1_cookie = cookie_map.add_new_cookie("user1", "password1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1)); // Eine Millisekunde warten, sodass die Cookies nicht gleich sind 
        let user2_cookie = cookie_map.add_new_cookie("user2", "password2").unwrap();

        // Veraendere die Zeit vom zweiten Nutzer, sodass der Cookie ungueltig ist
        let (user2, timestamp) = cookie_map.cookie_map.get(&user2_cookie).unwrap();
        let user2 = user2.0.clone();
        let timestamp = Timestamp(timestamp.0 + COOKIE_LIVE_TIME + 1);
        cookie_map.cookie_map.insert(user2_cookie, (AuthenticatedUser(user2), timestamp));

        cookie_map.cleanup();

        assert_eq!(cookie_map.cookie_map.keys().collect::<Vec<&Cookie>>(), vec![ &user1_cookie ])
    }
}