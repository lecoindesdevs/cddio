# CDDIO MACROS

## Exemple d'utilisation

```rust
use cddio_core::{ApplicationCommandEmbed, message};
use serenity::{
    client::Context,
    event::*,
    model::{
        event::ReadyEvent,
        id::ChannelId
    }
}

struct MyComponent;

#[component]
impl MyComponent {
    /// Nom de la commande Discord: ping
    /// Arguments: (aucun)
    /// Description: Renvoie un message 'Pong!'
    #[command(name="ping", description="Renvoie un message 'Pong!'")]
    async fn ping_cmd(ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>)
    {
        if let Err(e) = app_cmd.direct_response(ctx, message::success("Pong!")).await {
            println!("ping: Erreur lors de la réponse: {}", e);
        }
    }
    /// Nom de la commande Discord: creer_embed
    /// Arguments: 
    ///     - titre (obligatoire): type Texte,      Titre de l'embed
    ///     - contenu (obligatoire): type Texte,    Contenu de l'embed
    ///     - salon (optionnel): type ChannelId,    Salon où l'envoyer. Salon actuel par défaut
    ///     
    /// Description: Renvoie un message 'Pong!'
    #[command(description="Renvoie un message 'Pong!'")]
    async fn creer_embed(ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>
        #[argument(description="Titre de l'embed")]
        titre: String,
        #[argument(name="contenu", description="Contenu de l'embed")]
        content: String,
        #[argument(description="Salon où l'envoyer. Salon actuel par défaut")]
        salon: Option<ChannelId>,
    )
    {
        /// implémentation...
    }
    /// Evenement appelé lorsque le bot est prêt
    #[event(Ready)]
    async fn on_ready(ctx: &Context, _evt_ready: &ReadyEvent)
    {
        println!("Bot is ready");
    }
}
```

## API

Cette crate ne possède que des macros de type attribut à attacher à des symboles Rust.

Sauf dans le cas de l'attribut event, les arguments prennent la forme suivante: `nom_argument_1="valeur 1", nom_argument2="valeur 2", ...`. L'ordre des arguments n'importe pas.

### `#[component]`

Déclare une implémentation de structure en composant.

Sans cette attribut, les autres attributs qui suivent ne seront pas détectés correctement. Il est impératif de l'appliquer à une implémentation de structure. Cette attribut ne doit être utilisé sur qu'une seule implémentation par structure.

```rust
struct MyStruct;

#[component]
impl MyStruct
{

}
```

### `#[group()]`

Déclare un groupe de commande Discord. 

Par exemple pour créer la commande `ticket create`, *ticket* est un groupe de commande et *create* est une commande associée à ce groupe. Voir l'API [command](#command) pour associer une commande à un groupe. Un groupe peut s'associer à un autre groupe avec l'argument *parent*. Il faut que le groupe *parent* soit déclaré avant le groupe en cours. Le nom du groupe ne doit pas contenir de caractère blanc. Les attributs *group* doivent être déclaré sur l'implémentation de la structure en dessous de l'attribut [component](#component).

|argument|optionnel|description|
|:-|:-:|:-|
|*name*| |Nom du groupe|
|*description*| |Description du groupe|
|*parent*|x|Nom du groupe sur lequel s'associer|

```rust
struct MyStruct;

#[component]
#[group(name="ticket", description="Gestion des tickets")]
#[group(name="member", description="Gestion des membre dans un ticket", parent="ticket")]
impl MyStruct {
    #[command(name="add", description="Ajouter un membre au ticket")]
    async fn ticket_member_add(ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à ajouter")]
        member: UserId
        #[argument(name="bienvenue", description="Message de bienvenue")]
        welcome: Option<String>
    ) 
    {}
}
```

### `#[command()]`

Déclare une commande Discord à partir d'une fonction Rust. 

