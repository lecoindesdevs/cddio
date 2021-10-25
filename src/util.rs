#[doc = "macro make vec deque"]
#[doc(alias = "std::collections::VecDeque")]
#[macro_export]
macro_rules! vdq {
    ($($args:expr),*) => {
        {
            let mut v = std::collections::VecDeque::new();
            $(v.push_back($args);)*
            v
        }
    };
}