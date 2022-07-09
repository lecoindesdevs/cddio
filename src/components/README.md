# Système de composants au sein du bot

Le bot a été pensé pour être facilement maintenable dans le temps. Pour simplifier son développement, la partie core du bot a été développé séparément de ses fonctionnalités sur Discord.

Les outils pour les composants sont présents dans le module component_system et les composants sont dans le module components.

## Composition du système de composant

### Data : données persistente d'un composant

Le module data sert à tout ce qui concerne les données persistentes des composants. On peut l'utiliser pour enregistrer des paramètres ou des données à sauvegarder pour les réutiliser plus tard. Par exemple, un ban temporaire dans le module modo enregistre qui est concerné et le moment où doit prendre fin le bannissement.

C'est la structure Data gère cette sauvegarde. La structure demande un type générique qui représente le modèle de données. La structure comporte deux fonctions permettant l'accès aux données : read() et write(). Ces deux fonctions retournent un guardian qui bloque l'accès en écriture tant que tous les guardians ne sont pas détruit.

Le chemin des fichiers de données sont accessibles grace à la variable static DATA_DIR, ce qui peut être utile pour créer un fichier de données qui n'est pas contraint par la structure Data mais qui se trouve dans le même dossier que les enregistrements de Data. Par exemple, les tickets du module tickets sont enregistrées dans un dossier a part des fichier de données dans le dossier data.

### Event : réception d'événements

Les événements Discord sont gérés grâce au module event. Il n'y a pas grand chose à dire par rapport à ce module à l'exception qu'un tokio::thread est créer par événement par composant.
En d'autres termes, lorsqu'un événement Discord est recu par le bot, une tache individuelle est créé pour chacun des composants. Il faut par conséquent s'assurer que la communication entre composants doit se faire de manière **thread safe**.

### Les commandes

#### Le module command_parser

Le module command_parser est là pour aider à créer des commandes de bot aisément. Toutes les commandes et sous commandes d'un composant peuvent être défini par l'appel d'une suite de fonction.

Pour bénéficier des composants système `slash` et `help`, le composant doit retourner son noeud de commandes dans la fonction du trait `Component::node()`.

Pour comprendre comment utiliser ce module, référez vous à la documentation technique et inspirez vous de la définition des autres commandes de composants lors de leur création (dans les fonctions `new()` des composants ).

#### Composants `slash` et `help`

Les composants `slash` et `help` sont deux composants système, ce qui signifie qu'elles sont nécessaires au fonctionnement du bot. Ces composants repose sur les noeuds de commandes des composants pour fonctionner.

Le composants `slash` permet générer les commandes slashs à partir des noeuds de commande de tous les composants puis de mettre en ligne sur chaque serveur où le bot est présent. Des commandes de gestion des slashs commandes (entre autre au niveau des permissions des commandes au sein d'un serveur) y sont disponible. Référez vous à [la documentation de ce composant](components/slash) pour savoir comment l'utiliser.

Le composant `help` affiche la documentation des commandes du bot sur Discord. Référez vous à [la documentation de ce composant](components/help) pour savoir comment l'utiliser.

### Manager

Le manager est le conteneur des composants. Le manager peut être passé à d'autres composants pour traiter les informations des composants au sein d'un composant (ex. le cas des composants `slash` et `help` : à leur création, une copie du manager leur est donnée).

Contrairement à ce que son nom suggère, le manager n'a pas d'autre utilité.

## Créer un composant

La création est simple : 

**Ajoutez un composant au module components**. Tous les composants sont défini dans [ce module](components/mod.rs) par un dossier qui leur est propre. Créez le votre et pensez à rendre votre composants accessible en publique dans le fichier mod.rs.

**Appliquer le trait `cddio::component_system::Component`**. Sans ce trait, le composant ne pourra pas être pris en charge (une erreur de compilation va apparaitre au moment de l'ajouter au manager). Le trait `Component` nécessite la définition de quelques fonctions telles que `name()` et `event()`.

**Instanciez et ajoutez le composants dans le bot**. Pour se faire, allez dans le fichier bot.rs, dans la fonction `Bot::new()` et ajoutez une nouvelle ligne dans la liste des composants du manager via `Manager::add_component`.

*And voilà* comme le disent les Américains. Votre composant est disponible dans les fonctionnalités du bot. 