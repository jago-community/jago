mod memory;

author::error!(
    Incomplete,
    std::io::Error,
    std::env::VarError,
    context::Error,
    memory::Error,
);

#[test]
fn test_secret() {
    use ring::{rand::SystemRandom, signature::KeyPair};

    use std::{borrow::BorrowMut, io::Read};

    let might_after = context::before().unwrap();

    let mut identity_password = rpassword::prompt_password_stdout("Password: ")
        .unwrap()
        .into_bytes();

    let mut password = secret(&mut identity_password).unwrap();

    if let Some(after) = might_after {
        after();
    }

    let password = &mut password.borrow_mut();

    let identity = std::env::var("IDENTITY").unwrap();

    let mut private_key = vec![];

    let mut private_key_file = std::fs::File::open(&identity).unwrap();
    private_key_file.read_to_end(&mut private_key).unwrap();

    let key_pair = ring::signature::RsaKeyPair::from_pkcs8(&private_key).unwrap();

    let random = SystemRandom::new();

    let public_key = key_pair.public_key();

    let _signature = key_pair.sign(
        &ring::signature::RSA_PKCS1_SHA256,
        &random,
        public_key.as_ref(),
        password,
    );

    //assert_eq!(password, b"hello world");
}

pub fn secret<'a>(buffer: &'a mut [u8]) -> Result<Secret<'a>, Error> {
    memory::lock(buffer)?;

    Ok(Secret { source: buffer })
}

use std::borrow::{Borrow, BorrowMut};

pub struct Secret<'a> {
    source: &'a mut [u8],
}

impl<'a> Borrow<[u8]> for Secret<'a> {
    fn borrow(&self) -> &[u8] {
        &self.source[..]
    }
}

impl<'a> BorrowMut<[u8]> for Secret<'a> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.source
    }
}

impl Drop for Secret<'_> {
    fn drop(&mut self) {
        match memory::unlock(self.source) {
            Err(error) => log::error!("error unlocking memory: {}", error),
            _ => {}
        }
    }
}
