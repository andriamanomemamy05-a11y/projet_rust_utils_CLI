//! # Commande `rm` personnalisée
//!
//! Ce module fournit une commande Rust qui supprime un **fichier** ou un **dossier**,
//! avec ou sans option récursive (`-r`), de manière similaire à la commande Unix `rm`.
//!
//! ## Fonctionnement général
//! - Si l'utilisateur indique un **fichier**, il est supprimé avec [`fs::remove_file`].  
//! - Si l'utilisateur indique un **dossier** :  
//!   - Sans `-r` → erreur, car [`fs::remove_file`] ne peut pas supprimer de dossier.  
//!   - Avec `-r` → le dossier (et tout son contenu) est supprimé via [`fs::remove_dir_all`].
//!
//! ## Utilisation en ligne de commande
//! ```bash
//! # Dans le menu interactif, supprimer un fichier
//! > rm exemple.txt
//!
//! # Supprimer un dossier entier (récursivement)
//! > rm -r mon_dossier
//!
//! # Supprimer un fichier par chemin absolu
//! > rm C:/Users/test.txt
//!
//! # Quitter le menu
//! > quit
//! ```
//!
//! ## Gestion des erreurs
//! - Si le chemin n'existe pas → message d'avertissement et possibilité de retaper.
//! - Si le dossier n'est pas supprimé sans `-r` → message d'erreur explicite et possibilité de retaper.
//! - Si la commande est mal utilisée → rappel de la syntaxe correcte et possibilité de retaper.

use std::{fs, path::Path, env};
use std::io::{self, Write};

/// Point d'entrée principal de la commande `rm`.
///
/// Cette fonction démarre un shell interactif permettant à l'utilisateur de saisir
/// des commandes `rm`. La boucle continue jusqu'à ce que l'utilisateur tape `quit`.
/// En cas d'erreur, l'utilisateur peut simplement retaper une nouvelle commande.
///
/// # Fonctionnement
/// 1. Affiche le menu interactif.
/// 2. Lit la commande de l'utilisateur.
/// 3. Parse la commande pour extraire les options et le chemin.
/// 4. Vérifie que la commande commence par `rm`.
/// 5. Effectue la suppression selon les options.
/// 6. En cas d'erreur, affiche un message et recommence.
///
/// # Exemple
///
/// ```no_run
/// rm();
/// // L'utilisateur entre : rm -r logs
/// // Affiche : 📁 Le dossier 'logs' a été supprimé avec succès.
/// ```
///
/// # Avertissement
///
/// Si vous omettez l'option `-r` pour un dossier, une erreur se produira
/// mais vous pourrez retaper la commande correctement.
///
/// ```text
/// ❌ Erreur : Impossible de supprimer un dossier sans l'option -r
/// Utilisez 'rm -r mon_dossier' pour supprimer ce dossier
/// ```
pub fn rm() {
    loop {
        println!("\n=== Programme utilitaire rm ===");
        println!("Entrez votre commande (ou 'quit' pour quitter) :");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Erreur lors de la lecture de l'entrée");

        let input = input.trim();

        // Si l'utilisateur tape quit, on sort du programme
        if input == "quit" {
            break;
        }

        // Ignorer les lignes vides
        if input.is_empty() {
            continue;
        }

        // Parser la commande
        let parts = parse_command_line(input);

        if parts.is_empty() {
            continue;
        }

        // Vérifier que la commande commence par rm
        if parts[0] != "rm" {
            println!("❌ Erreur : La commande doit commencer par 'rm'");
            println!("💡 Utilisez 'rm --help' pour plus d'informations");
            continue; // Permet de retaper la commande
        }

        // Traiter la commande
        let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
        process_command(&args);
        // Pas de gestion d'erreur ici, process_command affiche les messages
        // et on retourne automatiquement au début de la boucle
    }
}

