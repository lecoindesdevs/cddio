# Slash commands

Les commandes slashes du bot sont créés à partir des noeuds de commandes des composants puis mises à jour sur chaque serveur. Par défaut, toute nouvelle commande est innacessible par tout le monde sauf les utilisateurs définis dans le fichier config.ron du bot. 

Les commandes de ce composants permettent de définir les permissions de chaque commandes pour des roles ou des membres.

## Commandes

```
/slash permissions set <command:nom commande> <who:role/membre> <type:"allow"/"deny">
```

Autoriser ou interdire une commande à un membre ou un rôle

### Paramètres

* **command** : Nom de la commande
* **who** : Qui est affecté (un rôle ou un membre)
* **type** : Type d'autorisation. "allow" (Autorisé) ou "deny" (Refusé)

-------

```
/slash permissions reset <command:nom commande>
```

Retire toutes les permissions d'une commande

### Paramètres

* **commande** : Nom de la commande

-------

```
/slash permissions remove <command:nom commande> <who:role/membre>
```

Efface la permission d'un membre ou d'un rôle à une commande.

### Paramètres

* **command** : Nom de la commande
* **who** : Qui est affecté (un rôle ou un membre)

-------

```
/slash permissions list
```

Liste les permissions des commandes sur le serveur.

