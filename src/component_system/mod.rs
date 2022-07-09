//! Un composant est une partie du bot. 
//! 
//! Un composant à un domaine défini, tel que là modération, la gestion des tickets ou d'autres taches du genre.
//! Chacun peuvent contenir un set de commandes et d'acquisition événements et réponde à certaines taches de leur domaine.
//! Un composant est sensé s'autogérer mais Mais rien n'empêche la communication entre ces derniers.
pub mod components;

// pub use framework::{Framework , FrameworkConfig};
// pub use framework::{Context, Message};
pub use serenity::model::event::Event;
use serenity::client::Context;
