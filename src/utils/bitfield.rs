#[macro_export]
macro_rules! bitfield {
    (
        $name: ident ($type: ty)
        {$($field: ident: $start: literal - $end: literal),*}
        $(with {$($attributes: item)+})?
    ) => {
        #[derive(Copy, Clone)]
        pub struct $name {
            value: $type
        }

        impl $name {
            $(paste::item! {
                fn [< $field _mask >]() -> $type {
                    ((1 << ($end - $start + 1)) - 1) << $start
                }

                pub fn [< get_ $field >](self) -> $type {
                    (self.value & Self::[< $field _mask>]()) >> $start
                }

                pub fn [< set_ $field>](mut self, value: $type) -> Self {
                    let mask = Self::[< $field _mask >]();
                    self.value &= !mask;
                    self.value |= mask & (value << $start);
                    return self
                }
            })*

            $(
                $($attributes)+
            )?
        }
    };
}
