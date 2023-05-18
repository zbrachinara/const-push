#![no_std]
//! Provides an arrayvec-like type which can be modified at const-time.
//!
//! Removing or swapping elements needs the crate feature `fake-move`, which depends on the lang
//! feature `const_ptr_read`. This is stable in the nightly rust version `1.71.0`.

use core::ptr::addr_of;
use core::{mem::ManuallyDrop, panic};

use tap::Tap;

#[cfg(feature = "fake-move")]
mod addressing;
mod assertions;
mod iter;

pub struct CapacityError<T, const CAP: usize> {
    pub vector: ConstVec<T, CAP>,
    pub item: T,
}

impl<T, const CAP: usize> core::fmt::Debug for CapacityError<T, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CapacityError")
            .field("capacity", &CAP)
            .finish()
    }
}

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

#[repr(C)]
pub struct ConstVec<T, const CAP: usize> {
    len: usize,
    xs_addr: (),
    xs: [MaybeUninit<T>; CAP],
}

impl<T, const CAP: usize> ConstVec<T, CAP> {
    pub const fn new() -> Self {
        Self {
            xs: unsafe { MaybeUninit::uninit().assume_init() },
            xs_addr: (),
            len: 0,
        }
    }

    #[cfg(feature = "fake-move")]
    pub const fn from_array<const N: usize>(xs: [T; N]) -> Self {
        assertions::Leq::<N, CAP>::assert();

        let addressor = addressing::AddressExtractor::new(xs);
        let address = addressing::extract_addr!(addressor<MaybeUninit<T>>);
        let mut buffer: [MaybeUninit<T>; CAP] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut ix = 0;
        while ix < N {
            buffer[ix] = unsafe { address.add(ix).read() };
            ix += 1;
        }

        // all elements have been copied to our own buffer
        core::mem::forget(addressor);

        Self {
            len: N,
            xs_addr: (),
            xs: buffer,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(addr_of!(self.xs_addr) as *const T, self.len) }
    }

    pub const fn get(&self, ix: usize) -> Option<&T> {
        if ix < self.len {
            Some(unsafe { core::mem::transmute(&self.xs[ix].value) })
        } else {
            None
        }
    }

    #[cfg(feature = "fake-move")]
    pub const fn try_swap_remove(mut self, ix: usize) -> (Self, Option<T>) {
        if self.len > 0 {
            if ix == self.len - 1 {
                self.try_pop()
            } else {
                unsafe {
                    let removing = addressing::copy_item!(self<T>[ix]);
                    let swapping = addressing::copy_item!(self<ManuallyDrop<T>>[self.len - 1]);
                    self.xs[ix] = MaybeUninit { value: swapping };
                    let len = self.len - 1;
                    self = self.set_len(len);
                    (self, Some(removing))
                }
            }
        } else {
            (self, None)
        }
    }

    #[cfg(feature = "fake-move")]
    pub const fn pop(mut self) -> (Self, T) {
        if self.len > 0 {
            let new_len = self.len - 1;
            let item = unsafe {
                let item = addressing::copy_item!(self<T>[new_len]);
                self = self.set_len(new_len);
                item
            };
            (self, item)
        } else {
            panic!()
        }
    }

    #[cfg(feature = "fake-move")]
    pub const fn try_pop(mut self) -> (Self, Option<T>) {
        if self.len > 0 {
            let new_len = self.len - 1;
            let item = unsafe {
                let item = addressing::copy_item!(self<T>[new_len]);
                self = self.set_len(new_len);
                item
            };
            (self, Some(item))
        } else {
            (self, None)
        }
    }

    #[cfg(feature = "fake-move")]
    /// # Safety
    ///
    /// At the time of writing, there are a lot of limitations around const. In this case, the
    /// relevant issue is that we are not allowed to get references to objects which may contain
    /// [`UnsafeCell`]s -- both `&self` and `&self.xs` are impossible. There is a lot of weird code
    /// in here to get around this, but the bottom line is that I don't know if this is actually a
    /// sound thing to do, and actually, with pointer provenance I think it is completey unsound.
    /// However, if you still feel like using this approach, I guess just avoid putting in types
    /// which have [`UnsafeCell`]s in them (which shouldn't be hard anyway, given that const
    /// disallows heap allocations).
    pub const unsafe fn pop_unchecked(mut self) -> (Self, T) {
        debug_assert!(self.len > 0);
        let new_len = self.len - 1;
        let item = addressing::copy_item!(self<T>[new_len]);
        self = self.set_len(new_len);
        (self, item)
    }

    pub const fn push(self, item: T) -> Self {
        if self.len < CAP {
            unsafe { self.push_unchecked(item) }
        } else {
            panic!()
        }
    }

    pub const fn try_push(self, item: T) -> Result<Self, CapacityError<T, CAP>> {
        if self.len < CAP {
            unsafe { Ok(self.push_unchecked(item)) }
        } else {
            Err(CapacityError { vector: self, item })
        }
    }

    pub const unsafe fn push_unchecked(mut self, item: T) -> Self {
        debug_assert!(self.len < CAP);
        self.xs[self.len] = MaybeUninit {
            // TODO actually make this unchecked
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

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array, const N: usize> From<ConstVec<A::Item, N>> for smallvec::SmallVec<A> {
    fn from(value: ConstVec<A::Item, N>) -> Self {
        smallvec::SmallVec::new().tap_mut(|v| v.extend(value))
    }
}
#[cfg(feature = "arrayvec")]
impl<T, const N: usize> From<ConstVec<T, N>> for arrayvec::ArrayVec<T, N> {
    fn from(value: ConstVec<T, N>) -> Self {
        arrayvec::ArrayVec::new().tap_mut(|v| v.extend(value))
    }
}
