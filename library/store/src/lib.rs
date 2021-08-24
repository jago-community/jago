/// Morning, so far, has consisted of this:
///
/// - 06:00: alarm clock goes off.
///
/// Soothing moment. I no longer use the alarm in the clock app. I use the special bedtime section
/// in the health feature. Sure, the sleep metrics are interesting but what got me are the noise
/// options. My use of noise might be drawn from being jolted awake by any kind of alarm I've used
/// so far in life aside from the periods of time I was able to wake up to a biological clock.
///
/// I no longer go to school feeling angry from being jolted awake by the crude electronic
/// emissions inspired by the requirements of the early analogue electronics. The health feature
/// still uses electronically generated audio so that the software's writers don't have to pay the
/// artists but the inspiration definitely comes from music.
mod store;

use ring::{digest, pbkdf2};
use std::{collections::HashMap, num::NonZeroU32};

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
pub type Credential = [u8; CREDENTIAL_LEN];

author::error!(WrongUsernameOrPassword);

struct PasswordDatabase {
    pbkdf2_iterations: NonZeroU32,
    db_salt_component: [u8; 16],

    // Normally this would be a persistent database.
    storage: HashMap<String, Credential>,
}

impl PasswordDatabase {
    pub fn store_password(&mut self, username: &str, password: &str) {
        let salt = self.salt(username);
        let mut to_store: Credential = [0u8; CREDENTIAL_LEN];
        pbkdf2::derive(
            PBKDF2_ALG,
            self.pbkdf2_iterations,
            &salt,
            password.as_bytes(),
            &mut to_store,
        );
        self.storage.insert(String::from(username), to_store);
    }

    pub fn verify_password(&self, username: &str, attempted_password: &str) -> Result<(), Error> {
        match self.storage.get(username) {
            Some(actual_password) => {
                let salt = self.salt(username);
                pbkdf2::verify(
                    PBKDF2_ALG,
                    self.pbkdf2_iterations,
                    &salt,
                    attempted_password.as_bytes(),
                    actual_password,
                )
                .map_err(|_| Error::WrongUsernameOrPassword)
            }

            None => Err(Error::WrongUsernameOrPassword),
        }
    }

    // The salt should have a user-specific component so that an attacker
    // cannot crack one password for multiple users in the database. It
    // should have a database-unique component so that an attacker cannot
    // crack the same user's password across databases in the unfortunate
    // but common case that the user has used the same password for
    // multiple systems.
    fn salt(&self, username: &str) -> Vec<u8> {
        let mut salt = Vec::with_capacity(self.db_salt_component.len() + username.as_bytes().len());
        salt.extend(self.db_salt_component.as_ref());
        salt.extend(username.as_bytes());
        salt
    }
}

use rand::Rng;

#[test]
fn test_database() {
    // Normally these parameters would be loaded from a configuration file.
    let mut db = PasswordDatabase {
        pbkdf2_iterations: NonZeroU32::new(100_000).unwrap(),
        db_salt_component: get_salt(),
        storage: HashMap::new(),
    };

    db.store_password("alice", "@74d7]404j|W}6u");

    // An attempt to log in with the wrong password fails.
    assert!(db.verify_password("alice", "wrong password").is_err());

    // Normally there should be an expoentially-increasing delay between
    // attempts to further protect against online attacks.

    // An attempt to log in with the right password succeeds.
    assert!(db.verify_password("alice", "@74d7]404j|W}6u").is_ok());
}

fn get_salt() -> [u8; 16] {
    let initial = "bG8ZcILW+BdtHig=".as_bytes();

    let mut output: [u8; 16] = Default::default();

    output.copy_from_slice(&initial[..]);

    output
}
