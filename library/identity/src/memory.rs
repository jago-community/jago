author::error!(Unknown);

use libc::{c_int, c_void, mlock, munlock};

pub fn lock(address: *mut c_void, offset: usize) -> Result<(), Error> {
    let result: c_int = unsafe { mlock(address, offset) };

    match result {
        0 => Ok(()),
        _ => Err(Error::Unknown),
    }
}

pub fn unlock(address: *mut c_void, offset: usize) -> Result<(), Error> {
    let result: c_int = unsafe { munlock(address, offset) };

    match result {
        0 => Ok(()),
        _ => Err(Error::Unknown),
    }
}
