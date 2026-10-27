# Ajouter un nouveau projet

## 1. Copier le pack

Copiez dans votre dépôt :

- `.github/workflows/ios-release.yml` ;
- `scripts/` ;
- `docs/` ;
- un fichier `.ios-release/mon-app.env`.

Dans un dépôt ne contenant qu’une app, remplacez la sélection `swift-v*` / `rn-v*`
de l’étape `Charger la configuration de l’app` par :

```bash
config_file=.ios-release/mon-app.env
```

Remplacez également les motifs de tags du workflow par `v*`.

## 2. Renseigner les quatre valeurs obligatoires

```bash
APP_NAME=MonApp
SCHEME=MonApp
PROJECT_FILE=ios/MonApp.xcworkspace
APP_TYPE=react-native
```

- `APP_NAME` sert au nom de l’IPA. Évitez les espaces.
- `SCHEME` doit être **partagé** dans Xcode et commité sous
  `xcshareddata/xcschemes/`.
- `PROJECT_FILE` pointe vers le workspace CocoaPods pour React Native, ou vers
  le workspace/projet Xcode d’une app Swift.
- `APP_TYPE` vaut `swift` ou `react-native`.

## 3. Régler les options du dépôt

Ajoutez au même fichier :

```bash
WORKING_DIRECTORY=.
ICON_PATH=assets/app-icon.svg
DEVELOPER_NAME=Mon_Nom
APP_SUBTITLE=Mon_app_personnelle
APP_DESCRIPTION=Description_de_mon_application.
```

Les underscores des trois textes sont convertis en espaces dans la source. Le
fichier `ICON_PATH` est relatif à `docs/`. Utilisez de préférence une icône PNG
carrée pour une compatibilité maximale ; les SVG des exemples sont surtout des
placeholders faciles à remplacer.

Pour une app utilisant des capabilities Apple, ajoutez chaque entitlement au
script `update_args` avec `--entitlement com.apple.…`. Les descriptions de
confidentialité `NS…UsageDescription` sont extraites automatiquement du
`Info.plist` compilé. AltStore compare ces permissions à l’IPA et refusera une
fiche incomplète.

## 4. React Native

Commitez obligatoirement :

- `ios/` et le `.xcodeproj` ;
- `Podfile` ;
- le `Gemfile` du template React Native quand il existe ;
- `package.json` et `package-lock.json`.

Le modèle actuel utilise Node 24 et React Native 0.88.1. Pour une autre version,
adaptez `node-version` dans le workflow. Quand un `Gemfile` est présent, le
workflow utilise `bundle exec pod install`; sinon il appelle directement
`pod install`. Cette commande crée le `.xcworkspace` sur le runner : le
workspace n’a pas besoin d’être commité.

## 5. Swift natif

Les dépendances Swift Package Manager sont résolues par `xcodebuild` avant la
compilation. Commitez `Package.resolved` quand votre projet en possède un afin
d’obtenir des builds reproductibles.

## 6. Publier

1. Activez Pages avec la source **GitHub Actions**.
2. Créez un tag `v1.0.0` depuis une GitHub Release.
3. Attendez les jobs `build` puis `publish-source`.
4. Ajoutez l’URL `https://UTILISATEUR.github.io/DEPOT/altstore.json` dans
   AltStore/SideStore.

La version AltStore ne vient pas du tag : elle vient de
`CFBundleShortVersionString`, et le build de `CFBundleVersion`. Il faut donc
mettre ces deux valeurs à jour dans le projet avant de créer un nouveau tag.

