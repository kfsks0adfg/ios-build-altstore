# Bootstrap iOS sans ordinateur — prototype iSH

Ce dossier contient notre propre chaîne d’amorçage pour iOS 27 : elle ne passe
ni par AltServer, ni par une installation OTA, ni par un certificat partagé.
Elle exécute trois opérations dans iSH :

1. l’iPhone se jumelle avec un hôte RPPairing annoncé localement par iSH ;
2. l’IPA est signée avec le compte Apple gratuit, sans sortir le mot de passe de
   l’iPhone ;
3. l’IPA est envoyée à `installation_proxy` au travers de LocalDevVPN.

## État exact

- **Vérifié hors iPhone** : compilation Rust native, intégration des protocoles
  RPPairing/TLS-PSK/RSD, signature avec `isideload` et scripts shell.
- **À vérifier en CI** : compilation statique i686-musl réellement exécutable
  dans iSH.
- **À vérifier sur l’iPhone cible** : diffusion Bonjour depuis le réseau virtuel
  d’iSH, survie du processus pendant le passage dans Réglages, ouverture du port
  de tunnel par iOS 27.2 et installation réelle.
- **Aucun contournement annoncé comme acquis** : si l’un de ces quatre contrôles
  échoue, la sortie de l’outil indique précisément le mur rencontré.

L’installation OTA par `itms-services` a été retirée : iOS refuse ce canal pour
un profil de développement gratuit. SideInstaller n’est ni copié ni re-signé,
sa licence interdisant la redistribution et la re-signature de ses builds.

## Prérequis sur l’iPhone

1. Installer [iSH](https://apps.apple.com/app/ish-shell/id1436902243).
2. Installer [LocalDevVPN](https://apps.apple.com/app/localdevvpn/id6755608044).
3. Activer **Réglages → Confidentialité et sécurité → Mode développeur**.
4. Rejoindre un réseau Wi-Fi. Le jumelage initial utilise le réseau local.

Si **Mode développeur** n’apparaît pas, cette version du prototype ne peut pas
encore amorcer le téléphone : le menu de jumelage iOS 27 se trouve à cet endroit.

## Installation du prototype

Dans iSH :

```sh
wget https://raw.githubusercontent.com/kfsks0adfg/ios-build-altstore/bootstrap-v0.1.0/bootstrap-ios/install.sh
less install.sh
sh install.sh
```

### 1. Jumeler

```sh
~/.ios-bootstrap/bootstrap.sh pair
```

L’outil affiche les gestes à faire. Passez rapidement dans **Réglages →
Confidentialité et sécurité → Mode développeur**, touchez **Jumeler avec iSH iOS
Bootstrap**, revenez lire le code dans iSH, puis saisissez-le dans Réglages.

### 2. Signer SideStore

```sh
~/.ios-bootstrap/bootstrap.sh sign
```

Le script télécharge SideStore 0.7.0-alpha depuis sa release officielle et
vérifie son SHA-256. L’adresse Apple, le mot de passe et le code 2FA sont saisis
localement. Un autre IPA peut être fourni à la place :

```sh
~/.ios-bootstrap/bootstrap.sh sign /chemin/MonApp.ipa
```

### 3. Installer directement

Connecter LocalDevVPN, garder iSH au premier plan, puis lancer :

```sh
~/.ios-bootstrap/bootstrap.sh install
```

Après l’installation, importer dans SideStore le fichier
`SideStore.mobiledevicepairing`, visible dans **Fichiers → iSH**. Lors de la
première connexion Apple, SideStore peut demander **Resign SideStore** : accepter
ce renouvellement lui permet de prendre possession de son propre certificat.

## Sécurité

- Le mot de passe Apple est lu sans écho et n’est jamais un argument, un secret
  GitHub ou une valeur écrite sur disque.
- La clé de signature et le jumelage restent sous `~/.ios-bootstrap`, avec des
  permissions privées.
- Les IPA téléchargées sont épinglées par SHA-256.
- Le signeur demande une confirmation avant toute révocation de certificat.
- Aucun profil racine, MDM ou certificat d’entreprise partagé n’est installé.

Utiliser de préférence un compte Apple distinct du compte iCloud principal.

## Commandes et diagnostic

```sh
~/.ios-bootstrap/bootstrap.sh status
```

Les erreurs sont séparées par étape : publication Bonjour, jumelage RPPairing,
vérification du fichier, port TLS-PSK, dialogue RSD ou `installation_proxy`.

## Limites inchangées

- signature gratuite valable sept jours ;
- trois applications actives au maximum ;
- dix App IDs créés par période de sept jours ;
- dépendance aux API Apple privées et à un serveur Anisette ;
- prototype iOS 27 uniquement tant que le jumelage autonome n’est pas validé.
