# Composants tiers

Le binaire utilise les dépendances déclarées dans `signer/Cargo.toml`, notamment :

- `isideload` — Nicholas Sharp et contributeurs — licence MIT —
  <https://github.com/nab138/isideload> ;
- `idevice` — Jackson Coxson et contributeurs — licence MIT —
  <https://github.com/jkcoxson/idevice> ;
- `mdns-sd` — licence MIT/Apache-2.0 —
  <https://github.com/keepsimple1/mdns-sd>.

Le script télécharge SideStore depuis sa release GitHub officielle et le signe
localement à la demande de l’utilisateur. Il ne réhéberge aucun binaire tiers.
