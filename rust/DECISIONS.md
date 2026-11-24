# Décisions

## ADR-001 : cible Windows

Statut : acceptée (2026-11-21)

Contexte : le navigateur est demandé dans un environnement Windows et la priorité est une faible empreinte mémoire.

Décision : commencer par une application de bureau Windows écrite en Rust.

Conséquences : les premières versions utilisent les composants de navigation et de sécurité Windows ; le portage multiplateforme n’est pas inclus dans le premier jalon.

## ADR-002 : moteur WebView2

Statut : acceptée (2026-11-21)

Contexte : WebView2 utilise le moteur Chromium d’Edge et permet d’éviter d’embarquer un moteur complet dans l’application.

Décision : utiliser Wry avec WebView2 sur Windows, partager le contexte entre les onglets et réduire les contrôles actifs. Un WebView inactif passe au niveau mémoire `Low` ; l’onglet actif revient à `Normal`.

Conséquences : la mémoire dépend des sites et des onglets. Le réglage `Low` est une demande d’optimisation au moteur et dépend d’un runtime WebView2 suffisamment récent ; il ne garantit pas une limite fixe. Certaines fonctions de Chrome, dont les extensions et les connexions Google intégrées, diffèrent ou sont absentes.

## ADR-003 : cible mémoire mesurée

Statut : acceptée (2026-11-21)

Contexte : l’objectif est d’utiliser le moins de RAM possible, avec une cible déclarée sous 500 Mo.

Décision : compter l’ensemble du processus hôte et des processus WebView2 associés, puis mesurer un scénario de quelques onglets légers. Le seuil n’est pas garanti pour tous les sites.

Conséquences : les résultats devront préciser le nombre d’onglets, les pages utilisées, la durée après lancement et la métrique Windows (mémoire de travail privée ou jeu de travail).

## ADR-004 : mots de passe

Statut : supersédée par ADR-005 (2026-11-21)

Contexte : les mots de passe doivent rester disponibles entre les sessions.

Décision : le futur coffre devra s’appuyer sur le stockage protégé par le compte Windows ; les secrets ne seront pas enregistrés en clair.

Conséquences : le coffre et le remplissage automatique sont hors du premier jalon de navigation et devront avoir leur propre caractérisation de sécurité.

## ADR-005 : utiliser les invites intégrées WebView2 pour les mots de passe

Statut : acceptée (2026-11-21), supersède ADR-004

Contexte : l’autosauvegarde WebView2 est désactivée par défaut ; quand elle est activée, WebView2 fournit ses propres invites de sauvegarde/mise à jour et le remplissage ultérieur. Wry expose l’interface COM WebView2 sur Windows.

Décision : activer `ICoreWebView2Settings4::SetIsPasswordAutosaveEnabled(true)` dans chaque WebView du profil Nagi, et laisser WebView2 gérer les formulaires et l’invite de sauvegarde. Ne pas copier les secrets vers un coffre applicatif maison.

Conséquences : les identifiants restent dans le profil WebView2 dédié de Nagi. Le flux doit être vérifié avec le runtime installé ; certains fournisseurs d’authentification bloquent les WebViews intégrées.
