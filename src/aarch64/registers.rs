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

            pub struct RegisterBuffer {
                value: usize
            }

            impl RegisterBuffer {
                $(paste::item! {
                    fn [< $field _mask >]() -> usize {
                        ((1 << ($end - $start + 1)) - 1) << $start
                    }

                    pub fn [< get_ $field >](self) -> usize {
                        (self.value & Self::[< $field _mask>]()) >> $start
                    }

                    pub fn [< set_ $field>](mut self, value: usize) -> Self {
                        let mask = Self::[< $field _mask >]();
                        self.value &= !mask;
                        self.value |= mask & (value << $start);
                        return self
                    }
                })*

                pub fn write_to_register(self) {
                    write!($register, self.value)
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

registers!{
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
        translation_state: 0-0
    } with {
        pub enum TranslationState {
            Enabled = 0b1,
            Disabled = 0b0
        }
    },
    TranslationTableBaseRegister("ttbr0_el1") {
        table_pointer: 0-47
    }
}