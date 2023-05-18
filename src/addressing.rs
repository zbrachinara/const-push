/// This is a macro meant for internal use on `ConstVec`. It copies the element at the given index
/// to the stack, not modifying the original index.
///
/// # Safety
///
/// At the time of writing, there are a lot of limitations around const. In this case, the relevant
/// issue is that we are not allowed to get references to objects which may contain [`UnsafeCell`]s
/// -- both `&self` and `&self.xs` are impossible. This is used to do operations such as remove,
/// remove_swap, or pop, which would usually require a reference to `self.xs` at least in order to
/// copy its contents onto the stack. Thus if T contains an `UnsafeCell` this approach probably
/// isn't allowed, or maybe it just isn't allowed at all because of the strange things we do with a
/// zst pointer.
///
/// And of course, since this function performs a copy of a non-copy type, you need to make sure
/// that *the element at this index is never accessed as a `T` again*.
macro_rules! copy_item {
    ($self:ident<$item_type:ty>[$ix:expr]) => {{
        // we can't get a pointer to xs or self, but we can get one to a zst with the same address
        let ptr_to_xs = crate::addressing::extract_addr!($self<$item_type>);
        // we have a pointer to our array now, but we need a pointer to the item's location
        let ptr_to_elem = ptr_to_xs.add($ix);
        // and then use a ptr read obtain the item
        ::core::ptr::read(ptr_to_elem)
    }};
}

#[repr(C)]
pub struct AddressExtractor<T, const N: usize> {
    pub xs_addr: (),
    #[allow(dead_code)]
    xs: [T; N],
}

impl<T, const N: usize> AddressExtractor<T, N> {
    pub const fn new(arr: [T; N]) -> Self {
        Self {
            xs: arr,
            xs_addr: (),
        }
    }
}


macro_rules! extract_addr {
    ($self:ident<$item_type:ty>) => {
        ::core::ptr::addr_of!($self.xs_addr) as *const $item_type
    };
}

pub(crate) use {copy_item, extract_addr};
