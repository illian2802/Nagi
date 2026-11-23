# Nagi — navigateur Rust pour Windows

Nagi est un navigateur Windows construit avec Rust et WebView2. Il propose une interface à onglets, une page de nouvel onglet, la recherche, les favoris et une navigation légère.

Le rendu des sites est confié à WebView2, le moteur Chromium fourni par Microsoft Edge. Le projet n’embarque donc pas un second moteur Chromium complet. La faible consommation de mémoire est une cible à mesurer, pas une limite garantie pour chaque site.

## Installer Nagi

Lance `Nagi-Installer.exe`. C'est un fichier unique qui contient l'assistant et ses données ; il extrait les fichiers temporaires puis ouvre l'installation de Nagi. L'assistant crée un raccourci dans le menu Démarrer, propose un raccourci sur le bureau et installe le runtime WebView2 Microsoft s'il manque. L'installation est limitée à ton compte Windows. La désinstallation enlève l'application et ses raccourcis ; tes favoris, mots de passe et données de navigation restent dans ton profil Nagi.

`nagi-browser.exe` est aussi fourni pour lancer directement le navigateur sans installation.
## Fonctions

- Onglets, navigation précédent/suivant, actualisation et raccourcis clavier
- Page de nouvel onglet avec recherche, accès rapides et transitions animées discrètes
- Animations discrètes des panneaux, boutons et onglets, réduites automatiquement selon le réglage Windows
- Réglages persistants : thème sombre, moteur de recherche et blocage publicitaire
- Effacement de l’historique depuis le panneau Réglages
- Reprise de la dernière page au démarrage, désactivable dans les réglages
- Réouverture des dix derniers onglets fermés avec `Ctrl+Maj+T`
- Barre de progression pendant le chargement d’une page
- Favoris persistants, historique et gestion des téléchargements
- Téléchargements dans `%USERPROFILE%\Downloads\Nagi`
- Filtrage de base des domaines publicitaires connus
- Sauvegarde et remplissage des mots de passe via les invites WebView2

Raccourcis : `Ctrl+L` adresse, `Ctrl+T` nouvel onglet, `Ctrl+Maj+T` réouvrir un onglet, `Ctrl+W` fermer, `Ctrl+H` historique, `Ctrl+J` téléchargements, `Ctrl+D` favori, `Ctrl+,` réglages.

## Compilation et lancement

Depuis `rust/app` :

```powershell
cargo run --release
```

On peut ouvrir directement une adresse ou une recherche :

```powershell
cargo run --release -- https://www.wikipedia.org
cargo run --release -- "recettes de crêpes"
```

Les données de navigation sont conservées dans `%LOCALAPPDATA%\NagiBrowser\WebView2`. Favoris, historique, téléchargements et préférences sont dans `%LOCALAPPDATA%\NagiBrowser`.

Le binaire Windows compilé est `nagi-browser.exe` dans ce dossier ; tu peux le lancer directement. La fenêtre affiche le logo Nagi et le binaire release ne fait pas apparaître de fenêtre PowerShell.

## Blocage des publicités

Nagi bloque les requêtes vers une liste intégrée de domaines publicitaires et de suivi connus. Cette première liste ne remplace pas une extension de filtrage complète : les publicités servies depuis le même domaine que la vidéo ou la page (par exemple certaines annonces YouTube) peuvent encore passer.

## Mémoire

La mesure doit inclure le processus Nagi et tous les processus `msedgewebview2.exe` associés. Les sites Web contrôlent une grande partie de leur propre consommation mémoire ; un site complexe peut dépasser la cible de 500 Mo.

Depuis ce dossier, exécute `.\mesure-memoire.ps1`. Si Nagi est déjà ouvert sur la vidéo, le script mesure cette instance après 20 secondes ; sinon, il ouvre trois pages légères. Il additionne le jeu de travail de l’hôte et des processus descendants WebView2. Il laisse le navigateur ouvert après le relevé.

## État du périmètre

La sauvegarde et le remplissage des mots de passe utilisent les invites intégrées à WebView2. Le bloqueur utilise une liste intégrée de domaines et ne peut pas supprimer toutes les annonces, notamment celles hébergées sur le même domaine que la vidéo ou la page.





## Langues

L’installateur propose 14 langues : français, anglais, espagnol, allemand, italien, portugais brésilien, néerlandais, polonais, turc, russe, japonais, chinois simplifié, chinois traditionnel et arabe. L’anglais est sélectionné par défaut ; tu peux choisir une autre langue pendant l’installation. Au premier lancement, Nagi reprend ce choix. Tu peux ensuite modifier la langue dans **Réglages**. L’arabe utilise une interface de droite à gauche.

## Mises à jour GitHub

Les builds créés par le workflow GitHub incluent automatiquement le dépôt qui les a compilés. Au démarrage, Nagi recherche une release stable plus récente. Si elle existe, le panneau Réglages s’ouvre avec un bouton pour télécharger l’installateur officiel de la release. Le téléchargement s’ouvre dans le navigateur par défaut ; l’utilisateur lance ensuite l’installateur pour appliquer la mise à jour. Nagi ne s’installe pas silencieusement.

Le dépôt GitHub doit être public pour que Nagi puisse consulter ses releases sans jeton secret embarqué dans l’application. Les exécutables compilés localement sans `NAGI_GITHUB_REPOSITORY` ne connaissent aucun dépôt et désactivent cette vérification.

