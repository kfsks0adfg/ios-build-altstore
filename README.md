# iOS sans Mac — pack de build GitHub Actions

Ce dépôt modèle compile une app iOS **sans signature** sur un runner macOS GitHub,
publie l’IPA dans une GitHub Release, puis met à jour une source AltStore sur
GitHub Pages. AltStore ou SideStore signe ensuite l’IPA sur l’iPhone avec votre
compte Apple gratuit.

Deux projets exécutables servent de référence :

- `examples/swift/HelloSwift` : application SwiftUI native ;
- `examples/react-native/HelloReactNative` : application React Native 0.88.1.

## Ce que ce pack permet — et ne permet pas

- Aucun Mac personnel ni abonnement Apple Developer payant n’est nécessaire.
- Un runner macOS reste indispensable pour compiler avec le SDK iOS.
- Un compte Apple gratuit impose normalement une signature valable 7 jours et
  au plus 3 apps actives. AltStore/SideStore doit donc rafraîchir les apps.
- Le workflow produit volontairement une IPA non signée. Elle n’est pas
  installable directement et n’est pas destinée à l’App Store.
- Le dossier [`bootstrap-ios`](bootstrap-ios/README.md) contient désormais un
  prototype iOS 27 qui tente le jumelage, la signature et l’installation depuis
  iSH. Sa compilation est vérifiée ; le parcours complet attend encore une
  validation sur l’iPhone réel et ne doit pas être présenté comme stable.
- Un dépôt public rend le code source visible. Un dépôt privé consomme des
  minutes macOS facturées selon le multiplicateur GitHub en vigueur.

## Mise en route

1. Créez un dépôt GitHub public à partir de ce contenu et poussez les fichiers.
   Un dépôt public évite la facturation en minutes des runners GitHub hébergés.
2. Dans **Settings → Pages → Build and deployment**, choisissez **GitHub
   Actions** comme source. Ce réglage manuel est obligatoire avant le premier
   lancement avec le `GITHUB_TOKEN` standard.
3. Ouvrez **Releases → Draft a new release**. Créez le tag `swift-v1.0.0` pour
   l’exemple Swift ou `rn-v1.0.0` pour l’exemple React Native, puis publiez la
   release. Cette opération est faisable depuis Safari sur iPhone.
4. Suivez le workflow **iOS release** dans l’onglet Actions.
5. À la fin, ouvrez l’URL indiquée par le job `publish-source`, puis touchez
   **Ajouter à AltStore**. Pour SideStore, copiez l’URL `altstore.json` affichée
   par la page et ajoutez-la comme source.

## Adapter le pack à votre app

Le fichier `.ios-release/<app>.env` contient les quatre réglages essentiels :

```bash
APP_NAME=MonApp
SCHEME=MonApp
PROJECT_FILE=ios/MonApp.xcworkspace
APP_TYPE=react-native
```

`PROJECT_FILE` accepte un `.xcworkspace` ou un `.xcodeproj`; `APP_TYPE` accepte
`swift` ou `react-native`. Dans un dépôt à une seule app, gardez un seul fichier
de configuration et simplifiez l’étape `load-config` du workflow pour le charger.
Les options supplémentaires (`WORKING_DIRECTORY`, `ICON_PATH`, descriptions)
sont documentées dans [Ajouter un projet](docs/AJOUTER_UN_PROJET.md). Le guide
[Installation sans ordinateur](docs/INSTALLATION_SANS_ORDINATEUR.md) détaille
les solutions réellement possibles pour amorcer AltStore ou SideStore.

## Déclencheurs du dépôt modèle

| Tag | Projet compilé | Runner |
| --- | --- | --- |
| `swift-v*` | SwiftUI | `macos-26` |
| `rn-v*` | React Native | `macos-26` |

Le déclenchement manuel permet de tester un build sans créer de Release ni
modifier la source publiée.

## État de la validation

Le schéma JSON suit la documentation AltStore « Make a Source » consultée le
27 octobre 2026 : une app possède une liste `versions`, et chaque version reprend
les valeurs réelles `CFBundleShortVersionString` et `CFBundleVersion` de l’app.
La publication Pages n’écrit dans aucune branche : elle emploie les actions
officielles GitHub Pages et évite donc le conflit `git push HEAD:main`.

Les contrôles Linux locaux valident le JSON, les scripts, les fichiers de projet
et le JavaScript. Les deux compilations Xcode ont aussi été validées sur GitHub :
SwiftUI en 42 secondes et React Native en moins de 4 minutes. Les IPA, les deux
Releases et la source Pages ont ensuite été vérifiées publiquement. Voyez
[Vérifications](docs/VERIFICATIONS.md).
