author::error!(
    Incomplete,
    std::io::Error,
    std::env::VarError,
    context::Error,
    crate::memory::Error,
    std::cell::BorrowMutError,
);

pub struct Secret {
    address: *mut u8,
    offset: usize,
}

impl Secret {
    pub fn new<I>(buffer: &mut [u8]) -> Result<Secret, Error> {
        let (address, offset) = (buffer.as_mut_ptr(), buffer.len());

        crate::memory::lock(address as *mut libc::c_void, offset)?;

        Ok(Secret { address, offset })
    }
}

impl std::borrow::Borrow<[u8]> for Secret {
    fn borrow(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.address, self.offset) }
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        match crate::memory::unlock(self.address as *mut libc::c_void, self.offset) {
            Err(error) => log::error!("error unlocking memory: {}", error),
            _ => {}
        }
    }
}
