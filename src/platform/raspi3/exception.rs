use core::arch::global_asm;

use crate::{
    aarch64::{
        registers::{ExceptionLinkRegister, ExceptionSyndromeRegister, FaultAddressRegister},
        syscall::SyscallArgs,
    },
    allocator::page_allocator::StackPointer,
    platform::platform_devices::{get_platform, PLATFORM},
    println,
};

global_asm!(include_str!("exception.s"));

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum ExceptionSource {
    CurrentELUserSP = 0,
    CurrentELCurrentSP = 1,
    LowerEL64 = 2,
    LowerEL32 = 3,
}

impl ExceptionSource {
    pub fn is_kernel(&self) -> bool {
        match self {
            Self::CurrentELUserSP | Self::CurrentELCurrentSP => true,
            Self::LowerEL64 | Self::LowerEL32 => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u64)]
pub enum ExceptionType {
    Synchronous = 0,
    Interrupt = 1,
    FastInterrupt = 2,
    SystemError = 4,
}

/// Partial enumeration of aarch64 exception classes. This is reported in ESR_EL1
/// For more, see: https://developer.arm.com/documentation/ddi0601/2025-12/AArch64-Registers/ESR-EL1--Exception-Syndrome-Register--EL1-?lang=en
#[derive(Debug, PartialEq, Eq)]
pub enum ExceptionClass {
    Unknown = 0b0,
    TrappedWF = 0b1,
    // Note: Skipping AA32 expcetions
    TrappedFPInstruction = 0b111,
    TrappedPointerAuthenticatedInstruction = 0b1001,
    TrappedInstructionExecution = 0b1010,
    BranchTargetException = 0b1101,
    IllegalExecutionState = 0b1110,
    SystemCall = 0b10101,
    TrappedSVE = 0b11001,
    InstructionAbortFromLowerLevel = 0b10_0000,
    InstructionAbort = 0b10_0001,
    PCAlignmentFault = 0b10_0010,
    DataAbortFromLowerLevel = 0b10_0100,
    DataAbort = 0b10_0101,
    SPAlignmentFault = 0b10_0110,
    MemoryOperationException = 0b10_0111,
}

impl TryFrom<u64> for ExceptionClass {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0b0 => Ok(Self::Unknown),
            0b1 => Ok(Self::TrappedWF),
            0b111 => Ok(Self::TrappedFPInstruction),
            0b1001 => Ok(Self::TrappedPointerAuthenticatedInstruction),
            0b1010 => Ok(Self::TrappedInstructionExecution),
            0b1101 => Ok(Self::BranchTargetException),
            0b1110 => Ok(Self::IllegalExecutionState),
            0b10101 => Ok(Self::SystemCall),
            0b11001 => Ok(Self::TrappedSVE),
            0b10_0000 => Ok(Self::InstructionAbortFromLowerLevel),
            0b10_0001 => Ok(Self::InstructionAbort),
            0b10_0010 => Ok(Self::PCAlignmentFault),
            0b10_0100 => Ok(Self::DataAbortFromLowerLevel),
            0b10_0101 => Ok(Self::DataAbort),
            0b10_0110 => Ok(Self::SPAlignmentFault),
            0b10_0111 => Ok(Self::MemoryOperationException),
            _ => Err("Unknown Exception Class"),
        }
    }
}

/// https://df.lth.se/~getz/ARM/SysReg/AArch64-esr_el1.html#fieldset_0-24_0_16-5_0
#[derive(PartialEq, Eq)]
pub enum DataFaultStatus {
    AddressSizeFaultLevel0 = 0b00_0000,
    AddressSizeFaultLevel1 = 0b00_0001,
    AddressSizeFaultLevel2 = 0b00_0010,
    AddressSizeFaultLevel3 = 0b00_0011,

    TranslationFaultLevel0 = 0b00_0100,
    TranslationFaultLevel1 = 0b00_0101,
    TranslationFaultLevel2 = 0b00_0110,
    TranslationFaultLevel3 = 0b00_0111,

    AccessFlagFaultLevel1 = 0b00_1001,
    AccessFlagFaultLevel2 = 0b00_1010,
    AccessFlagFaultLevel3 = 0b00_1011,

    PermissionFaultLevel1 = 0b00_1101,
    PermissionFaultLevel2 = 0b00_1110,
    PermissionFaultLevel3 = 0b00_1111,
    // Skipping Synchronous External Aborts
}

impl DataFaultStatus {
    pub fn is_address_size_fault(&self) -> bool {
        match self {
            Self::AddressSizeFaultLevel0
            | Self::AddressSizeFaultLevel1
            | Self::AddressSizeFaultLevel2
            | Self::AccessFlagFaultLevel3 => true,
            _ => false,
        }
    }

