//! Fonctions utiles pour l'ensemble du projet
#[doc = "Macro pour créer un VecDeque de la même manière que vec!"]
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