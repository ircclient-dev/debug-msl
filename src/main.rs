use windows::core::{w, PCSTR, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE, HWND, LPARAM, WPARAM};
use windows::Win32::Globalization::{WideCharToMultiByte, CP_UTF8};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::GetProcessHandleFromHwnd;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, IsWindowVisible, SendMessageW, WM_USER,
};

const WM_MCOMMAND: u32 = WM_USER + 200;
const WM_MEVALUATE: u32 = WM_USER + 201;

fn main() {
    // Make sure we've received arguments
    // if std::env::args().len() == 1 {
    //     println!("Usage: debug-msl.exe <mSL code to run>");
    //     return;
    // }

    if unsafe { EnumWindows(Some(enum_windows_proc), LPARAM(0)).is_err() } {
        println!("Unable to enumerate windows");
    }
}

extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    unsafe {
        // Ensure the window is visible
        if IsWindowVisible(hwnd).as_bool() {
            // Get process handle from window handle
            let process_handle = GetProcessHandleFromHwnd(hwnd);
            if !process_handle.is_invalid() {
                // Get the process name from the process handle
                let mut process_name = [0; 256];
                let process_name_length =
                    GetModuleBaseNameW(process_handle, None, &mut process_name);
                if process_name_length > 0 {
                    let process_name_slice = &process_name[..process_name_length as usize];
                    let process_name = PCWSTR(process_name_slice.as_ptr()).to_string().unwrap();

                    // Check if the process is mIRC or AdiIRC
                    if process_name.to_lowercase() == "mirc.exe"
                        || process_name.to_lowercase() == "adiirc.exe"
                    {
                        // Send a message to the IRC client to evaluate the version and bitness
                        let m_eval = send_message_to_irc_client(
                            hwnd,
                            WM_MEVALUATE,
                            &w!("$+(v,$version $chr(40),$bits bit,$chr(41))"),
                        );
                        match m_eval {
                            Ok(m_eval) => {
                                // We got a reply from the IRC client, so we know it's valid.
                                let m_result = PCWSTR(m_eval.as_ptr());

                                let args: Vec<String> = std::env::args().collect();
                                let args_string = args[1..].join(" ");

                                let message = format!("//{}\0", args_string);
                                let mut message_wide: Vec<u16> = message.encode_utf16().collect();
                                message_wide.push(0); // Our Vec<u16> needs to be null-terminated to be a valid PCWSTR;
                                let message_pcwstr = PCWSTR(message_wide.as_ptr());
                                let _ =
                                    send_message_to_irc_client(hwnd, WM_MCOMMAND, &message_pcwstr);

                                // Let the user know we've sent the command
                                println!(
                                    "Sent command to {} {}",
                                    process_name,
                                    m_result.to_string().unwrap()
                                );
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        }
    }
    BOOL(1) // Continue enumeration
}

fn send_message_to_irc_client(
    hwnd: HWND,
    message_type: u32,
    message: &PCWSTR,
) -> Result<Vec<u16>, String> {
    // Sanity check
    if (message_type != WM_MCOMMAND) && (message_type != WM_MEVALUATE) {
        return Err("Invalid message type".to_string());
    }
    unsafe {
        // Create a mapped file
        let file_mapping = CreateFileMappingW(
            HANDLE::default(),
            None,
            PAGE_READWRITE,
            0,
            4096,
            PCWSTR::from_raw(w!("mIRC").as_ptr()),
        );

        let file_mapping = match file_mapping {
            Ok(handle) => handle,
            Err(_) => panic!("Failed to create file mapping"),
        };

        let map_view = MapViewOfFile(file_mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
        if map_view.Value.is_null() {
            return Err("Failed to map view of file".to_string());
        }

        // // Ideally, we would like to send our Unicode string, but AdiIRC has a bug that prevents it from working.
        // // https://dev.adiirc.com/issues/5812
        // std::ptr::copy_nonoverlapping(message.as_ptr(), map_view.Value as *mut u16, message.len());
        // let result = SendMessageW(hwnd, WM_MEVALUATE, Some(WPARAM(8)), Some(LPARAM(0)));
        // let reply = PCWSTR(map_view.Value as *const u16);

        ////////////////////////////////
        // START OF ADIIRC WORKAROUND //
        // START OF ADIIRC WORKAROUND //
        // START OF ADIIRC WORKAROUND //
        ////////////////////////////////

        // Convert the Unicode string to UTF-8
        let message_slice = std::slice::from_raw_parts(message.0, message.len() as usize);
        let message_mb_len = WideCharToMultiByte(CP_UTF8, 0, message_slice, None, None, None);
        let mut message_mb_vec = vec![0u8; message_mb_len as usize + 1];
        WideCharToMultiByte(
            CP_UTF8,
            0,
            message_slice,
            Some(&mut message_mb_vec),
            None,
            None,
        );

        // Copy the UTF-8 string to the mapped file
        std::ptr::copy_nonoverlapping(
            message_mb_vec.as_ptr(),
            map_view.Value as *mut u8,
            (message_mb_len as usize) + 1,
        );

        // Send the message to IRC Client
        let result = SendMessageW(hwnd, message_type, Some(WPARAM(0)), Some(LPARAM(0)));
        if result.0 == 0 {
            return Err("Failed to send message to IRC Client".to_string());
        }

        // Get the reply from IRC Client
        let reply_multibyte: PCSTR = PCSTR(map_view.Value as *const u8);
        let reply_str = reply_multibyte.to_string().unwrap();
        let mut reply_wide: Vec<u16> = reply_str.encode_utf16().collect();
        reply_wide.push(0); // Our Vec<u16> needs to be null-terminated to be a valid PCWSTR

        //////////////////////////////
        // END OF ADIIRC WORKAROUND //
        // END OF ADIIRC WORKAROUND //
        // END OF ADIIRC WORKAROUND //
        //////////////////////////////

        // Clean up
        let _ = UnmapViewOfFile(map_view);
        Ok(reply_wide)
    }
}
