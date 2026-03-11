use libc::{c_void, iovec, pid_t, process_vm_readv, process_vm_writev};
use std::process::Child;

use std::fs::File;
use std::io::Read;

use super::{Architecture, CopyAddress, ProcessHandleExt, PutAddress, TryIntoProcessHandle};

/// On Linux a `Pid` is just a `libc::pid_t`.
pub type Pid = pid_t;
/// On Linux a `ProcessHandle` is just a `libc::pid_t`.
pub type ProcessHandle = (Pid, Architecture);

impl ProcessHandleExt for ProcessHandle {
    fn check_handle(&self) -> bool {
        self.0 != 0
    }
    fn null_type() -> Self {
        (0, Architecture::from_native())
    }
    fn set_arch(self, arch: Architecture) -> Self {
        (self.0, arch)
    }
    fn close(&self) {}
}

/// `対象のHANDLEからArchitectureを取得するヘルパー関数`
fn get_arch_from_handle(pid: Pid) -> Architecture {
    let path = format!("/proc/{pid}/exe");

    // 対象プロセスの実行ファイルを読み取り専用で開く
    if let Ok(mut file) = File::open(&path) {
        let mut buffer = [0u8; 5];
        // 先頭5バイトだけ読み取る
        if file.read_exact(&mut buffer).is_ok() {
            // ELFヘッダのマジックナンバー (0x7F, 'E', 'L', 'F') を確認
            if buffer[0..4] == [0x7F, 0x45, 0x4c, 0x46] {
                return match buffer[4] {
                    1 => Architecture::Arch32Bit,     // ELFCLASS32
                    2 => Architecture::Arch64Bit,     // ELFCLASS64
                    _ => Architecture::from_native(), // 未知の場合はフォールバック
                };
            }
        }
    }

    // 権限不足などで読めなかった場合は自身のアーキテクチャにフォールバック
    Architecture::from_native()
}

/// A `Child` always has a pid, which is all we need on Linux.
impl TryIntoProcessHandle for Child {
    fn try_into_process_handle(&self) -> std::io::Result<ProcessHandle> {
        #[allow(clippy::cast_possible_wrap)]
        let pid = self.id() as Pid;
        let arch = get_arch_from_handle(pid);
        Ok((pid, arch))
    }
}

impl TryIntoProcessHandle for Pid {
    fn try_into_process_handle(&self) -> std::io::Result<ProcessHandle> {
        let arch = get_arch_from_handle(*self);
        Ok((*self, arch))
    }
}

impl CopyAddress for ProcessHandle {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn get_pointer_width(&self) -> Architecture {
        self.1
    }

    fn copy_address(&self, addr: usize, buf: &mut [u8]) -> std::io::Result<()> {
        let local_iov = iovec {
            iov_base: buf.as_mut_ptr().cast::<c_void>(),
            iov_len: buf.len(),
        };
        let remote_iov = iovec {
            iov_base: addr as *mut c_void,
            iov_len: buf.len(),
        };
        let result = unsafe {
            process_vm_readv(self.0, &raw const local_iov, 1, &raw const remote_iov, 1, 0)
        };
        if result == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl PutAddress for ProcessHandle {
    fn put_address(&self, addr: usize, buf: &[u8]) -> std::io::Result<()> {
        let local_iov = iovec {
            iov_base: buf.as_ptr() as *mut c_void,
            iov_len: buf.len(),
        };
        let remote_iov = iovec {
            iov_base: addr as *mut c_void,
            iov_len: buf.len(),
        };
        let result = unsafe {
            process_vm_writev(self.0, &raw const local_iov, 1, &raw const remote_iov, 1, 0)
        };
        if result == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
