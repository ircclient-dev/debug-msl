use windows::core::{w, PCSTR, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE, HWND, LPARAM, WPARAM};
use windows::Win32::Globalization::{
    lstrlenA, MultiByteToWideChar, WideCharToMultiByte, CP_UTF8, MULTI_BYTE_TO_WIDE_CHAR_FLAGS,
};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{GetProcessHandleFromHwnd, GetProcessId};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, IsWindowVisible, SendMessageW, WM_USER,
};

const WM_MCOMMAND: u32 = WM_USER + 200;
const WM_MEVALUATE: u32 = WM_USER + 201;

fn main() {
    // Make sure we've received arguments
    println!("Arguments: {:?}", std::env::args().collect::<Vec<String>>());
    if std::env::args().len() == 1 {
        println!("Usage: debug-msl.exe [-p] <mSL code to run>");
        println!("-p: Print the process name and PID target clients");
        return;
    }

    if unsafe { EnumWindows(Some(enum_windows_proc), LPARAM(0)).is_err() } {
        println!("Unable to enumerate windows");
    }
}

extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    unsafe {
        // Skip invisible windows (eg. Minimised to tray)
        if IsWindowVisible(hwnd).as_bool() {
            // We want process handle, so we can determine the process name
            let process_handle = GetProcessHandleFromHwnd(hwnd);
            if !process_handle.is_invalid() {
                // Get the process name (allowing max 256 characters)
                let mut process_name = [0; 256];
                let name_length = GetModuleBaseNameW(process_handle, None, &mut process_name);
                if name_length > 0 {
                    let process_name_slice = &process_name[..name_length as usize];
                    let process_name = PCWSTR(process_name_slice.as_ptr()).to_string().unwrap();
                    if process_name.to_lowercase() == "mirc.exe"
                        || process_name.to_lowercase() == "adiirc.exe"
                    {
                        let m_eval = send_message_to_irc_client(
                            hwnd,
                            WM_MEVALUATE,
                            &w!("$+(v,$version $chr(40),$bits bit,$chr(41))"),
                        );
                        match m_eval {
                            Ok(m_eval) => {
                                // We've received a reply from the IRC client
                                
                                let args: Vec<String> = std::env::args().collect();
                                let args_string = args[1..].join(" ");

                                if args[1] == "-p" {
                                    let pid = GetProcessId(process_handle);
                                    println!("{} (PID: {})", process_name, pid);
                                }
                                else {
                                    let m_eval = m_eval.to_string().unwrap();
                                    println!("Sent command to {} {}", process_name, m_eval);
                                    
                                    let message = format!("//{}\0", args_string);
                                    let message_wide: Vec<u16> = message.encode_utf16().collect();
                                    let message_pcwstr = PCWSTR(message_wide.as_ptr());
                                    let _ =
                                        send_message_to_irc_client(hwnd, WM_MCOMMAND, &message_pcwstr);
                                }
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
) -> Result<PCWSTR, String> {
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
        let reply_mb = PCSTR(map_view.Value as *const u8);

        // Convert the reply from UTF-8 to Unicode
        let reply_mb_slice = std::slice::from_raw_parts(reply_mb.0, lstrlenA(reply_mb) as usize);
        let reply_wide_length = MultiByteToWideChar(
            CP_UTF8,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            reply_mb_slice,
            None,
        );
        let mut reply_wide_vec = vec![0u16; (reply_wide_length as usize) + 1];
        MultiByteToWideChar(
            CP_UTF8,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            reply_mb_slice,
            Some(&mut reply_wide_vec),
        );
        let reply = PCWSTR(reply_wide_vec.as_ptr());

        //////////////////////////////
        // END OF ADIIRC WORKAROUND //
        // END OF ADIIRC WORKAROUND //
        // END OF ADIIRC WORKAROUND //
        //////////////////////////////

        // Clean up
        let _ = UnmapViewOfFile(map_view);
        Ok(reply)
    }
}
