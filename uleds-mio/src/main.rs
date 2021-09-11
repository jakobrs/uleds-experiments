use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::unix::prelude::AsRawFd,
};

use mio::{unix::SourceFd, Events, Interest, Poll};

fn main() {
    let mut poll = Poll::new().unwrap();
    let registry = poll.registry();
    let mut events = Events::with_capacity(20);

    let mut uleds = Vec::with_capacity(1000);

    for n in 0..1000 {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open("/dev/uleds")
            .unwrap();

        #[repr(C)]
        struct UledDeclaration {
            name: [u8; 64],
            max_brightness: u32,
        }

        let mut uled_declaration = UledDeclaration {
            name: [0; 64],
            max_brightness: 1000,
        };

        let name = format!("neopixel::led{}", n);
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

        registry
            .register(
                &mut SourceFd(&file.as_raw_fd()),
                mio::Token(n),
                Interest::READABLE,
            )
            .unwrap();

        uleds.push(file);
    }

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            let n = event.token().0;

            let mut brightness_buf = [0u8; 4];
            uleds[n].read_exact(&mut brightness_buf).unwrap();
            let brightness = u32::from_ne_bytes(brightness_buf);

            println!("Set brightness on led {} to {}", n, brightness);
        }
    }
}
