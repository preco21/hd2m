use std::{thread::sleep, time::Duration};

use windows::{
    core::*,
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::Threading::*,
        UI::{
            Input::KeyboardAndMouse::{
                GetActiveWindow, SetKeyboardState, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_W,
            },
            WindowsAndMessaging::*,
        },
    },
};

fn main() {
    const KEYUP: u32 = 0x0002;
    loop {
        std::thread::sleep(Duration::from_millis(3000));

        // let wh = GetActiveWindow();
        let vk = 0x57;
        let sc = 0x11;
        unsafe {
            let hwnd: HWND = unsafe { FindWindowW(None, w!("HELLDIVERS")) };

            // SetKeyboardState(&[0; 256]);
            // println!("hwnd: {:?}", hwnd);
            // let res2 = PostMessageA(hwnd, WM_KEYUP, WPARAM(vk), LPARAM(sc >> 16));

            let res = PostMessageA(
                hwnd,
                WM_KEYDOWN,
                WPARAM(vk),
                LPARAM((sc << 16) | 0x00000001),
            );
            println!("res: {:?}", res);
            sleep(Duration::from_millis(100));
            let res2 = PostMessageA(hwnd, WM_KEYUP, WPARAM(vk), LPARAM((sc << 16) | 0xC0000001));
            println!("res: {:?}", res2);
            sleep(Duration::from_millis(1000));
        }
    }

    // unsafe {
    //     windows::Win32::UI::Input::KeyboardAndMouse::keybd_event(
    //         VK_W.0 as u8,
    //         0,
    //         KEYBD_EVENT_FLAGS::default(),
    //         0,
    //     )
    // };
    // sleep(Duration::from_millis(100));

    // unsafe {
    //     windows::Win32::UI::Input::KeyboardAndMouse::keybd_event(
    //         VK_W.0 as u8,
    //         0,
    //         KEYEVENTF_KEYUP,
    //         0,
    //     )
    // };

    // press_key(0x57, 0);
    // std::thread::sleep(std::time::Duration::from_millis(120));
    // press_key(0x57, KEYUP);

    // std::thread::sleep(std::time::Duration::from_millis(120));

    // press_key(0x57, 0);
    // std::thread::sleep(std::time::Duration::from_millis(120));
    // press_key(0x57, KEYUP);

    // std::thread::sleep(std::time::Duration::from_millis(120));

    // press_key(0x57, 0);
    // std::thread::sleep(std::time::Duration::from_millis(120));
    // press_key(0x57, KEYUP);

    std::thread::sleep(std::time::Duration::from_millis(1000));
}

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
