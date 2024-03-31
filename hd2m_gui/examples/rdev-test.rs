use enigo::*;
use inputbot::{KeySequence, KeybdKey::*, MouseButton::*};
use mki::Sequence;
use std::{thread::sleep, time::Duration};
use winput::Vk;

fn main() {
    use rdev::{grab, Event, EventType, Key};

    let callback = |event: Event| -> Option<Event> {
        if let EventType::KeyPress(Key::CapsLock) = event.event_type {
            fn press_key(key: u16, flags: u32) -> anyhow::Result<()> {
                use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT};

                let mut input_u: INPUT_u = unsafe { std::mem::zeroed() };
                unsafe {
                    *input_u.ki_mut() = KEYBDINPUT {
                        wVk: 0x57,
                        dwExtraInfo: 0,
                        wScan: key,
                        time: 0,
                        dwFlags: flags | 0x0008,
                    }
                }

                let mut input = INPUT {
                    type_: INPUT_KEYBOARD,
                    u: input_u,
                };
                let ipsize = std::mem::size_of::<INPUT>() as i32;
                unsafe {
                    SendInput(1, &mut input, ipsize);
                };
                Ok(())
            }

            const KEYUP: u32 = 0x0002;
            std::thread::sleep(std::time::Duration::from_millis(70));
            press_key(0x11, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x11, KEYUP);
            std::thread::sleep(std::time::Duration::from_millis(70));
            press_key(0x11, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x11, KEYUP);
            std::thread::sleep(std::time::Duration::from_millis(70));
            press_key(0x11, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x11, KEYUP);

            // None // CapsLock is now effectively disabled
            Some(event)
        } else {
            Some(event)
        }
    };
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}
