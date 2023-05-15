//! Provides an arrayvec-like type which can be modified at const-time.

use core::mem::ManuallyDrop;

/// Essentially a [std::mem::MaybeUninit], but with externals exposed for const contexts
union MaybeUninit<T> {
    uninit: (),
    value: ManuallyDrop<T>,
}

impl<T> MaybeUninit<T> {
    const fn uninit() -> Self {
        Self { uninit: () }
    }

    /// #Safety
    /// Undefined behavior where `T` is uninhabited
    const unsafe fn assume_init(self) -> T {
        ManuallyDrop::into_inner(self.value)
    }
}

pub struct ConstVec<T, const CAP: usize> {
    xs: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<T, const CAP: usize> ConstVec<T, CAP> {
    pub const fn new() -> Self {
        Self {
            xs: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub const fn push(self, item: T) -> Self {
        todo!()
    }

    pub const unsafe fn push_unchecked(mut self, item: T) -> Self {
        debug_assert!(self.len < CAP);
        self.xs[self.len] = MaybeUninit {
            value: ManuallyDrop::new(item),
        };
        let len = self.len;
        self = self.set_len(len + 1);
        self
    }

    pub const unsafe fn set_len(mut self, length: usize) -> Self {
        debug_assert!(length <= CAP);
        self.len = length;
        self
    }
}
