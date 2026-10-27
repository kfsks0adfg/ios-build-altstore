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

## À vérifier par un premier lancement GitHub

- La compilation réelle des deux exemples sous la version de Xcode actuellement
  installée sur `macos-26`.
- La version CocoaPods fournie par l’image et sa compatibilité avec React Native
  0.88.1.
- L’activation initiale de Pages doit être faite manuellement via
  **Settings → Pages → GitHub Actions** avant le premier lancement.
- L’acceptation de l’icône SVG par la version d’AltStore/SideStore utilisée.
  Remplacez-la par un PNG si nécessaire.
- L’installation et le rafraîchissement, qui dépendent des versions actuelles
  d’AltStore/SideStore et des services Apple.

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

