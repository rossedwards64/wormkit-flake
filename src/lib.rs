#![allow(non_snake_case)]

use std::ffi::c_void;

use windows::{
    core::{self, PCWSTR},
    Win32::{
        Foundation::{self, HINSTANCE},
        Storage::FileSystem::{self},
        System::LibraryLoader,
        System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        UI,
    },
};

const VV1: u32 = 3;
const VV2: u32 = 8;
const VV3: u32 = 1;

const DW_SIGNATURE: u32 = 0xFEEF04BD;

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
pub extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut c_void) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                UI::WindowsAndMessaging::MessageBoxW(
                    Foundation::HWND::default(),
                    core::w!("The version of your game is 3.8.1."),
                    core::w!("wkVersionCheck"),
                    UI::WindowsAndMessaging::MB_OK | UI::WindowsAndMessaging::MB_ICONERROR,
                );
            }
            true
        }
        DLL_PROCESS_DETACH => detach(),
        _ => false,
    }
}

fn check_version(dll_module: HINSTANCE) -> bool {
    unsafe {
        let mut mod_info = make_buffer();
        let mod_info_buf = mod_info.as_mut_slice();
        LibraryLoader::GetModuleFileNameW(dll_module, mod_info_buf);

        let mod_name = PCWSTR::from_raw(mod_info_buf.as_ptr());
        let handle: Option<*mut u32> = Some(&mut u32::default());
        let size = FileSystem::GetFileVersionInfoSizeW(mod_name, handle);

        if size != 0 {
            let ver_info_buf = make_buffer().as_mut_ptr();

            let _ = FileSystem::GetFileVersionInfoW(mod_name, u32::default(), size, ver_info_buf);

            let mut file_info =
                &mut FileSystem::VS_FIXEDFILEINFO::default() as *mut _ as *mut c_void;

            if FileSystem::VerQueryValueW(
                ver_info_buf,
                core::w!(r"\"),
                &mut file_info,
                &mut u32::default(),
            )
            .as_bool()
            {
                let file_info = *(file_info as *mut FileSystem::VS_FIXEDFILEINFO);

                file_info.dwSignature == DW_SIGNATURE
                    && hiword!(file_info.dwFileVersionMS) == VV1
                    && loword!(file_info.dwFileVersionMS) == VV2
                    && hiword!(file_info.dwFileVersionLS) == VV3
            } else {
                panic!("Couldn't get version info from file!")
            }
        } else {
            panic!("Error getting file info size!")
        }
    }
}

fn detach() -> bool {
    true
}

fn make_buffer<T>() -> Vec<T> {
    Vec::<T>::with_capacity(Foundation::MAX_PATH as usize)
}
