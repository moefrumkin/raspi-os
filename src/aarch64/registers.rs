#![allow(non_snake_case)]
#[macro_export]
macro_rules! read {
    ($sysreg: literal) => {
        unsafe {
            let value: usize;
            asm!(concat!("mrs {}, " , $sysreg), out(reg) value);
            value
        }
    }
}

#[macro_export]
macro_rules! write {
    ($sysreg: literal, $value: expr) => {
        unsafe {
            asm!(concat!("msr ", $sysreg, ", {}"), in(reg) $value);
        }
    };
}

macro_rules! registers {
    (
        $($name: ident ($register: literal)
        {$($field: ident: $start: literal-$end:literal),*}
        $(with {$($attributes: item);+})?),+
    ) => {
        $(pub mod $name {
            use core::arch::asm;
            use crate::bitfield;

            bitfield! {
                RegisterBuffer(usize) {
                    $($field: $start - $end),*
                } with {
                    pub fn write_to_register(self) {
                        write!($register, self.value)
                    }

                    // TODO: get rid of this
                    pub fn value(self) -> usize {
                        self.value
                    }
                }
            }

            $(
                $($attributes)+
            )?

            pub fn read_to_buffer() -> RegisterBuffer {
                RegisterBuffer {
                    value: read!($register)
                }
            }
        })+
    }
}

registers! {
    TranslationControlRegister("tcr_el1") {
        granule_size: 30-31,
        table_offset: 0-5
    } with {
        pub enum GranuleSize {
            Kb4 = 0b10,
            Kb16 = 0b01,
            Kb64 = 0b11,
        }
    },
    SystemControlRegister("sctlr_el1") {
        cache_enable: 2-2,
        translation_state: 0-0
    } with {
        pub enum TranslationState {
            Enabled = 0b1,
            Disabled = 0b0
        }
    },
    KernelTranslationTableBaseRegister("ttbr1_el1") {
        table_pointer: 0-47
    },
    UserTranslationTableBaseRegister("ttbr0_el1") {
        table_pointer: 0-47
    },
    ExceptionSyndromeRegister("esr_el1") {
        exception_class: 26-31,
        instruction_number: 0-15
    },
    ExceptionLinkRegister("elr_el1") {},
    FaultAddressRegister("far_el1") {}
}
