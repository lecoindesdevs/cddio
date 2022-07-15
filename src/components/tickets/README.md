# Gestion des tickets

Le serveur du Coin des développeurs dispose d'un service de tickets permettant d'intéragir avec le staff en créant des salons réservés pour le créateur du salon et pour le staff.
La création des tickets se fait grâce à un menu déroulant présent dans le salon prévu pour ça (actuellement #📚・ticket-staff).

Pour créer un nouveau ticket, allez dans le salon approprié, et selectionnez une catégorie correspondante à votre demande. Un salon textuel va se créer, le bot vous mentionnera. Pour fermer ce ticket, appuyez sur le bouton "Fermer le ticket" dans le message du bot dans ce salon (ce message sera épinglé pour accéder au bouton facilement). Vous pouvez aussi utiliser la commande [/ticket close](#tickets-close) pour fermer le ticket.

## Commandes

### /tickets categories add

Ajoute une catégorie de ticket. À ne pas confondre avec les catégories discord

#### Arguments

* **nom**: Nom de la catégorie
* **categorie_discord**: Catégorie Discord où les tickets seront créés
* **prefix**: Préfixe des tickets
* **description** (optionnel): Description de la catégorie

### /tickets categories remove

Supprime une catégorie de ticket

#### Arguments

* **nom**: Nom de la catégorie

### /tickets categories list

Liste les catégories de ticket


### /tickets set_channel

Assigne le salon de création de tickets

#### Arguments

* **salon** (optionnel): Salon textuel

### /tickets close

Ferme le ticket actuel


### /ticket add_member

Ajoute une personne au ticket

#### Arguments

* **qui**: Personne à ajouter au ticket