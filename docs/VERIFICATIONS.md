# Vérifications et limites connues

## Vérifié dans ce pack

- Le [schéma officiel AltStore](https://faq.altstore.io/developers/make-a-source)
  consulté le 27 octobre 2026 utilise `apps[].versions`.
- Les champs racine documentés n’incluent pas `identifier`.
- `altstore://source?url=…` est le lien utilisé dans la
  [documentation des sources de confiance](https://faq.altstore.io/altstore-classic/trusted-sources)
  AltStore Classic.
- Le runner `macos-26` est listé par
  [`actions/runner-images`](https://github.com/actions/runner-images); `macos-14`
  y est marqué déprécié à cette date.
- Le workflow ne pousse ni sur `main` ni sur `gh-pages`. La publication emploie
  `configure-pages`, `upload-pages-artifact` et `deploy-pages`.
- `scripts/check_pack.sh` contrôle les scripts, le JSON et le JavaScript sur Linux.
- L’exemple SwiftUI a été compilé, empaqueté et publié avec succès sur
  `macos-26` le 27 octobre 2026.
- L’exemple React Native 0.88.1 a passé `npm ci`, Bundler, CocoaPods, Xcode et
  l’empaquetage IPA sur le même runner.
- Les deux IPA publiques commencent par une archive ZIP valide, et la source
  publiée contenant les deux applications passe le validateur du dépôt.
- Le second workflow Pages, exécuté depuis `main`, a été déclenché automatiquement
  après la publication React Native et a déployé la source avec succès.

## Erreurs rencontrées et corrigées

- Les anciennes versions des actions GitHub utilisaient Node 20. Elles ont été
  remplacées par leurs versions majeures courantes utilisant Node 24.
- Le premier déploiement Pages depuis le tag `swift-v1.0.0` a été refusé par la
  protection de l’environnement `github-pages`, limitée à `main`. Le workflow a
  été séparé : le build sur tag déclenche désormais un workflow Pages sur `main`,
  sans élargir la règle de protection.

## À vérifier sur l’iPhone

- L’acceptation de l’icône SVG par la version d’AltStore/SideStore utilisée.
  Remplacez-la par un PNG si nécessaire.
- L’installation et le rafraîchissement, qui dépendent des versions actuelles
  d’AltStore/SideStore et des services Apple.
- Pour le prototype iSH iOS 27 : la visibilité de l’annonce Bonjour dans Mode
  développeur, la continuité du processus en arrière-plan, l’ouverture du tunnel
  TLS-PSK et l’installation par `installation_proxy`.

## Prototype iSH vérifié hors appareil

- Les trois exécutables Rust (jumelage, signature, installation) compilent et
  passent `cargo test --all-targets` sur Linux.
- Le jumelage utilise l’API RPPairing initiée par l’appareil introduite dans
  `idevice` 0.1.68, et non un profil UDID ou une installation OTA.
- SideStore est téléchargé depuis sa release officielle et contrôlé par le
  SHA-256 publié avec l’asset.
- Le mot de passe Apple est demandé sans écho et ne transite pas par GitHub.
- La validation i686-musl par GitHub Actions reste nécessaire avant de publier
  la première release du prototype.

## Diagnostic d’un build

1. Ouvrez le job `build` et repérez la première étape rouge.
2. Si `pod install` échoue, utilisez la version de Ruby/CocoaPods recommandée par
   votre version React Native (souvent via le `Gemfile` du template officiel).
3. Si le scheme est introuvable, rendez-le partagé dans Xcode et commitez le
   fichier `.xcscheme`.
4. Si plusieurs `.app` sont produites, rendez la recherche plus précise dans
   l’étape `Lire les métadonnées`.
5. Si AltStore refuse l’IPA pour permissions non concordantes, complétez les
   `--entitlement` et vérifiez les clés `NS…UsageDescription`.

## Limites du compte Apple gratuit

Les règles Apple peuvent évoluer. Au moment de la conception du pack, le cas
usuel reste une validité de 7 jours et 3 apps actives. La configuration et le
rafraîchissement de SideStore/AltStore doivent être revérifiés dans leurs
documentations respectives au moment de l’installation.

