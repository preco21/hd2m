use enigo::*;
use inputbot::{KeySequence, KeybdKey::*, MouseButton::*};
use mki::Sequence;
use std::{thread::sleep, time::Duration};
use windows::Win32::Foundation::WPARAM;
use winput::Vk;

fn main() {
    // Bind the number 1 key your keyboard to a function that types
    // "Hello, world!" when pressed.
    // Numrow1Key.bind(|| KeySequence("Hello, world!").send());

    // Bind your caps lock key to a function that starts an autoclicker.
    CapsLockKey.bind(move || {
        while CapsLockKey.is_toggled() {
            let i = interception::Interception::new().unwrap();
            i.send(
                0x04D8,
                &[interception::Stroke::Keyboard {
                    code: interception::ScanCode::W,
                    state: interception::KeyState::DOWN,
                    information: 0,
                }],
            );

            sleep(Duration::from_millis(70));
            // winput::send_str("wasd wasd");
        }
    });

    // Call this to start listening for bound inputs.
    inputbot::handle_input_events();
}
