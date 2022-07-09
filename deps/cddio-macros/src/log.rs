use std::fmt::Display;
#[inline]
pub fn log<T:Display>(t: &T) {
    if cfg!(feature = "verbose") {
        println!("{}", t);
    }
}