/// Parse une ligne de commande en respectant les guillemets et échappements.
///
/// # Algorithme
/// - Parcours caractère par caractère.
/// - Bascule l'état `in_quotes` sur les guillemets.
/// - Si espace ou tabulation et hors guillemets, finalise le mot.
/// - Gère les échappements `\`.
/// - Ajoute le mot final à la liste.
///
/// # Arguments
/// * `input` - Ligne de commande brute.
///
/// # Retour
/// Vecteur de chaînes (`Vec<String>`), chaque élément un argument.
///
/// # Exemple
/// ```rust
/// let args = parse_command_line(r#"rm -r "dossier avec espaces""#);
/// assert_eq!(args, vec!["rm", "-r", "dossier avec espaces"]);
/// ```
fn parse_command_line(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            },
            ' ' | '\t' => {
                if in_quotes {
                    current.push(ch);
                } else if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            },
            '\\' => {
                // Gérer les échappements
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '"' || next_ch == '\\' {
                        current.push(chars.next().unwrap());
                    } else {
                        current.push(ch);
                    }
                } else {
                    current.push(ch);
                }
            },
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Traite une commande `rm` avec ses arguments.
///
/// Cette fonction ne retourne pas d'erreur mais affiche des messages appropriés.
/// L'utilisateur peut ensuite retaper une commande dans la boucle principale.
///
/// # Algorithme
/// 1. Gère `--help` si présent.
/// 2. Parse les arguments pour extraire l'option `-r` et le chemin cible.
/// 3. Vérifie qu'un chemin a été fourni (sinon affiche un message).
/// 4. Résout le chemin (gère `.` pour le dossier courant).
/// 5. Vérifie l'existence du chemin (sinon affiche un message).
/// 6. Effectue la suppression appropriée selon le type et l'option `-r`.
///
/// # Arguments
/// * `args` - Arguments de la commande (sans "rm").
///
/// # Exemple
/// ```no_run
/// process_command(&["-r", "mon_dossier"]);
/// // Affiche : 📁 Le dossier 'mon_dossier' a été supprimé avec succès.
/// ```
fn process_command(args: &[&str]) {
    // Gérer --help
    if args.contains(&"--help") {
        display_help();
        return;
    }

    // Parser les arguments pour extraire -r et le chemin
    let (recursive, target) = parse_arguments(args);

    // Si aucun chemin n'est fourni
    if target.is_none() {
        println!("⚠️  Vous devez préciser un nom de fichier ou de dossier à supprimer.");
        println!("💡 Utilisez 'rm --help' pour plus d'informations");
        return; // Retour à la boucle pour retaper
    }

    let target_name = target.unwrap();
    
    // Résoudre le chemin (gérer . pour le dossier courant)
    let file_path = match resolve_path(&target_name) {
        Some(path) => path,
        None => {
            println!("❌ Erreur : Impossible de récupérer le dossier courant");
            return; // Retour à la boucle pour retaper
        }
    };

    let path_obj = Path::new(&file_path);

    // Si le chemin n'existe pas
    if !path_obj.exists() {
        println!("⚠️  Le chemin '{}' n'existe pas.", file_path);
        println!("💡 Vérifiez le chemin et réessayez");
        return; // Retour à la boucle pour retaper
    }

    // Vérifier si c'est un dossier ou un fichier avant suppression
    let is_dir = path_obj.is_dir();

    // Si c'est un dossier et que -r n'est pas spécifié
    if is_dir && !recursive {
        println!("❌ Erreur : Impossible de supprimer un dossier sans l'option -r");
        println!("💡 Utilisez 'rm -r {}' pour supprimer ce dossier", target_name);
        return; // Retour à la boucle pour retaper
    }

    // Effectuer la suppression
    let result = if recursive && is_dir {
        fs::remove_dir_all(path_obj)
    } else {
        fs::remove_file(path_obj)
    };

    match result {
        Ok(_) => {
            // Si c'est une suppression de dossier
            if is_dir {
                println!("✅ Le dossier '{}' a été supprimé avec succès.", target_name);
            } else {
                // Sinon, on récupère le dossier parent, puis le fichier supprimé
                let parent = path_obj.parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("le dossier inconnu");
                println!("✅ Le fichier '{}' dans '{}' a été supprimé avec succès.", target_name, parent);
            }
        }
        Err(e) => {
            println!("❌ Erreur lors de la suppression : {}", e);
            println!("💡 Vérifiez les permissions et réessayez");
        }
    }
}

/// Parse les arguments pour extraire l'option `-r` et le chemin cible.
///
/// # Algorithme
/// - Parcourt tous les arguments.
/// - Si l'argument est `-r`, active le mode récursif.
/// - Sinon, considère l'argument comme le chemin cible.
/// - Seul le premier chemin trouvé est conservé.
///
/// # Arguments
/// * `args` - Slice des arguments.
///
/// # Retour
/// Tuple `(bool, Option<String>)` : (récursif, chemin_optionnel).
///
/// # Exemple
/// ```rust
/// let (recursive, path) = parse_arguments(&["-r", "mon_dossier"]);
/// assert_eq!(recursive, true);
/// assert_eq!(path.unwrap(), "mon_dossier");
/// ```
fn parse_arguments(args: &[&str]) -> (bool, Option<String>) {
    let mut recursive = false;
    let mut target: Option<String> = None;

    for arg in args {
        if *arg == "-r" {
            recursive = true;
        } else if target.is_none() {
            // Prendre le premier argument qui n'est pas -r comme chemin
            target = Some(arg.to_string());
        }
    }

    (recursive, target)
}

/// Résout un chemin en gérant les cas spéciaux comme `.` (dossier courant).
///
/// # Algorithme
/// - Si le chemin est `.`, retourne le dossier courant.
/// - Sinon, retourne le chemin tel quel.
///
/// # Arguments
/// * `path` - Chemin à résoudre.
///
/// # Retour
/// `Option<String>` contenant le chemin résolu, ou `None` en cas d'erreur.
///
/// # Exemple
/// ```rust
/// let resolved = resolve_path(".").unwrap();
/// // resolved contient le chemin absolu du dossier courant
/// ```
fn resolve_path(path: &str) -> Option<String> {
    if path == "." {
        // Récupérer le dossier courant
        env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        Some(path.to_string())
    }
}

/// Affiche l'aide complète du programme `rm`.
///
/// # Exemple
/// ```no_run
/// display_help();
/// ```
fn display_help() {
    println!("Usage: rm [OPTIONS] FICHIER...");
    println!();
    println!("Supprime des fichiers ou des dossiers.");
    println!();
    println!("Options:");
    println!("  -r                       Supprime les dossiers et leur contenu de manière récursive");
    println!("      --help               Affiche cette aide et quitte");
    println!();
    println!("Exemples:");
    println!("  rm fichier.txt           Supprime le fichier 'fichier.txt'");
    println!("  rm -r mon_dossier        Supprime le dossier 'mon_dossier' et tout son contenu");
    println!("  rm \"fichier avec espaces.txt\"  Supprime un fichier avec des espaces dans le nom");
    println!("  rm .                     Supprime le dossier courant (nécessite -r)");
    println!();
    println!("Attention:");
    println!("  ⚠️  La suppression est définitive et irréversible !");
    println!("  Utilisez cette commande avec précaution.");
}