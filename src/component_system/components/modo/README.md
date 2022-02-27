# Modération

Plusieurs commandes de modération sont disponible pour modérer la communauté au sein du serveur.

## Commandes

```
/ban <qui:@id_user> <pourquoi:explication> [pendant:durée]
```

Bannir un membre du serveur. Temporaire si le [paramètre *pendant*](#format-paramètre-pendant) est renseigné. Un message avec la raison et la durée du bannissement si renseignée est envoyé au membre bani.

### Paramètres

* **qui** : Le membre à bannir.
* **pourquoi** : La raison du ban. S'enregistre dans le ban discord et est envoyé au membre.
* **pendant** (*opt*) : Pendant combien de temps. Indéfiniment si non renseigné. 

```
/mute <qui:@id_user> <pourquoi:explication> [pendant:durée]
```

Attribue le rôle *muted* à un membre. Temporaire si le [paramètre *pendant*](#format-paramètre-pendant) est renseigné. Un message avec la raison et la durée du mute si rensignée est envoyé au membre en sourdine.

### Paramètres

* **qui** : Le membre à mute.
* **pourquoi** : La raison du mute. S'enregistre dans le ban discord et est envoyé au membre.
* **pendant** (*opt*) : Pendant combien de temps. Indéfiniment si non renseigné. 

```
/unban <qui:@id_user>
```

Débannir un membre du serveur.

### Paramètres

* **qui** : Le membre à débannir.

```
/unmute <qui:@id_user>
```

Retire le rôle *muted* à un membre.

### Paramètres

* **qui** : Le membre à unmute.

## Notes

### Format paramètre *pendant*

```
duration    ::= integer unit | time
integer     ::= digit+
digit       ::= "0"..."9"
unit        ::= "sec" | "min" | "hr" | "jr" | "sem" | "mo" | "an" 
time        ::= digit{1,2} ":" digit{2} ":" digit{2}
```

Exemples : 

* 4 jours: 4jr
* 3 semaines: 3sem
* 10 heures: 10h 
* 2 heures et 13 minutes: 2:13:00