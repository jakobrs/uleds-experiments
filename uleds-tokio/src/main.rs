use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::unix::prelude::AsRawFd,
};

use tokio::{
    io::{unix::AsyncFd, Interest},
    signal::unix::{signal, SignalKind},
};

struct Uled {
    inner: AsyncFd<File>,
}

impl Uled {
    fn new(name: &str) -> Self {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open("/dev/uleds")
            .unwrap();

        unsafe {
            let flags = libc::fcntl(file.as_raw_fd(), libc::F_GETFL);
            if libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
                panic!("{:?}", io::Error::last_os_error());
            }
        }

        #[repr(C)]
        struct UledDeclaration {
            name: [u8; 64],
            max_brightness: u32,
        }

        let mut uled_declaration = UledDeclaration {
            name: [0; 64],
            max_brightness: 1000,
        };

        let name_bytes = name.as_bytes();
        uled_declaration.name[0..name_bytes.len()].copy_from_slice(name_bytes);

        unsafe {
            let data = &uled_declaration as *const _ as *const u8;
            file.write(std::slice::from_raw_parts(
                data,
                std::mem::size_of::<UledDeclaration>(),
            ))
            .unwrap();
        }

        Self {
            inner: AsyncFd::with_interest(file, Interest::READABLE).unwrap(),
        }
    }

    pub async fn read(&self) -> io::Result<u32> {
        loop {
            let mut guard = self.inner.readable().await?;

            let mut buf = [0u8; 4];

            match guard.try_io(|inner| inner.get_ref().read_exact(&mut buf)) {
                Ok(res) => return res.map(|_| u32::from_ne_bytes(buf)),
                Err(_would_block) => continue,
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    for n in 0..1000 {
        tokio::task::spawn(async move {
            let uled = Uled::new(format!("neopixel::led{}", n).as_str());

            while let Ok(brightness) = uled.read().await {
                println!("Brightness of LED {} changed to {}", n, brightness);
            }
        });
    }

    signal(SignalKind::interrupt()).unwrap().recv().await;
}
