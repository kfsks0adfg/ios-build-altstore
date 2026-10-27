# Installer depuis l’iPhone sans ordinateur : état réel des solutions

Vérification effectuée le 27 octobre 2026.

## Réponse courte

Sur un iPhone standard à jour, **AltStore Classic ne peut pas être amorcé depuis
l’iPhone seul** avec un compte Apple gratuit. Apple exige qu’une première app de
sideloading soit signée puis installée par un hôte déjà autorisé. Le runner macOS
GitHub peut compiler l’IPA, mais il ne peut pas établir la liaison USB et la
relation de confiance avec votre iPhone.

SideStore supprime le besoin d’un ordinateur **après sa première installation**,
pas pendant celle-ci. Sa documentation officielle exige actuellement une
connexion USB ponctuelle avec `iloader`.

## Solution recommandée sans Windows ni Mac

Utilisez temporairement l’un des hôtes acceptés par SideStore :

1. un ordinateur Linux emprunté ;
2. un Chromebook récent non administré, avec l’environnement Linux activé ;
3. un Raspberry Pi ou un mini-PC Linux auquel l’iPhone peut être relié en USB.

Installez ensuite SideStore avec [`iloader`](https://iloader.app), en suivant les
[prérequis officiels](https://docs.sidestore.io/docs/installation/prerequisites)
et la [procédure officielle](https://docs.sidestore.io/docs/installation/install).
Une fois cette étape terminée, SideStore peut installer et rafraîchir les IPA
depuis l’iPhone avec LocalDevVPN et une connexion Wi-Fi.

Ce pack recommande SideStore plutôt qu’AltStore Classic dans ce scénario, car
AltStore Classic continue normalement à dépendre d’AltServer pour les
rafraîchissements.

## Cas où une installation 100 % iPhone peut exister

### AltStore PAL dans l’Union européenne

AltStore PAL peut être installé depuis le Web dans les régions éligibles. Il ne
résout cependant pas ce projet : PAL distribue des applications notarées au
format marketplace/ADP et ne transforme pas une IPA personnelle non signée en
app installable avec un compte gratuit.

### TrollStore, jailbreak ou faille compatible

Certains anciens appareils et certaines versions précises d’iOS permettent un
amorçage sans ordinateur grâce à TrollStore ou à un jailbreak. Cette possibilité
dépend exactement du modèle de l’iPhone, de la version iOS et parfois de la
méthode utilisée pour arriver sur cette version. Elle n’est pas disponible sur
un iPhone actuel générique et ne doit pas être présentée comme méthode universelle.

### Certificat de signature déjà possédé

Si vous possédez déjà un certificat Apple de distribution valide et son fichier
privé, des outils iPhone peuvent signer et installer une IPA. Cela n’est plus la
solution gratuite demandée et nécessite de protéger soigneusement la clé privée.

## Méthodes déconseillées

Évitez les sites « direct install », profils MDM inconnus et boutiques utilisant
des certificats d’entreprise partagés :

- leurs certificats sont fréquemment révoqués ;
- ils peuvent modifier les IPA ;
- un profil de gestion peut donner des droits importants sur l’iPhone ;
- ne communiquez jamais le mot de passe ni le code 2FA de votre compte Apple à
  un service Web de signature.

## Pourquoi GitHub Actions ne peut pas remplacer l’étape USB

Le runner macOS est distant et éphémère. Il n’a aucun accès USB à l’iPhone, ne
peut pas recevoir la confirmation « Faire confiance à cet ordinateur » et ne
peut pas créer le fichier d’appairage propre à l’appareil. Il peut donc produire
l’IPA et la source AltStore, mais pas amorcer AltStore/SideStore sur le téléphone.

