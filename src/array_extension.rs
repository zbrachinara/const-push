use crate::{
    addressing::{extract_addr, AddressExtractor},
    assertions, MaybeUninit,
};

pub(crate) const fn extend_uninit_array<T, const N: usize, const CAP: usize>(
    xs: [MaybeUninit<T>; N],
) -> [MaybeUninit<T>; CAP] {
    let addressor = AddressExtractor::new(xs);
    let address = extract_addr!(addressor<MaybeUninit<T>>);
    let copy = copy_from_raw::<_, N, CAP>(address);
    core::mem::forget(addressor);
    copy
}

pub(crate) const fn extend_array<T, const N: usize, const CAP: usize>(
    xs: [T; N],
) -> [MaybeUninit<T>; CAP] {
    let addressor = AddressExtractor::new(xs);
    let address = extract_addr!(addressor<MaybeUninit<T>>);
    let copy = copy_from_raw::<_, N, CAP>(address);
    core::mem::forget(addressor);
    copy
}

const fn copy_from_raw<T, const N: usize, const CAP: usize>(
    xs: *const MaybeUninit<T>,
) -> [MaybeUninit<T>; CAP] {
    assertions::Leq::<N, CAP>::assert();
    let mut buffer: [MaybeUninit<T>; CAP] = unsafe { MaybeUninit::uninit().assume_init() };

    let mut ix = 0;
    while ix < N {
        buffer[ix] = unsafe { xs.add(ix).read() };
        ix += 1;
    }

    buffer
}
