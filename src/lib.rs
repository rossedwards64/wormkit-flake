#![allow(non_snake_case)]

use std::ffi::c_void;

use windows::{
    core::{self, PCWSTR},
    Win32::{
        Foundation::{self, HINSTANCE},
        Storage::FileSystem::{self, VS_FIXEDFILEINFO},
        System::{
            LibraryLoader,
            SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        },
        UI,
    },
};

macro_rules! hiword {
    ($l:expr) => {
        $l & 0xffff
    };
}

macro_rules! loword {
    ($l:expr) => {
        ($l >> 16) & 0xffff
    };
}

#[no_mangle]
#[allow(unused_variables, improper_ctypes_definitions)]
pub extern "system" fn DllMain(module_handle: HINSTANCE, call_reason: u32, _: *mut c_void) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                let file_info = check_version(module_handle);
                if let Some(file_info) = file_info {
                    let version_msg = format!(
                        "The version of your game is {}.{}.{}.",
                        hiword!(file_info.dwFileDateMS),
                        loword!(file_info.dwFileDateMS),
                        hiword!(file_info.dwFileDateLS)
                    );
                    UI::WindowsAndMessaging::MessageBoxW(
                        Foundation::HWND::default(),
                        PCWSTR::from_raw(version_msg.encode_utf16().collect::<Vec<u16>>().as_ptr()),
                        core::w!("wkVersionCheck"),
                        UI::WindowsAndMessaging::MB_OK,
                    );
                } else {
                    UI::WindowsAndMessaging::MessageBoxW(
                        Foundation::HWND::default(),
                        core::w!("Couldn't find version information!"),
                        core::w!("wkVersionCheck"),
                        UI::WindowsAndMessaging::MB_ICONERROR,
                    );
                }
            }
            true
        }
        DLL_PROCESS_DETACH => detach(),
        _ => false,
    }
}

fn check_version(module_handle: HINSTANCE) -> Option<VS_FIXEDFILEINFO> {
    let mut result = None;
    unsafe {
        let mut module_info = make_buffer();
        let module_info_buf = module_info.as_mut_slice();
        LibraryLoader::GetModuleFileNameW(module_handle, module_info_buf);

        let module_name = PCWSTR::from_raw(module_info_buf.as_ptr());
        let file_handle: *mut u32 = &mut u32::default();
        let size = FileSystem::GetFileVersionInfoSizeW(module_name, Some(file_handle));

        if size != 0 {
            let ver_info_buf = make_buffer().as_mut_ptr();

            let _ = FileSystem::GetFileVersionInfoW(module_name, *file_handle, size, ver_info_buf);

            let mut file_info =
                std::ptr::from_mut(&mut VS_FIXEDFILEINFO::default()).cast::<c_void>();

            if FileSystem::VerQueryValueW(
                ver_info_buf,
                core::w!(r"\"),
                &mut file_info,
                &mut u32::default(),
            )
            .as_bool()
            {
                result = Some(*file_info.cast::<FileSystem::VS_FIXEDFILEINFO>());
            }
        } else {
            panic!("Error getting file info size!")
        }
    }
    result
}

const fn detach() -> bool {
    true
}

fn make_buffer<T>() -> Vec<T> {
    Vec::<T>::with_capacity(Foundation::MAX_PATH as usize)
}
