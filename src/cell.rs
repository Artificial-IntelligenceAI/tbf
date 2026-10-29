//! A tape cell: `u8`, `u16` or `u32`, always wrapping.

pub trait Cell: Copy + Eq + Default + Send + 'static {
    /// Truncates `v` to this width.
    fn from_u32(v: u32) -> Self;
    fn to_u32(self) -> u32;
    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_mul(self, other: Self) -> Self;
    /// Every bit set: −1 at this width.
    const MAX: Self;
}

macro_rules! cell {
    ($($t:ty),*) => {$(
        impl Cell for $t {
            #[inline(always)]
            fn from_u32(v: u32) -> Self { v as $t }
            #[inline(always)]
            fn to_u32(self) -> u32 { self as u32 }
            #[inline(always)]
            fn wrapping_add(self, other: Self) -> Self { <$t>::wrapping_add(self, other) }
            #[inline(always)]
            fn wrapping_mul(self, other: Self) -> Self { <$t>::wrapping_mul(self, other) }
            const MAX: Self = <$t>::MAX;
        }
    )*};
}

cell!(u8, u16, u32);
