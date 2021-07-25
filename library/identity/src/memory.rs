author::error!(Unknown);

pub fn lock(buffer: &mut [u8]) -> Result<(), Error> {
    use libc::{c_int, c_void, mlock};

    let result: c_int = unsafe { mlock(buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    match result {
        0 => Ok(()),
        _ => Err(Error::Unknown),
    }
}

pub fn unlock(buffer: &mut [u8]) -> Result<(), Error> {
    use libc::{c_int, c_void, munlock};

    let result: c_int = unsafe { munlock(buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    match result {
        0 => Ok(()),
        _ => Err(Error::Unknown),
    }
}
