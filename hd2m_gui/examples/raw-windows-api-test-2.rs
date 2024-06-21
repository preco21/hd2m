use enigo::*;
use inputbot::{KeySequence, KeybdKey::*, MouseButton::*};
use mki::Sequence;
use std::{thread::sleep, time::Duration};
use winapi::shared::ntdef::LPSTR;
use windows::{
    core::PCSTR,
    Win32::Foundation::{HWND, LPARAM, WPARAM},
};
use winput::Vk;

fn main() {
    // Bind the number 1 key your keyboard to a function that types
    // "Hello, world!" when pressed.
    // Numrow1Key.bind(|| KeySequence("Hello, world!").send());

    use windows::{
        core::*, Win32::System::Threading::*, Win32::UI::Input::KeyboardAndMouse::*,
        Win32::UI::WindowsAndMessaging::*,
    };
    let hwnd: HWND = unsafe { FindWindowW(None, w!("HELLDIVERS")) };

    // Bind your caps lock key to a function that starts an autoclicker.
    // CapsLockKey.bind(move || {
    //     while CapsLockKey.is_toggled() {
    use windows::{
        core::*, Win32::System::Threading::*, Win32::UI::Input::KeyboardAndMouse::*,
        Win32::UI::WindowsAndMessaging::*,
    };

    // loop {
    println!("run");

    sleep(Duration::from_millis(1000));

    unsafe {
        // let window_title = "HELLDIVERS™ 2";
        // let window_title_wide: Vec<u16> =
        //     window_title.encode_utf16().chain(Some(0)).collect();

        // let window =
        //     windows::Win32::UI::WindowsAndMessaging::FindWindowA(None, process_name);

        // Call the FindWindowW function to find the window handle
        // println!(" {:?}", error); //0

        println!("hwnd: {:?}", hwnd);

        fn press_key(key: u16, flags: u32) -> anyhow::Result<()> {
            use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT};

            let mut input_u: INPUT_u = unsafe { std::mem::zeroed() };
            unsafe {
                *input_u.ki_mut() = KEYBDINPUT {
                    wVk: 0x57,
                    dwExtraInfo: 0,
                    wScan: 0x11,
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

        // let wh = GetActiveWindow();
        let vk = 0x57;
        let sc = 0x11;
        // let res = PostMessageA(hwnd, WM_KEYDOWN, WPARAM(vk), LPARAM(sc << 16));
        let res = PostMessageA(
            hwnd,
            WM_KEYDOWN,
            WPARAM(vk),
            LPARAM((sc << 16) | 0x00000001),
        );
        println!("res: {:?}", res);
        sleep(Duration::from_millis(130));
        // let res2 = PostMessageA(hwnd, WM_KEYUP, WPARAM(vk), LPARAM(sc << 16));
        let res2 = PostMessageA(hwnd, WM_KEYUP, WPARAM(vk), LPARAM((sc << 16) | 0xC0000001));
        println!("res: {:?}", res2);

        std::thread::spawn(move || {
            const KEYUP: u32 = 0x0002;
            press_key(0x57, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x57, KEYUP);

            sleep(Duration::from_millis(70));

            press_key(0x57, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x57, KEYUP);

            sleep(Duration::from_millis(70));

            press_key(0x57, 0);
            println!("dw");
            std::thread::sleep(std::time::Duration::from_millis(70));
            println!("up");
            press_key(0x57, KEYUP);
        });

        sleep(Duration::from_millis(130));

        // let vk = 0x41;
        // let sc = 0x1E;
        // let res = PostMessageA(
        //     hwnd,
        //     WM_KEYDOWN,
        //     WPARAM(vk),
        //     LPARAM((sc << 16) | 0x00000001),
        // );
        // println!("res: {:?}", res);
        // sleep(Duration::from_millis(130));
        // // let res2 = PostMessageA(hwnd, WM_KEYUP, WPARAM(vk), LPARAM(sc << 16));
        // let res2 = PostMessageA(hwnd, WM_KEYUP, WPARAM(vk), LPARAM((sc << 16) | 0xC0000001));
        // println!("res: {:?}", res2);
        // let error = windows::Win32::Foundation::GetLastError();
        // println!("error: {:?}", error); //02
    }

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
    sleep(Duration::from_millis(1000));

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

    // fn press_key(key: u16, flags: u32) -> anyhow::Result<()> {
    //     use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT};

    //     let mut input_u: INPUT_u = unsafe { std::mem::zeroed() };
    //     unsafe {
    //         *input_u.ki_mut() = KEYBDINPUT {
    //             wVk: 0,
    //             dwExtraInfo: 0,
    //             wScan: key,
    //             time: 0,
    //             dwFlags: flags | 0x0008,
    //         }
    //     }

    //     let mut input = INPUT {
    //         type_: INPUT_KEYBOARD,
    //         u: input_u,
    //     };
    //     let ipsize = std::mem::size_of::<INPUT>() as i32;
    //     unsafe {
    //         SendInput(1, &mut input, ipsize);
    //     };
    //     Ok(())
    // }

    // const KEYUP: u32 = 0x0002;
    // press_key(0x1E, 0);
    // println!("dw");
    // std::thread::sleep(std::time::Duration::from_millis(70));
    // println!("up");
    // press_key(0x1E, KEYUP);

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

    //         sleep(Duration::from_millis(70));
    //         // winput::send_str("wasd wasd");
    //     }
    // });
    // }
    // Call this to start listening for bound inputs.
    // inputbot::handle_input_events();
}
