# Coffre de mots de passe Windows pour le navigateur Rust/Wry

## Recommandation

Pour un premier navigateur léger fondé sur Wry + WebView2, utiliser l’autosauvegarde et le remplissage intégrés à WebView2 est le chemin le plus simple et le plus sûr côté produit : pas de gestion maison des formulaires ni des identifiants. Activer `ICoreWebView2Settings4::put_IsPasswordAutosaveEnabled(TRUE)` sur le profil dédié de l’application. WebView2 gère les invites de sauvegarde/mise à jour et le remplissage; le réglage est partagé entre toutes les vues qui emploient le même profil. Par défaut, cette fonction est désactivée. [Microsoft: ICoreWebView2Settings](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2settings) [Microsoft: CoreWebView2Settings](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings?view=webview2-winrt-1.0.3230-prerelease)

Si l’application doit elle-même afficher, exporter/importer ou gérer chaque compte, préférer Windows Credential Manager via `CredWriteW` / `CredReadW` avec des credentials génériques, un `TargetName` stable et `CRED_PERSIST_LOCAL_MACHINE`. Cette persistance couvre les sessions successives du même utilisateur sur le même PC; elle n’expose pas le secret aux autres comptes Windows. Microsoft recommande Credential Manager avant DPAPI pour stocker des identifiants. [Handling Passwords](https://learn.microsoft.com/en-us/windows/win32/SecBP/handling-passwords) [CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw) [CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew)

DPAPI (`CryptProtectData` / `CryptUnprotectData`) est un choix secondaire pour chiffrer un fichier local personnalisé ou un ensemble de données structuré. Éviter `CRYPTPROTECT_LOCAL_MACHINE` pour les mots de passe: tout utilisateur de la machine pourrait déchiffrer les données. Libérer la sortie chiffrée avec `LocalFree`. [CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata)

## Accès depuis Wry

Wry ne semble pas fournir de méthode de configuration de l’autosauvegarde dans son API de haut niveau. Son extension Windows `WebViewExtWindows` retourne cependant l’interface COM `ICoreWebView2` avec `webview()`. Le code Rust peut ensuite récupérer les réglages et appeler la méthode d’interface WebView2, avec les bindings COM appropriés. L’interface `ICoreWebView2Settings4` ajoute le setter de l’autosauvegarde; son accès dépend du WebView2 Runtime/bindings disponibles. [Wry: `WebViewExtWindows`](https://github.com/tauri-apps/wry/blob/dev/src/lib.rs) [Microsoft: ICoreWebView2Settings4](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2settings4)

Wry expose aussi `with_general_autofill_enabled`, qui concerne le remplissage général, distinct des mots de passe. Le réglage password autosave doit donc être configuré explicitement par l’interface WebView2. [Code source Wry](https://github.com/tauri-apps/wry/blob/dev/src/lib.rs)

Pour le coffre natif, Rust peut appeler Win32 via le crate `windows` (bindings générés de Microsoft) ou via FFI. Utiliser `CRED_TYPE_GENERIC`; sérialiser dans le blob uniquement les champs nécessaires (par exemple identifiant + mot de passe), avec un format versionné et une clé de cible spécifique au domaine et au compte. `CredWriteW` remplace une entrée qui a le même type et la même cible. [windows-rs](https://github.com/microsoft/windows-rs) [CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew) [CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)

## Limites et risques

- Un coffre applicatif complet exige une interface pour voir, modifier et supprimer les identifiants, plus une confirmation avant remplissage. Faire l’injection via script dans chaque site expose davantage les secrets et casse sur les formulaires atypiques; préférer le gestionnaire intégré de WebView2 quand ses fonctions suffisent.
- L’autosauvegarde doit être vérifiée dans la version de Runtime WebView2 réellement installée et dans le profil réellement utilisé par Wry. Chaque instance utilisant le même profil partage ce réglage et ses données.
- Credential Manager est un stockage de secrets du système, pas une base de mots de passe navigateur. L’application reste responsable du choix des origines, des doublons, des suppressions et d’une UX de remplissage sûre. `CredReadW` opère sur le jeu d’identifiants associé à la session de connexion du jeton courant. [CredReadW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credreadw)
- `CREDENTIALW` limite la taille du blob générique; contrôler la limite documentée `CRED_MAX_CREDENTIAL_BLOB_SIZE` et gérer proprement les erreurs d’écriture. [CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)
- Les secrets en clair existent brièvement en mémoire pendant la saisie et le remplissage. Les effacer dès que possible, ne jamais les journaliser, et ne jamais transmettre les mots de passe sans HTTPS/TLS. [Handling Passwords](https://learn.microsoft.com/en-us/windows/win32/SecBP/handling-passwords)

## Plan d’implémentation conseillé

1. Dédier un profil WebView2 à l’application afin de garder ses identifiants isolés.
2. Après initialisation de Wry, obtenir `ICoreWebView2` via `WebViewExtWindows::webview`, consulter `ICoreWebView2Settings4`, puis activer l’autosauvegarde.
3. Garder l’autosauvegarde désactivée tant qu’une implémentation native distincte n’est pas prête, si la priorité est le contrôle strict de l’utilisateur.
4. Si un coffre maison devient nécessaire, commencer par Credential Manager générique; ne stocker que le strict nécessaire et ajouter lecture, suppression et confirmation de remplissage.
5. Traiter les mots de passe comme des secrets en mémoire et exclure les champs sensibles des logs, diagnostics et rapports de crash.
