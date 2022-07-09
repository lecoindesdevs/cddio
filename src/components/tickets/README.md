# Gestion des tickets

Le serveur du Coin des développeurs dispose d'un service de tickets permettant d'intéragir avec le staff en créant des salons réservés pour le créateur du salon et pour le staff.
La création des tickets se fait grâce à un menu déroulant présent dans le salon prévu pour ça (actuellement #📚・ticket-staff).

Pour créer un nouveau ticket, allez dans le salon approprié, et selectionner une catégorie correspondante à votre demande. Un salon textuel va se créer, le bot vous mentionnera. Pour fermer ce ticket, appuyez sur le bouton "Fermer le ticket" dans le message du bot dans ce salon.

## Commandes

Les commandes de ce module concernent la gestion du menu déroulant, à savoir dans quel channel placer le menu, et quelles catégories y faire référence. Sur le serveur du Coin des développeur, les commandes sont réservées aux membres du staff.


```
/ticket set channel <id:#channel_id>
```

Assigne le salon où le menu déroulant pour créer le ticket doit apparaitre. Une fois la commande lancé, si un ancien menu avait été mis en place, il sera supprimé, puis un nouveau menu sera créer à l'emplacement souhaité.

### Paramètres

* **id** : ID du salon (textuel uniquement). Si la commande est lancée par commande slash, le paramètre id vous demandera directement un salon à renseigner.


```
/ticket categories add <name:texte> <id:#channel_id> <prefix:texte> [desc:texte]
```

Ajouter une nouvelle catégorie dans le menu.

### Paramètres

* **name** : Nom de la catégorie. Ce sera le titre de la catégorie affiché dans le menu.
* **id** : Identifiant de la catégorie Discord. Attention à ne selectionner qu'une catégorie et pas un salon textuel.
* **prefix** : préfixe des tickets de cette catégories. Lorsqu'un ticket sera créé, le nom du salon prendra pour format `{prefix}_{username}`
* **desc** : Description de la catégorie. Ce sera la description de la catégorie affichée dans le menu.


```
/ticket categories remove <name:texte>
```

Retire une catégorie du menu. La suppression est définitive.

### Paramètres

* **name** : Nom de la catégorie attribué précédement via la commande `/ticket categories add`


```
/ticket categories list
```

Lister les catégories de ticket déjà attribués. Ceux ci sont présent dans le menu déroulant