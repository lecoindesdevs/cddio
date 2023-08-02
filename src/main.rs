/*!
# CDDIO

Bot Discord officiel du serveur Coin des Developpeurs ([Rejoignez nous !](https://discord.gg/m9EZNKVaPz))

Crée par la communauté pour la communauté.

Ce bot est développé en [**Rust**](https://www.rust-lang.org/) et repose sur la crate [`serenity`], [`cddio_core`] et [`cddio_macros`].

## Fonctionnalités

* [*Autobahn*, l'anti spam](components::autobahn)
* [Aide du bot](components::help)
* [Commandes diverses](components::misc)
* [Commandes de modération](components::modo)
* [Déclaration des slash commands](components::slash)
* [Gestion de ticket du serveur](components::tickets)
* [Dall-e Mini](components::dalle_mini)

## Licence

Ce projet est licencié sous **GPLv3**. 
Je vous invite à aller [sur cette page](https://choosealicense.com/licenses/gpl-3.0/) pour plus de renseignement.
*/

pub mod bot;
pub mod components;
pub mod config;
pub mod log;
pub mod db;

/// Trait à implémenter pour logger les erreurs dans la console.
trait ResultLog {
    type OkType;
    /// Si une erreur se produit, panic et log le message en entrée et l'erreur.
    /// Sinon, renvoie la valeur.
    fn expect_log(self, msg: &str) -> Self::OkType;
}

impl<T, S: std::fmt::Display> ResultLog for Result<T, S> {
    type OkType=T;
    fn expect_log(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) if msg.is_empty() => panic!("{}", e),
            Err(e) => panic!("{}: {}", msg, e),
        } 
    }
}
#[tokio::main]
async fn main() {
    if let Err(e) =  log::init() {
        panic!("Unable to set logger: {}", e);
    }
    let config = config::Config::load("./config.yaml").expect_log("Could not load the configuration file");
    let database = db::start_db("sqlite:./data.db?mode=rwc").await.or_else(|e| Err(e.to_string())).expect_log("Unable to start the database");
    let mut bot = bot::Bot::new(config, database).await
        .or_else(|e|Err(e.to_string()))
        .expect_log("");
    bot
        .start().await
        .or_else(|e| Err(e.to_string()))
        .expect_log("Could not start the bot");
}
