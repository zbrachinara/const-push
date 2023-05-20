#[macro_export]
macro_rules! constvec {
    () => { ... };
    ($elem:expr; $n:expr) => { ... };
    ($($x:expr),+ $(,)?) => { ... };
}
