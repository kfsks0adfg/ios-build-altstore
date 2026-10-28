# Installer depuis l’iPhone sans ordinateur : état réel des solutions

Vérification mise à jour le 28 octobre 2026.

## Réponse courte

La méthode officielle reste inchangée : **AltStore Classic ne peut pas être
amorcé depuis l’iPhone seul** avec un compte Apple gratuit. Le runner macOS
GitHub compile l’IPA, mais ne peut pas établir la liaison avec l’iPhone.

Sur iOS 27, un nouveau jumelage initié par l’iPhone ouvre toutefois une voie
expérimentale. Ce dépôt contient maintenant un
[prototype iSH](../bootstrap-ios/README.md) qui annonce l’hôte de jumelage,
signe l’IPA localement et l’envoie directement à `installation_proxy` via
LocalDevVPN. Le code compile, mais le parcours complet n’est pas encore validé
sur l’iPhone cible ; il ne constitue donc pas encore une solution garantie.

SideStore supprime le besoin d’un ordinateur **après sa première installation**,
pas pendant celle-ci. Sa documentation officielle exige actuellement une
connexion USB ponctuelle avec `iloader`.

## Solution stable recommandée sans Windows ni Mac personnel

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

## Prototype 100 % iPhone pour iOS 27

Le prototype évite l’installation OTA, qui ne fonctionne pas avec un profil de
développement gratuit. Il suit ce chemin :

1. iSH annonce un hôte Bonjour `_remotepairing-pairable-host._tcp` ;
2. l’iPhone crée son fichier RPPairing depuis le menu Mode développeur ;
3. `isideload` signe SideStore avec le compte Apple gratuit ;
4. LocalDevVPN rend les services locaux de l’iPhone accessibles ;
5. le client ouvre le tunnel TLS-PSK/RSD et installe l’IPA avec
   `installation_proxy`.

Les limites matérielles à vérifier sont la diffusion Bonjour depuis iSH et la
suspension d’iSH lors du passage dans Réglages. Les étapes exactes et les règles
de sécurité sont dans le README du prototype.

## Note pour iOS 27.2

SideStore fonctionne sur iOS 27.2, mais des utilisateurs rencontrent actuellement
les erreurs de connexion Apple `-80045` ou `-80046`. Le contournement confirmé
dans [l’incident SideStore #1693](https://github.com/SideStore/SideStore/issues/1693)
est le suivant :

1. désactiver **On-Device Anisette** dans `Settings → User Customizations`, puis
   redémarrer SideStore quand il le demande ;
2. dans **Anisette Servers**, sélectionner **SideStore (New)**
   (`new.ani.sidestore.io`) et désactiver **Enable Auto Rotation** ;
3. ouvrir `Anisette Client Configuration → Remote Server Sync`, toucher
   **Fetch**, puis **Save as Overrides** ;
4. revenir dans `User Customizations` et toucher **Reset adi.pb** ;
5. connecter LocalDevVPN, tenter une seule connexion et attendre le code 2FA.

Cette procédure a aussi été confirmée par des utilisateurs de la version stable,
mais elle reste un contournement communautaire susceptible d’évoluer.

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

## Pourquoi GitHub Actions ne peut pas remplacer le jumelage

Le runner macOS est distant et éphémère. Il ne peut pas recevoir la confirmation
de jumelage et ne peut pas créer le fichier propre à l’appareil. Il produit l’IPA
et la source AltStore ; le prototype iOS 27 essaie de réaliser localement sur
l’iPhone la partie que GitHub ne peut pas faire.

