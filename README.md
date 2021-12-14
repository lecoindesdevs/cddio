# OpenCDD

Bot Discord officiel du serveur Coin des Developpeur ([Rejoignez nous !](https://discord.gg/m9EZNKVaPz))

Crée par la communauté pour la communauté.

Ce bot est développé en **Rust** et utilise [serenity](https://crates.io/crates/serenity) pour l'API Discord.

## Fonctionnalités

### Système de tickets

*todo*

### Modération

Plusieurs commandes de modération sont disponible pour modérer la communauté au sein du serveur.

#### Commandes

```
/ban <qui:@id_user> <pourquoi:explication> [pendant:durée]
```

Bannir un membre du serveur. Temporaire si le parametre *pendant* est renseigné. Un message avec la raison et la durée du bannissement est envoyé au membre bani.

*Parametres*

* qui : Le membre à bannir.
* pourquoi : La raison du ban. S'enregistre dans le ban discord et est envoyé au membre.
* pendant (*opt*) : Pendant combien de temps. Indéfiniment si non renseigné. 

```
/mute <qui:@id_user> <pourquoi:explication> [pendant:durée]
```

Attribue le rôle *muted* à un membre. Temporaire si le parametre *pendant* est renseigné. Un message avec la raison et la durée du mute est envoyé au membre bani.

*Parametres*

* qui : Le membre à mute.
* pourquoi : La raison du mute. S'enregistre dans le ban discord et est envoyé au membre.
* pendant (*opt*) : Pendant combien de temps. Indéfiniment si non renseigné. 

```
/unban <qui:@id_user>
```

Débannir un membre du serveur.

*Parametres*

* qui : Le membre à débannir.

```
/unmute <qui:@id_user>
```

Retire le rôle *muted* à un membre.

*Parametres*

* qui : Le membre à unmute.


### Aide du bot

La commande help permet d'afficher l'aide d'une commande ou la liste des commandes du bot.

#### Commande

```
/help [commande:nom commande]
```

Affiche l'aide d'une commande ou la liste des commandes du bot si le parametre commande n'est pas précisé.

*Paramètres*

* commande : Nom de la commande

## Licence

Le code et l'utilisation de ce bot est **réservé** au serveur du **Coin des Developpeurs**. 