Le nom de la commande ne doit pas contenir de caractère blanc. Une commande déclaré sans group est déclaré au premier niveau, c'est à dire que son nom est disponible juste après le slash. Pour créer des groupes de commande, allez voir l'attribut [group](#group). Pour associer une commande à un groupe, utilisez l'argument *parent* avec le nom du groupe. Les commandes n'ont aucun argument obligatoire. Néanmoins, le contexte et l'application command seront nécessaire pour répondre à la commande. Dans un but de simplification, l'argument de l'application command utilise un type custom de cddio-core : `ApplicationCommandEmbed<'_>`. Si la commande discord a besoin de paramètre, vous pouvez ajouter des arguments à la fonction Rust en plus du contexte et de l'app cmd. Voir l'attribut [argument](#argument) pour plus de détails.

|argument|optionnel|description|
|:-|:-:|:-|
|*name*|x|Nom de la commande. Utilise le nom de la fonction rust si non renseigné|
|*description*| |Description de la commande|
|*group*|x|Nom du groupe sur lequel s'associer|

Voir l'exemple d'une commande dans l'attribut [group](#group)

### `#[argument()]`

Déclare un argument de commande à un paramètre de fonction Rust.

Le nom de l'argument ne doit pas contenir de caractère blanc. Parce que la description d'un argument est obligatoire, l'attribut *argument* est obligatoire pour chaque paramètre de fonction qui corresponde à un argument de commande Discord.

|argument (de l'attribut)|optionnel|description|
|:-|:-:|:-|
|*name*|x|Nom de l'argument. Utilise le nom de la variable si non renseigné|
|*description*| |Description de l'argument|

Le type du paramètre de fonction est restreint à ce que peut recevoir une commande Discord. Voici la liste des types supportés : 

|Type rust|Type API Discord|Description|
|-:|:-|:-|
|String|String||
|u64, u32, u16, u8, i64, i32, i16, i8|Integer|Un nombre entier|
|f32, f64|Number|Un nombre à virgule|
|bool|Boolean|Un état|
|User*, UserId*|User|Un utilisateur|
|Role*, RoleId*|Role|Un role|
|PartialChannel, ChannelId|Channel|Un salon (peut etre textuel, vocal, catégorie, stage ou fil)|
|Mentionable**|Mentionable|Peut être un utilisateur ou un role|

*: Type disponible dans la crate serenity

**: Type disponible dans la crate cddio-core

Si l'argument de la commande discord doit être optionnel, encapsulez l'un des types au dessus dans un std::Option<...> 

Voir l'exemple d'un argument commande dans l'attribut [group](#group)

### `#[event()]`

Déclare un événement Discord.

L'attribut event a deux fonctionnement : le mode identifiant et le mode pattern.

**Dans le mode identifiant**, l'attribut prend pour seul argument le nom de l'événement (qui est un item de l'enumérateur Event dans la crate *serenity*). Vous pouvez retrouver la liste des événements supportés dans [la documentation de la crate *serenity*](https://docs.rs/serenity/latest/serenity/model/event/enum.Event.html). 

Dans ce mode là, la fonction que l'attribut attache doit **nécessairement** avoir pour arguments la référence du contexte puis la référence de la structure que l'enumérateur *serenity* embarque.

```rust
#[event(Ready)]
async fn on_ready(ctx: &Context, evt_ready: &ReadyEvent)
{}
```

**Le mode pattern** se base sur l'énumérateur Event de serenity. Il est possible d'extraire les valeurs des structure pour les utiliser en argument de fonction. L'ordre et le contenu des argument n'importe pas et le contexte peut être omis. Les evenement et les enumeration

```rust
#[event(GuildBanAdd(GuildBanAddEvent{user, guild_id}) | GuildBanRemove(GuildBanRemoveEvent{user, guild_id}))]
async fn on_guild_ban(ctx: &Context, user: &User, guild_id: &GuildId)
{}
```

### `#[message_component()]`

Déclare un événement Discord de type *message component*.

|argument|optionnel|description|
|:-|:-:|:-|
|*custom_id*| |custom_id intégré au message component|


```rust
#[message_component(custom_id="button_ticket_close")]
async fn on_button_ticket_close(&self, ctx: &Context, msg: &MessageComponentInteraction) 
{}
```

Cette attribut est un helper en plus de l'attribut [event](#event). L'équivalent de l'exemple au dessus en utilisant l'attribut event :

```rust
use serenity::model::{
    event::{
        Event::InteractionCreate,
        InteractionCreateEvent
    },
    interaction::Interaction::MessageComponent
}
#[event(InteractionCreate(InteractionCreateEvent{interaction: MessageComponent(message_interaction), ..}) if message_interaction.data.custom_id == "button_ticket_close")]
async fn on_button_ticket_close(&self, ctx: &Context, message_interaction: &MessageComponentInteraction) 
{}
```