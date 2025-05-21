use libc::{c_void, ftruncate, mmap, munmap, shm_open, shm_unlink, MAP_FAILED, MAP_PRIVATE, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR, PROT_READ, PROT_WRITE};

#[derive(Clone)]
pub struct ShmPool {
    pub fd:     i32,
    pub addr:   *mut c_void,
    pub size:   usize,
}

impl ShmPool {
    pub fn new(size: usize) -> std::io::Result<ShmPool> {
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
            addr,
            size,
        })
    }

    pub fn from_fd(fd: i32, size: usize) -> std::io::Result<ShmPool> {
        let addr = unsafe { mmap(std::ptr::null_mut(), size, PROT_READ, MAP_PRIVATE, fd, 0) };
        if addr == MAP_FAILED {
            eprint!("mmap in ShmPool::from_fd() failed with fd {}", fd);
            return Err(std::io::Error::last_os_error());
        }
        Ok(ShmPool {fd, addr, size})
    }

    pub fn read_string(&self, offset: usize) -> std::io::Result<String> {
        let mut res: Vec<u8> = Vec::new();
        for i in offset..self.size {
            let byte = unsafe {*(self.addr.offset(i as isize) as *const u8)};
            if byte == 0 {
                break;
            }
            res.push(byte);
        };
        Ok(String::from_utf8(res).unwrap())
    }

    pub fn resize(&mut self, size: usize) -> std::io::Result<()> {
        if unsafe { ftruncate(self.fd, size as i64) } == -1 {
            return Err(std::io::Error::last_os_error())
        };
        self.size = size;

        Ok(())
    }

    pub fn write(&mut self, data: &Vec<u32>, offset: usize) {
        if offset + data.len() >= self.size {
            return;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u32, // src: data as *const u32
                self.addr.offset(4*offset as isize) as *mut u32, // dst: ShmPool address as *mut u32
                data.len()
            );
        }
    }

    pub fn write_pixel(&mut self, data: u32, offset: usize) {
        // TODO: Return error if out of bounds
        if offset + 3 > self.size {
            return;
        }
        unsafe {*(self.addr.offset(offset as isize*4) as *mut u32) = data;}
    }

    pub fn read_pixel(&self, offset: usize) -> Option<u32> {
        if offset + 3 > self.size {
            return None;
        }
        return Some(unsafe {*(self.addr.offset(4*offset as isize) as *const u32)});
    }
}

impl Drop for ShmPool {
    fn drop(&mut self) {
        println!("Dropping ShmPool!");
        unsafe { munmap(self.addr, self.size); }
    }
}

unsafe impl Send for ShmPool {}
unsafe impl Sync for ShmPool {}
