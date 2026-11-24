# Reprise

- Objectif : navigateur Windows Rust rapide, faible consommation de mémoire, sauvegarde des mots de passe à venir.
- Projet : `rust/app`.
- Architecture : Tao + Wry/WebView2 avec un `WebContext` partagé.
- Référence mémoire : trois pages simples ; compter l’hôte et tous ses processus descendants. La capture de Gestionnaire des tâches fournie montre 0,6 Mo pour `nagi-browser.exe` et 468,1 Mo pour Chrome, mais elle ne suffit pas à comparer les groupes de processus.
- État : `cargo check` passe. Le premier exécutable release est verrouillé par une fenêtre déjà lancée ; une reconstruction séparée est en cours dans `work/nagi-browser-target`.
- Navigation : l’utilisateur a indiqué avoir ouvert la même vidéo que dans Chrome. Le binaire livré inclut aussi un bouton « Aller » explicite et l’envoi par Entrée.
- À vérifier : relever la mémoire totale avec `mesure-memoire.ps1` ; comparer démarrage à froid et à chaud.
- Mots de passe : autosauvegarde WebView2 activée via `ICoreWebView2Settings4`; vérifier le dialogue de sauvegarde puis le remplissage après redémarrage.
- Non implémenté : favoris, historique, téléchargements, navigation privée.
- Skill suivant : `rust-anything` n’a pas de code source à migrer ici ; poursuivre comme projet Rust neuf.
