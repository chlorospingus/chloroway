use libc::{c_void, ftruncate, mmap, munmap, shm_open, shm_unlink, MAP_FAILED, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR, PROT_READ, PROT_WRITE};

#[derive(Clone)]
pub struct ShmPool {
    pub fd: i32,
    pub id: u32,
    pub addr: *mut c_void,
    pub size: usize,
}

impl ShmPool {
    pub fn new(size: usize, id: u32) -> std::io::Result<ShmPool> {
        let shm_path: *const i8 = b"/chlorostart\0".as_ptr() as *const i8;
        let fd = unsafe { shm_open(shm_path, O_RDWR | O_EXCL | O_CREAT, 0o600) };
        if fd == -1 {
            eprint!("shm_open in ShmPool::new() failed: ");
            return Err(std::io::Error::last_os_error())
        }
        if unsafe { shm_unlink(shm_path) } == -1 {
            eprint!("shm_unlink in ShmPool::new() failed: ");
            return Err(std::io::Error::last_os_error())
        }
        if unsafe { ftruncate(fd, size as i64) } == -1 {
            eprint!("ftruncate in ShmPool::new() failed: ");
            return Err(std::io::Error::last_os_error())
        }
        let addr = unsafe {
            mmap(std::ptr::null_mut(), size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
        };

        if addr == MAP_FAILED {
            eprint!("mmap in ShmPool::new() failed: ");
            return Err(std::io::Error::last_os_error())
        }

        Ok(ShmPool {
            fd,
            id,
            addr,
            size 
        })
    }

    pub fn resize(&mut self, size: usize) -> std::io::Result<()> {
        if unsafe { ftruncate(self.fd, size as i64) } == -1 {
            return Err(std::io::Error::last_os_error())
        };
        self.size = size;

        Ok(())
    }

    pub fn write(&mut self, data: &Vec<u32>, offset: isize) -> std::io::Result<()> {
        // TODO: Bounds check
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u32, // src: data as *const u32
                self.addr.offset(offset*4) as *mut u32, // dst: ShmPool address as *mut u32
                data.len()
            );
        }

        Ok(())
    }
}

impl Drop for ShmPool {
    fn drop(&mut self) {
        unsafe { munmap(self.addr, self.size); }
    }
}
