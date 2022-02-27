//! # Bot OpenCDD
//! 
//! ## Bot asynchrone
//! 
//! Le bot se repose sur la crate tokio pour bénéficier de la gestion des tâches asynchrones.
//! 
//! ## Bot orienté par composants
//! 
//! La structure du bot est composée de plusieurs composants.
//! 
//! Chaque composant correspond à un domaine de fonctionnalité. 
//! 
//! ## Pour implémenter une nouvelle fonctionnalité
//! 
//! Pour ajouter une nouvelle fonctionnalité, créez un fichier dans le dossier `src/component/components`. 
//! Dans ce fichier, créez une struct implémentant le trait [`Component`].
//! Rendez le module publique dans le module [`component::components`].
//! Enfin, ajoutez le composant dans la fonction [`Bot::new`].
//! 
//! Prenez exemple sur le composant [`misc`] si nécessaire.
//! 
//! [`Component`]: crate::component::Component
//! [Bot::new]: crate::bot::Bot::new()
//! [`misc`]: crate::component::components::misc

mod bot;
mod component_system;
mod config;
#[macro_use]
mod util;

/// Trait à implémenter pour logger les erreurs dans la console.
trait ResultLog {
    type OkType;
    /// Si une erreur se produit, panic et log le message en entrée et l'erreur.
    /// Sinon, renvoie la valeur.
    fn expect_log(self, msg: &str) -> Self::OkType;
}
impl<T, S: AsRef<str>> ResultLog for Result<T, S> {
    type OkType=T;
    fn expect_log(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) if msg.is_empty() => panic!("{}", e.as_ref()),
            Err(e) => panic!("{}: {}", msg, e.as_ref()),
        } 
    }
}

#[tokio::main]
async fn main() {
    let config = config::Config::load("./config.ron").expect_log("Could not load the configuration file");
    let mut bot = bot::Bot::new(&config).await
        .or_else(|e|Err(e.to_string()))
        .expect_log("");
    bot
        .start().await
        .or_else(|e| Err(e.to_string()))
        .expect_log("Could not start the bot");
}
