use enigo::*;
use inputbot::{KeySequence, KeybdKey::*, MouseButton::*};
use mki::Sequence;
use std::{thread::sleep, time::Duration};
use winput::Vk;

fn main() {
    // Bind the number 1 key your keyboard to a function that types
    // "Hello, world!" when pressed.
    // Numrow1Key.bind(|| KeySequence("Hello, world!").send());

    // Bind your caps lock key to a function that starts an autoclicker.
    CapsLockKey.bind(move || {
        // let mut enigo = Enigo::new();
        // simple
        use sysinputs::keyboard::{send_char, send_str};
        // medium
        use sysinputs::keyboard::{send_combo, send_key, Key, Physical};
        // complicated
        use sysinputs::keyboard::{press_key, release_key};
        while CapsLockKey.is_toggled() {
            // enigo.key_down(Key::W);
            // sleep(Duration::from_millis(50));
            // enigo.key_up(Key::W);

            // sleep(Duration::from_millis(50));

            // enigo.key_down(Key::S);
            // sleep(Duration::from_millis(50));
            // enigo.key_up(Key::S);

            // sleep(Duration::from_millis(50));

            // enigo.key_down(Key::A);
            // sleep(Duration::from_millis(50));
            // enigo.key_up(Key::A);

            // sleep(Duration::from_millis(50));

            // enigo.key_down(Key::D);
            // sleep(Duration::from_millis(50));
            // enigo.key_up(Key::D);
            sleep(Duration::from_millis(70));

            // send_combo(&[
            //     Key::Physical(Physical::W),
            //     Key::Physical(Physical::S),
            //     Key::Physical(Physical::D),
            // ]);

            // Sequence::text("WSDW").unwrap().send();
            // use windows::Win32::UI::Input::KeyboardAndMouse::{
            //     KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_LSHIFT, VK_W,
            // };

            // unsafe {
            //     windows::Win32::UI::Input::KeyboardAndMouse::keybd_event(
            //         VK_W.0 as u8,
            //         0,
            //         KEYBD_EVENT_FLAGS::default(),
            //         0,
            //     )
            // };
            // sleep(Duration::from_millis(20));

            // unsafe {
            //     windows::Win32::UI::Input::KeyboardAndMouse::keybd_event(
            //         VK_W.0 as u8,
            //         0,
            //         KEYEVENTF_KEYUP,
            //         0,
            //     )
            // };

            fn press_key(key: u16, flags: u32) -> anyhow::Result<()> {
                use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT};

                let mut input_u: INPUT_u = unsafe { std::mem::zeroed() };
                unsafe {
                    *input_u.ki_mut() = KEYBDINPUT {
                        wVk: 0,
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
            press_key(0x1E, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x1E, KEYUP);

            // winput::send_keys(vec![Vk::A, Vk::W, Vk::S, Vk::D]);

            // mki::Keyboard::W.press();
            // sleep(Duration::from_millis(70));
            // mki::Keyboard::W.release();

            // mki::Keyboard::A.press();
            // sleep(Duration::from_millis(70));
            // mki::Keyboard::A.release();

            // mki::Keyboard::D.press();
            // sleep(Duration::from_millis(70));
            // mki::Keyboard::D.release();

            // sleep(Duration::from_millis(70));

            // mki::Keyboard::W.press();
            // sleep(Duration::from_millis(20));
            // mki::Keyboard::W.release();

            // winput::release(Vk::W);
            // winput::press(Vk::W);
            // sleep(Duration::from_millis(70));
            // winput::release(Vk::W);

            // winput::release(Vk::S);
            // winput::press(Vk::S);
            // sleep(Duration::from_millis(70));
            // winput::release(Vk::S);

            // winput::release(Vk::A);
            // winput::press(Vk::A);
            // sleep(Duration::from_millis(70));
            // winput::release(Vk::A);

            // winput::release(Vk::D);
            // winput::press(Vk::D);
            // sleep(Duration::from_millis(70));
            // winput::release(Vk::D);

            sleep(Duration::from_millis(70));
            // winput::send_str("wasd wasd");
        }
    });

    // Call this to start listening for bound inputs.
    inputbot::handle_input_events();
}
