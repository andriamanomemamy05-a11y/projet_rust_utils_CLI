#Inial commit
Lien du projet public : https://github.com/andriamanomemamy05-a11y/projet_rust_utils_CLI.git

# Projet Rust Utils CLI

L’objectif de ce projet est de créer un utilitaire en ligne de commande capable de reproduire le comportement de plusieurs commandes Linux courantes (telles que ls, cat, cp, mv, rm, wc et head).

Ce travail permet de renforcer la maîtrise du langage Rust en manipulant ses concepts fondamentaux tels que la gestion de la mémoire, le système de modules, les entrées/sorties, et la gestion des erreurs.

Il constitue également une mise en pratique des bonnes pratiques de programmation système, tout en illustrant la capacité à produire une documentation technique complète et structurée en utilisant l’outil rustdoc.

# Documentation du projet

La documentation générée avec rustdoc se trouve dans le répertoire target/doc/

Pour la consulter, il suffit d’ouvrir le fichier index.html situé dans le répertoire target/doc/ avec un navigateur web.

# Lancement du Projet

Pour lancer le projet, exécutez :

```bash
cargo run
```

Vous obtiendrez alors le menu suivant :

Bonjour et bienvenue dans l'utilitaire de commande linux.

Veuillez choisir votre utilitaire (tapez quit pour quitter) :

1. ls
2. cat
3. cp
4. mv
5. rm
6. wc
7. head

Votre choix :

En fonction de votre choix compris entre 1 à 7, l’utilitaire exécutera la commande correspondante.
Toutefois, certaines commandes possèdent des spécificités d'utilisation. Vous trouverez leur mode d’emploi ci-dessous.

# Commandes `cp`, `mv` et `head`

Implémente des versions simplifiées des commandes Linux `cp`, `mv` et `head`.
Des fichiers et répertoires de tests sont fournis

## Lancement

Se placer dans le répertoire `target/test_cp_mv_head` puis exécuter :

```bash
cargo run
```

Choisir ensuite la fonctionnalité souhaitée dans le menu.

---

## Commande `cp`

### Copier un fichier

```bash
fichier_source destination
```

Exemple :

```bash
test_cp.txt cp/
```

### Copier avec mode verbeux `-v`

Affiche chaque fichier copié.

```bash
-v fichier_source destination
```

Exemple :

```bash
-v test_cp_v.txt cp_v/
```

### Copier avec confirmation `-i`

Demande confirmation avant d'écraser un fichier existant.
("y" pour Oui, "n" pour Non)

```bash
-i fichier_source destination
```

Exemple :

```bash
-i test_cp_i.txt cp_i/
```

---

## Commande `mv`

### Déplacer un fichier

```bash
fichier_source destination
```

Exemple :

```bash
test_mv.txt mv/
```

### Déplacer avec mode verbeux `-v`

```bash
-v fichier_source destination
```

Exemple :

```bash
-v test_mv_v.txt mv_v/
```

### Déplacer avec confirmation `-i`

("y" pour Oui, "n" pour Non)

```bash
-i fichier_source destination
```

Exemple :

```bash
-i test_mv_i.txt mv_i/
```

### Renommer un fichier

```bash
fichier_source nouveau_nom
```

Exemple :

```bash
test_mv_for_rename.txt toto
```

---

## Commande `head`

### Afficher les 10 premières lignes

```bash
fichier_source
```

Exemple :

```bash
test_head.txt
```

### Mode verbeux `-v`

Affiche le nom du fichier avant le contenu.

```bash
-v fichier_source
```

Exemple :

```bash
-v test_head.txt
```

### Afficher un nombre précis de lignes `-n`

```bash
-n nombre fichier_source
```

Exemple :

```bash
-n 5 test_head.txt
```
