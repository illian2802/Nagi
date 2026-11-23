# GitHub, releases et site web Nagi

Rien dans ce projet ne publie automatiquement vers GitHub ou Vercel. La publication reste une action manuelle du propriétaire du compte.

## 1. Créer le dépôt GitHub

Crée un dépôt GitHub **public** (Nagi ne contient aucun jeton d’accès ; les mises à jour ne peuvent donc pas lire les releases d’un dépôt privé). Ajoute les fichiers du projet à la racine du dépôt, puis pousse-les sur `main`. Le `.gitignore` exclut les caches de compilation et les exécutables locaux.

Le workflow `.github/workflows/release.yml` compile Nagi et intègre automatiquement le nom réel de ce dépôt dans le navigateur construit par GitHub Actions.

## 2. Relier le site au dépôt

Dans `site/config.js`, remplace la valeur vide avec le propriétaire et le nom exacts du dépôt :

```js
window.NAGI_SITE_CONFIG = { githubRepository: "mon-compte/nagi-browser" };
```

Importe ensuite le dépôt dans Vercel et choisis `site` comme **Root Directory**. Le site est statique et n’a besoin ni de base de données ni de clé secrète. Avant la première release, il garde son installateur local de secours ; ensuite il pointe vers la release stable la plus récente.

## 3. Créer la première release

Après avoir poussé le workflow et le code, crée la première release stable depuis un tag correspondant à la version de `rust/app/Cargo.toml`, actuellement `v0.3.3` :

```powershell
git tag v0.3.3
git push origin v0.3.3
```

GitHub Actions compile alors l’application avec `owner/repository`, fabrique `Nagi-Installer.exe` et crée la GitHub Release. Installe cette version compilée par GitHub pour activer la vérification des mises à jour. Un exécutable local plus ancien, compilé sans `NAGI_GITHUB_REPOSITORY`, ne peut pas retrouver le dépôt tout seul.

## 4. Publier une mise à jour

Modifie la version dans `rust/app/Cargo.toml`, puis pousse le changement sur `main`. Crée et pousse ensuite le tag stable correspondant, par exemple :

```powershell
git tag v0.3.4
git push origin main
git push origin v0.3.4
```

Le workflow fabrique un installateur autonome et crée la release. Au lancement de Nagi, le navigateur vérifie la dernière release stable. Si une version plus récente existe, il ouvre Réglages et affiche le bouton de téléchargement. Celui-ci ouvre l’installateur GitHub dans le navigateur par défaut ; l’utilisateur lance l’installation pour appliquer la mise à jour. Nagi ne télécharge ni n’installe de mise à jour en silence.

La création du dépôt, l’envoi du code, la première release et le déploiement Vercel doivent être effectués depuis les comptes GitHub/Vercel. Ce projet ne les exécute pas à ta place.
