use core::ffi::c_void;
use std::os::windows::io::AsRawHandle;
use std::process::Child;
mod windows {
    pub(crate) use windows::Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
            SystemInformation::{
                IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM, IMAGE_FILE_MACHINE_ARM64,
                IMAGE_FILE_MACHINE_ARMNT, IMAGE_FILE_MACHINE_I386, IMAGE_FILE_MACHINE_IA64,
                IMAGE_FILE_MACHINE_THUMB, IMAGE_FILE_MACHINE_UNKNOWN,
            },
            Threading::{
                IsWow64Process2, OpenProcess, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION,
                PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
            },
        },
    };
}

use super::{
    Architecture, CopyAddress, Machine, ProcessHandleExt, PutAddress, TryIntoProcessHandle,
};

/// On Windows a `Pid` is a unsigned 32-bit integer.
pub type Pid = u32;
/// On Windows a `ProcessHandle` is a `HANDLE`.
pub type ProcessHandle = (windows::HANDLE, Architecture);

impl ProcessHandleExt for ProcessHandle {
    fn check_handle(&self) -> bool {
        !self.0.is_invalid()
    }
    fn null_type() -> ProcessHandle {
        (windows::HANDLE::default(), Architecture::from_native())
    }
    fn set_arch(self, arch: Architecture) -> Self {
        (self.0, arch)
    }
    fn close(&self) {
        if self.0.is_invalid() {
            return;
        }
        unsafe {
            let _ = windows::CloseHandle(self.0);
        }
    }

    fn get_machine(&self) -> Machine {
        if self.0.is_invalid() {
            return Machine::Unknown;
        }

        let mut process_machine = windows::IMAGE_FILE_MACHINE_UNKNOWN;
        let mut native_machine = windows::IMAGE_FILE_MACHINE_UNKNOWN;

        unsafe {
            if windows::IsWow64Process2(
                self.0,
                &raw mut process_machine,
                Some(&raw mut native_machine),
            )
            .as_bool()
            {
                let machine = if process_machine == windows::IMAGE_FILE_MACHINE_UNKNOWN {
                    native_machine
                } else {
                    process_machine
                };

                match machine {
                    windows::IMAGE_FILE_MACHINE_I386 => Machine::X86,
                    windows::IMAGE_FILE_MACHINE_AMD64 => Machine::X64,
                    windows::IMAGE_FILE_MACHINE_ARM
                    | windows::IMAGE_FILE_MACHINE_ARMNT
                    | windows::IMAGE_FILE_MACHINE_THUMB => Machine::Arm32,
                    windows::IMAGE_FILE_MACHINE_ARM64 => Machine::Arm64,
                    _ => Machine::Unknown,
                }
            } else {
                Machine::Unknown
            }
        }
    }
}

/// 対象のHANDLEから `IsWow64Process2` を使って正確なArchitectureを取得するヘルパー関数
fn get_arch_from_handle(handle: windows::HANDLE) -> Architecture {
    let mut process_machine = windows::IMAGE_FILE_MACHINE_UNKNOWN;
    let mut native_machine = windows::IMAGE_FILE_MACHINE_UNKNOWN;

    unsafe {
        if windows::IsWow64Process2(
            handle,
            &raw mut process_machine,
            Some(&raw mut native_machine),
        )
        .as_bool()
        {
            let machine = if process_machine == windows::IMAGE_FILE_MACHINE_UNKNOWN {
                native_machine
            } else {
                process_machine
            };

            match machine {
                // 64-bit Architecture
                windows::IMAGE_FILE_MACHINE_AMD64
                | windows::IMAGE_FILE_MACHINE_ARM64
                | windows::IMAGE_FILE_MACHINE_IA64 => Architecture::Arch64Bit,

                // 32-bit Architecture
                windows::IMAGE_FILE_MACHINE_I386
                | windows::IMAGE_FILE_MACHINE_ARM
                | windows::IMAGE_FILE_MACHINE_ARMNT
                | windows::IMAGE_FILE_MACHINE_THUMB => Architecture::Arch32Bit,

                _ => Architecture::from_native(),
            }
        } else {
            Architecture::from_native()
        }
    }
}

/// A `Pid` can be turned into a `ProcessHandle` with `OpenProcess`.
impl TryIntoProcessHandle for Pid {
    fn try_into_process_handle(&self) -> std::io::Result<ProcessHandle> {
        let handle = unsafe {
            windows::OpenProcess(
                windows::PROCESS_CREATE_THREAD
                    | windows::PROCESS_QUERY_INFORMATION
                    | windows::PROCESS_VM_READ
                    | windows::PROCESS_VM_WRITE
                    | windows::PROCESS_VM_OPERATION,
                false,
                *self,
            )
        }?;
        let arch = get_arch_from_handle(handle);

        Ok((handle, arch))
    }
}

/// A `std::process::Child` has a `HANDLE` from calling `CreateProcess`.
impl TryIntoProcessHandle for Child {
    fn try_into_process_handle(&self) -> std::io::Result<ProcessHandle> {
        let handle = windows::HANDLE(self.as_raw_handle() as isize);
        let arch = get_arch_from_handle(handle);
        Ok((handle, arch))
    }
}

/// Use `ReadProcessMemory` to read memory from another process on Windows.
impl CopyAddress for ProcessHandle {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn get_pointer_width(&self) -> Architecture {
        self.1
    }

    #[allow(clippy::ptr_as_ptr)]
    fn copy_address(&self, addr: usize, buf: &mut [u8]) -> std::io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }

        if unsafe {
            windows::ReadProcessMemory(
                self.0,
                addr as *const c_void,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                None,
            )
        } == false
        {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

/// Use `WriteProcessMemory` to write memory from another process on Windows.
impl PutAddress for ProcessHandle {
    #[allow(clippy::ptr_as_ptr)]
    fn put_address(&self, addr: usize, buf: &[u8]) -> std::io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        if unsafe {
            windows::WriteProcessMemory(
                self.0,
                addr as *const c_void,
                buf.as_ptr().cast(),
                buf.len(),
                None,
            )
        } == false
        {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
