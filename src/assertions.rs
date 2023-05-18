// https://github.com/nvzqz/static-assertions-rs/issues/40#issuecomment-1458897730
pub struct Leq<const LESSER: usize, const GREATER: usize>;

impl<const LESSER: usize, const GREATER: usize> Leq<LESSER, GREATER> {
    const CHECK: () = assert!(LESSER <= GREATER);

    pub const fn assert() -> Self {
        #[allow(clippy::let_unit_value)]
        let _ = Self::CHECK;
        Self {}
    }
}
