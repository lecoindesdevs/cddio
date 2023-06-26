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

use async_std::channel;
use sea_orm::EntityTrait;
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

struct Handler {
    db: sea_orm::DbConn
}


// #[tokio::main]
async fn _main() {
    if let Err(e) =  log::init() {
        panic!("Unable to set logger: {}", e);
    }
    let db = match db::start_db("sqlite:./data.db?mode=rwc").await {
        Err(e) => panic!("Unable to start the database: {}", e),
        Ok(v) => v
    };
    // {
    //     use sea_orm::prelude::*;
    //     let active_model = db::archive::ActiveModel {
    //         channel_id: sea_orm::ActiveValue::Set(920707775313621033),
    //         opened_by: sea_orm::ActiveValue::Set(381478305540341761),
    //         closed_by: sea_orm::ActiveValue::Set(153569924277731330),
    //         ..Default::default()
    //     };
    //     let res = db::archive::Entity::insert(active_model).exec(&db).await.expect("Unable to create the archive");
    // }

    let _config = config::Config::load("./config.json").expect_log("Could not load the configuration file");

    let user = db::model::discord::User::find_by_id(381478305540341761 as db::IDType).one(&db).await.expect("Unable to find the user").expect("no user found with id 381478305540341761");
    let tickets = user.opened_archives().find_also_related(db::model::discord::Channel).all(&db).await.expect("Unable to get ticket opened by user");
    println!("List odf tickets open by {}", user.id);
    for ticket in tickets.into_iter().filter_map(|(_ticket, chan)| chan) {
        println!("    - {}", ticket.name);
        println!("    Messages:");
        for msg in ticket.messages().all(&db).await.expect("Unable to get messages") {
            println!("        - {}", msg.content);
        }
    }

    
    // let client = Client::builder(&config.token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
    //     .application_id(config.app_id)
    //     .event_handler(Handler{db})
    //     .await
    //     .expect("Could not create the client")
    //     .start()
    //     .await
    //     .expect("Could not start the client");

    
}

#[tokio::main]
async fn main() {
    if let Err(e) =  log::init() {
        panic!("Unable to set logger: {}", e);
    }
    let config = config::Config::load("./config.json").expect_log("Could not load the configuration file");
    let mut bot = bot::Bot::new(&config).await
        .or_else(|e|Err(e.to_string()))
        .expect_log("");
    bot
        .start().await
        .or_else(|e| Err(e.to_string()))
        .expect_log("Could not start the bot");
}
