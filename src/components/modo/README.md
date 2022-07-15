# Modération

Plusieurs commandes de modération sont disponible pour modérer la communauté au sein du serveur.

## Commandes

### /ban

Banni un membre du serveur

#### Arguments

* **qui**: Membre à bannir
* **raison**: Raison du ban
* **historique** (optionnel): Supprimer l'historique du membre (nombre de jours de 0 à 7)
* **duree** (optionnel): Durée du ban ([voir le format ici](#format-paramètre-pendant))

### /kick

Expulse un membre du serveur

#### Arguments

* **qui**: Membre à expulser
* **raison**: Raison de l'expulsion

### /mute

Mute un membre du serveur

#### Arguments

* **qui**: Membre à mute
* **raison**: Raison du ban
* **duree** (optionnel): Durée du mute ([voir le format ici](#format-paramètre-pendant))

### /unban

Débanni un membre du serveur

#### Arguments

* **qui**: Membre à débannir

### /unmute

Démute un membre du serveur

#### Arguments

* **qui**: Membre à démute

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