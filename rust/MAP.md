# Plan du navigateur

Objectif : créer un navigateur Windows léger en Rust, avec WebView2, une cible de démarrage rapide et une cible de mémoire inférieure à 500 Mo dans un scénario de référence.

Le dossier de départ ne contenait aucun code source. `rust-anything analyze --whole-app` a donc trouvé zéro fichier source et zéro unité à migrer. Ce projet est créé directement en Rust.

## Unités

| Unité | État | Périmètre |
|---|---|---|
| shell | en cours de vérification | fenêtre, barre d’adresse/recherche, navigation, onglets; l’utilisateur a chargé une vidéo, vérification via bouton « Aller » incluse dans le binaire courant |
| profile | ouvert | profil persistant et isolation des données |
| passwords | implémenté, à vérifier | invites WebView2 de sauvegarde et remplissage sur le profil Nagi |
| bookmarks-history | ouvert | favoris et historique |
| downloads-settings | ouvert | téléchargements et préférences |
| memory-startup | ouvert | mesure de tous les processus, démarrage froid et chaud, gestion des onglets inactifs |

## Prochain jalon

Valider la sauvegarde et le remplissage intégrés WebView2 dans un formulaire réel ; relever la mémoire du groupe complet dans le scénario de vidéo. Ensuite, poursuivre avec favoris/historique.
