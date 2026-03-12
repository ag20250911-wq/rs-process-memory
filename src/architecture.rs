use std::convert::TryInto;

/// Enum representing the architecture of a process
#[derive(Clone, Debug, Copy)]
#[repr(u8)]
pub enum Architecture {
    /// 8-bit architecture
    #[cfg(any(
        target_pointer_width = "8",
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    Arch8Bit = 1,
    /// 16-bit architecture
    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    Arch16Bit = 2,
    /// 32-bit architecture
    #[cfg(any(
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    Arch32Bit = 4,
    /// 64-bit architecture
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
    Arch64Bit = 8,
    /// 128-bit architecture
    #[cfg(target_pointer_width = "128")]
    Arch128Bit = 16,
}

impl Architecture {
    /// Create an Architecture matching that of the host process.
    #[must_use]
    pub fn from_native() -> Architecture {
        #[cfg(target_pointer_width = "8")]
        return Architecture::Arch8Bit;
        #[cfg(target_pointer_width = "16")]
        return Architecture::Arch16Bit;
        #[cfg(target_pointer_width = "32")]
        return Architecture::Arch32Bit;
        #[cfg(target_pointer_width = "64")]
        return Architecture::Arch64Bit;
        #[cfg(target_pointer_width = "128")]
        return Architecture::Arch128Bit;
    }

    /// Convert bytes read from memory into a pointer in the
    /// current architecture.
    ///
    /// # Panics
    /// If there are not enough bytes in the slice to make an integer of the sized indicated by
    /// `self`.
    #[must_use]
    pub fn pointer_from_ne_bytes(self, bytes: &[u8]) -> usize {
        match self {
            #[allow(clippy::cast_possible_truncation)]
            #[cfg(any(
                target_pointer_width = "8",
                target_pointer_width = "16",
                target_pointer_width = "32",
                target_pointer_width = "64",
                target_pointer_width = "128"
            ))]
            Architecture::Arch8Bit => u8::from_ne_bytes(bytes.try_into().unwrap()) as usize,
            #[allow(clippy::cast_possible_truncation)]
            #[cfg(any(
                target_pointer_width = "16",
                target_pointer_width = "32",
                target_pointer_width = "64",
                target_pointer_width = "128"
            ))]
            Architecture::Arch16Bit => u16::from_ne_bytes(bytes.try_into().unwrap()) as usize,
            #[allow(clippy::cast_possible_truncation)]
            #[cfg(any(
                target_pointer_width = "32",
                target_pointer_width = "64",
                target_pointer_width = "128"
            ))]
            Architecture::Arch32Bit => u32::from_ne_bytes(bytes.try_into().unwrap()) as usize,
            #[allow(clippy::cast_possible_truncation)]
            #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
            Architecture::Arch64Bit => u64::from_ne_bytes(bytes.try_into().unwrap()) as usize,
            #[allow(clippy::cast_possible_truncation)]
            #[cfg(target_pointer_width = "128")]
            Architecture::Arch128Bit => u128::from_ne_bytes(bytes.try_into().unwrap()) as usize,
        }
    }
}

/// Enum representing the CPU instruction set architecture of a process
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Machine {
    /// 32-bit architecture
    X86,
    /// 64-bit architecture
    X64,
    /// 32-bit ARM architecture
    Arm32,
    /// 64-bit ARM architecture
    Arm64,
    /// Unknown architecture
    Unknown,
}

impl Machine {
    /// Create a Machine matching that of the host process.
    #[must_use]
    pub fn from_native() -> Machine {
        #[cfg(target_arch = "x86")]
        return Machine::X86;
        #[cfg(target_arch = "x86_64")]
        return Machine::X64;
        #[cfg(target_arch = "arm")]
        return Machine::Arm32;
        #[cfg(target_arch = "aarch64")]
        return Machine::Arm64;
        #[cfg(not(any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "arm",
            target_arch = "aarch64"
        )))]
        return Machine::Unknown;
    }
}
