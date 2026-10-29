//! What a program's directives set (`tbf cell …`, `tbf eof …`).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Width {
    #[default]
    U8,
    U16,
    U32,
}

impl Width {
    /// The cell's bits, all set.
    pub fn mask(self) -> u32 {
        match self {
            Width::U8 => 0xff,
            Width::U16 => 0xffff,
            Width::U32 => 0xffff_ffff,
        }
    }

    /// Wraps a signed amount into this width, as an unsigned value.
    pub fn wrap(self, v: i64) -> u32 {
        (v as u32) & self.mask()
    }
}

/// What `,` does at the end of input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Eof {
    #[default]
    Zero,
    Keep,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Settings {
    pub cell: Width,
    pub eof: Eof,
}
