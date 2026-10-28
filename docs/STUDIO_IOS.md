# Forge iOS — piloter le développement depuis l’iPhone

Forge iOS est l’interface web installable publiée par ce dépôt. Elle ne contient
pas Xcode : elle envoie les opérations à GitHub, puis GitHub Actions compile sur
un runner macOS.

## Installation de l’interface

1. Ouvrez `https://UTILISATEUR.github.io/DEPOT/` dans Safari.
2. Touchez le bouton de partage.
3. Choisissez **Sur l’écran d’accueil**, puis **Ajouter**.
4. Ouvrez Forge iOS depuis son icône.

Cette PWA ne demande pas de mode développeur et n’expire pas après sept jours.

## Lecture publique

Sans jeton, Forge iOS peut :

- afficher les dernières exécutions GitHub Actions ;
- afficher et télécharger les Releases ;
- lire les fichiers texte du dépôt ;
- afficher et copier la source AltStore/SideStore.

## Autoriser les modifications et les builds

1. Dans GitHub, ouvrez **Settings → Developer settings → Personal access tokens
   → Fine-grained tokens**.
2. Limitez le jeton au seul dépôt utilisé par Forge iOS.
3. Accordez **Actions: Read and write**.
4. Accordez **Contents: Read and write**.
5. Collez le jeton dans le panneau **Dépôt GitHub**, puis touchez **Connecter**.

Le jeton est transmis directement par Safari à `api.github.com`. Il n’est pas
inclus dans le dépôt, dans les builds ou dans GitHub Pages. Forge iOS le conserve
dans `sessionStorage` : utilisez **Déconnecter** pour l’effacer immédiatement.
Pour un iPhone partagé, ne cochez aucune option de mémorisation proposée par un
gestionnaire de mots de passe.

## Compiler

- **Tester sans publier** déclenche manuellement `ios-release.yml`. Le job
  compile l’exemple choisi, mais ne crée pas de Release.
- **Compiler et publier** crée `swift-vX.Y.Z` ou `rn-vX.Y.Z` sur la branche
  choisie. Le tag déclenche le build, la Release et la mise à jour de la source
  AltStore.

Une version ne peut être publiée qu’une fois : si le tag existe déjà, augmentez
le numéro de version.

## Modifier le code

L’éditeur propose des raccourcis vers `ContentView.swift`, `App.tsx` et les deux
fichiers `.ios-release`. Il peut également ouvrir un autre fichier texte par son
chemin dans le dépôt. **Enregistrer sur GitHub** crée directement un commit sur
la branche sélectionnée.

La première version privilégie les modifications courtes sur iPhone. Elle ne
remplace pas encore un IDE complet : pas d’autocomplétion Swift/TypeScript, pas
de renommage multi-fichiers et pas de résolution visuelle des conflits.

## Séparation des responsabilités

| Fonction | Exécutée par |
| --- | --- |
| Interface, édition et commandes | PWA sur l’iPhone |
| Hébergement du code | Dépôt GitHub |
| Compilation avec Xcode | Runner macOS GitHub Actions |
| Hébergement de l’IPA | GitHub Releases |
| Catalogue des apps | GitHub Pages |
| Signature et installation | AltStore/SideStore sur l’iPhone |

Forge iOS résout le pilotage du développement sans ordinateur. Les restrictions
d’installation d’une IPA personnelle restent celles d’iOS : signature, mode
développeur, limite du compte gratuit et renouvellement périodique.
