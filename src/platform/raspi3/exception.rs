use core::arch::global_asm;

use crate::{
    aarch64::registers::{ExceptionLinkRegister, ExceptionSyndromeRegister, FaultAddressRegister},
    allocator::page_allocator::StackPointer,
    platform::platform_devices::{get_platform, PLATFORM},
    println,
};

global_asm!(include_str!("exception.s"));

#[derive(Debug)]
#[repr(u64)]
pub enum ExceptionSource {
    CurrentELUserSP = 0,
    CurrentELCurrentSP = 1,
    LowerEL64 = 2,
    LowerEL32 = 3,
}

#[derive(Debug, PartialEq, Eq)]
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

impl TryFrom<u64> for DataFaultStatus {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
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
}

#[no_mangle]
pub extern "C" fn handle_exception(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    exception_source: ExceptionSource,
    exception_type: ExceptionType,
    frame: &mut InterruptFrame,
    sp: StackPointer,
) {
    let esr = ExceptionSyndromeRegister::read_to_buffer().value();
    let far = FaultAddressRegister::read_to_buffer().value();
    let elr = ExceptionLinkRegister::read_to_buffer().value();

    let platform = get_platform();
    platform.push_frame(frame, sp);

    if exception_type == ExceptionType::Interrupt {
        platform.handle_interrupt();
    } else if exception_type == ExceptionType::Synchronous {
        let esr = ExceptionSyndromeRegister::read_to_buffer();

        let exception_class: ExceptionClass =
            (esr.get_exception_class() as u64).try_into().unwrap();

        if exception_class == ExceptionClass::SystemCall {
            let syscall_number = esr.get_instruction_number();

            platform.handle_syscall(syscall_number, [arg1, arg2, arg3]);
        } else if exception_class == ExceptionClass::DataAbort {
            println!("Kernel Page fault!");
        }
    }

    println!(
        "Received Exception Type {:?} from {:?}",
        exception_type, exception_source
    );

    if let Some(ref thread) = PLATFORM.get_current_thread() {
        println!("From thread: {}", thread.name);
        println!("With sp: {:#p}", *thread.stack_pointer.lock());
    }

    println!("elr: {:#x}", elr);
    println!("esr: {:#x}", esr);
    println!("far: {:#x}", far);

    println!("{:?}", frame);

    loop {}
}
