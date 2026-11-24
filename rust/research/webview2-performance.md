# Wry / WebView2: démarrage, mémoire et limites

## Sources primaires consultées

- Dépôt et README Wry : <https://github.com/tauri-apps/wry>
- Exemple Wry `simple.rs` : <https://github.com/tauri-apps/wry/blob/dev/examples/simple.rs>
- API Wry `WebViewBuilderExtWindows` : <https://github.com/tauri-apps/wry/blob/dev/src/lib.rs>
- Conseils Microsoft sur les performances de WebView2 : <https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance>
- Différences entre Edge et WebView2 : <https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/browser-features>

## Constats

1. **Moteur sous Windows.** Wry utilise WebView2, basé sur Microsoft Edge Chromium. Wry exige une boucle d’événements active et une fenêtre native ; son README cite `tao` et `winit`, et son exemple de bureau associe `tao` à `wry`. Cela convient à une coque de navigateur Rust sans embarquer un second moteur Chromium complet. Sources : [Wry README](https://github.com/tauri-apps/wry), [exemple simple](https://github.com/tauri-apps/wry/blob/dev/examples/simple.rs).

2. **Un moteur partagé n’équivaut pas à un seul processus de rendu.** WebView2 fonctionne en processus multiples. Chaque contrôle WebView2 peut créer des processus navigateur, rendu et GPU ; la mémoire varie avec le contenu et le nombre d’instances. Microsoft recommande de réutiliser un `CoreWebView2Environment` pour les onglets afin de partager le pool et de limiter les contrôles redondants. Les pages complexes peuvent quand même consommer beaucoup de mémoire. Source : [Microsoft, performances et gestion mémoire](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance).

3. **Mesures de réduction mémoire.** Microsoft documente `MemoryUsageTargetLevel::Low` pour les WebViews inactives, ainsi que `TrySuspendAsync()` pour suspendre un renderer ; la suspension est au mieux et interrompt les scripts de la page. Réactiver une page restaure son activité. Retirer proprement les gestionnaires d’événements et disposer les contrôles est aussi nécessaire pour éviter les fuites. Ce sont des leviers, pas une garantie de plafond fixe. Source : [Microsoft, performances et gestion mémoire](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance).

4. **Démarrage.** L’initialisation froide du runtime et la navigation initiale ajoutent du délai. Microsoft recommande une UI initiale native légère, de ne créer WebView2 que pour le contenu réel, de conserver le dossier de données dans le dossier local par défaut, d’éviter les contrôles redondants et d’utiliser le runtime Evergreen pour recevoir les optimisations et correctifs récents. Quand Edge de version correspondante est déjà lancé, certaines bibliothèques WebView2 peuvent être en mémoire et accélérer le démarrage. Source : [Microsoft, optimisation du démarrage](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance).

5. **Différences avec un navigateur complet.** Dans WebView2, l’autoremplissage des mots de passe est désactivé par défaut mais configurable ; les extensions et les favoris Edge sont désactivés ; Microsoft indique que l’authentification Google est bloquée dans les WebViews intégrées. Il faudra construire l’interface des favoris et le coffre de mots de passe dans l’application et traiter explicitement les flux de connexion Google. Source : [Microsoft, fonctionnalités navigateur](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/browser-features).

## Conséquences pour le premier jalon

- Utiliser Tao pour la fenêtre et la boucle d’événements, Wry pour le contenu WebView2.
- Afficher la fenêtre et l’UI de départ nativement ; éviter une page d’accueil bâtie avec un framework web lourd.
- Réutiliser un environnement WebView2 pour les onglets, garder le nombre de WebViews actives bas et suspendre les onglets inactifs seulement si l’intégration le permet sans dégrader la navigation.
- Mesurer la mémoire du **processus hôte et de tous les processus WebView2 associés**, pas seulement celle du processus Rust. Le plafond de 500 Mo sera une cible sur un scénario spécifié, pas une garantie pour tout site.
- Le futur code qui appelle directement les API WebView2 doit vérifier les fonctionnalités exposées par la version Wry retenue. Wry expose une extension Windows pour fournir un environnement existant, mais la gestion du cycle de vie des onglets et des contrôleurs peut nécessiter les API COM Windows dédiées.

## Pièges sémantiques et limites

- « Plusieurs onglets partagent le même environnement » ne veut pas dire qu’ils partagent un seul renderer ni que leur mémoire est constante.
- Suspendre un onglet met en pause les scripts ; certains sites peuvent perdre des mises à jour ou des appels en arrière-plan. Réactivation et restauration doivent être caractérisées.
- WebView2 dépend du runtime installé sur Windows. La distribution et le premier démarrage doivent vérifier la présence du runtime et afficher une erreur compréhensible s’il manque.
- Un navigateur Wry/WebView2 n’est pas Chrome : les pages Web utilisent Chromium via Edge, mais les fonctions listées par Microsoft ne sont pas automatiquement fournies par le contrôle.