    pub fn is_translation_fault(&self) -> bool {
        match self {
            Self::TranslationFaultLevel0
            | Self::TranslationFaultLevel1
            | Self::TranslationFaultLevel2
            | Self::TranslationFaultLevel3 => true,
            _ => false,
        }
    }

    pub fn is_access_flag_fault(&self) -> bool {
        match self {
            Self::AccessFlagFaultLevel1
            | Self::AccessFlagFaultLevel2
            | Self::AccessFlagFaultLevel3 => true,
            _ => false,
        }
    }

    pub fn is_permission_fault(&self) -> bool {
        match self {
            Self::PermissionFaultLevel1
            | Self::PermissionFaultLevel2
            | Self::PermissionFaultLevel3 => true,
            _ => false,
        }
    }
}

impl TryFrom<usize> for DataFaultStatus {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0b00_0000 => Ok(Self::AddressSizeFaultLevel0),
            0b00_0001 => Ok(Self::AddressSizeFaultLevel1),
            0b00_0010 => Ok(Self::AddressSizeFaultLevel2),
            0b00_0011 => Ok(Self::AddressSizeFaultLevel3),

            0b00_0100 => Ok(Self::TranslationFaultLevel0),
            0b00_0101 => Ok(Self::TranslationFaultLevel1),
            0b00_0110 => Ok(Self::TranslationFaultLevel2),
            0b00_0111 => Ok(Self::TranslationFaultLevel3),

            0b00_1001 => Ok(Self::AccessFlagFaultLevel1),
            0b00_1010 => Ok(Self::AccessFlagFaultLevel2),
            0b00_1011 => Ok(Self::AccessFlagFaultLevel3),

            0b00_1101 => Ok(Self::PermissionFaultLevel1),
            0b00_1110 => Ok(Self::PermissionFaultLevel2),
            0b00_1111 => Ok(Self::PermissionFaultLevel3),

            _ => Err("Unknown Data Fault Type"),
        }
    }
}

pub struct Exception {
    exception_syndrome_register: ExceptionSyndromeRegister::RegisterBuffer,
    fault_address: usize,
    source: ExceptionSource,
    exception_type: ExceptionType,
}

impl Exception {
    pub fn get_type(&self) -> ExceptionType {
        self.exception_type
    }

    pub fn get_exception_class(&self) -> ExceptionClass {
        (self.exception_syndrome_register.get_exception_class() as u64)
            .try_into()
            .unwrap()
    }

    /// Precondition: Exception must be caused by a syscall
    pub fn get_syscall_number(&self) -> usize {
        self.exception_syndrome_register.get_instruction_number()
    }

    /// Was the exception caused in kernel code?
    pub fn is_kernel(&self) -> bool {
        self.source.is_kernel()
    }

    pub fn get_address(&self) -> usize {
        self.fault_address
    }

    pub fn get_data_fault_class(&self) -> DataFaultStatus {
        self.exception_syndrome_register
            .get_data_fault_status_code()
            .try_into()
            .expect("Unknown data fault class")
    }

    // TODO: is it ok to keep this?
    pub fn get_esr(&self) -> usize {
        self.exception_syndrome_register.value()
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct InterruptFrame {
    pub gp_registers: [u64; 32],
    pub elr: u64,
    pub spsr: u64,
    pub fp_regs: [u128; 32],
    pub fpsr: u64,
}

impl InterruptFrame {
    pub fn with_kernel_entry(entry_point: u64) -> Self {
        Self {
            gp_registers: [0; 32],
            elr: entry_point,
            spsr: 0b101, // EL1 with SP_EL1h
            fp_regs: [0; 32],
            fpsr: 0,
        }
    }

    pub fn set_arg(&mut self, arg: u64) {
        self.gp_registers[0] = arg;
    }

    pub fn get_syscall_arguments(&self) -> SyscallArgs {
        [
            self.gp_registers[0] as usize,
            self.gp_registers[1] as usize,
            self.gp_registers[2] as usize,
        ]
    }

    pub fn get_exception_link_register(&self) -> u64 {
        self.elr as u64
    }
}

#[no_mangle]
pub extern "C" fn handle_exception(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    exception_source: ExceptionSource,
    exception_type: ExceptionType,
    //TODO: Is is ok for this to be static?
    interrupt_frame: &'static mut InterruptFrame,
    stack_pointer: StackPointer,
) -> ! {
    let exception = Exception {
        exception_syndrome_register: ExceptionSyndromeRegister::read_to_buffer(),
        fault_address: FaultAddressRegister::read_to_buffer().value(),
        source: exception_source,
        exception_type,
    };

    PLATFORM.handle_exception(exception, interrupt_frame, stack_pointer);
}
