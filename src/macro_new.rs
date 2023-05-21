use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "fake-move")] {
        #[macro_export]
        macro_rules! constvec {
            () => { ::const_push::ConstVec<_, _>::new() };
            ($elem:expr; $n:literal..$cap:literal) => {
                ::const_push::constvec_by_array!($elem;$n..$cap)
            };
            ($($x:expr),+ $(,)? ; ..$cap:literal) => {
                ::const_push::constvec_by_array!($($x,)*; ..$cap)
            };
        }
    } else {

    }
}

#[macro_export]
macro_rules! constvec_by_array {
    ($elem:expr; $n:literal..$cap:literal) => {
        ::const_push::ConstVec::<_,$cap>::from_array([$elem;$n])
    };
    ($($x:expr),+ $(,)? ; ..$cap:literal) => {
        ::const_push::ConstVec::<_,$cap>::from_array([$($x,)+])
     };
}
