//! # Commande `ls` personnalisée
//!
//! Ce module fournit une commande Rust qui liste et affiche tous les contenu d'un **dossier**
//!
//! ## Fonctionnement général
//! - L'utilisateur indique un **dossier** :  
//!   - avec la commande et le chemin absolu du dossier.
//!
//! ## Utilisation en ligne de commande
//! ```bash
//!
//! # Lister le contenu d'un dossier par chemin absolu
//! > ls "C:/Users/test.txt"
//!
//! # Quitter le menu
//! > quit
//! ```
//!
//! ## Gestion des erreurs
//! - Si le chemin n'existe pas → message d'avertissement et possibilité de retaper.
//! - Si la commande est mal utilisée → rappel de la syntaxe correcte et possibilité de retaper.

use std::{fs, path::Path, env};
use std::io::{self, Write};

/// Point d'entrée principal de la commande `ls`.
///
/// Cette fonction démarre un shell interactif permettant à l'utilisateur de saisir
/// des commandes `ls`. La boucle continue jusqu'à ce que l'utilisateur tape `quit`.
/// En cas d'erreur, l'utilisateur peut simplement retaper une nouvelle commande.
///
/// # Fonctionnement
/// 1. Affiche le menu interactif.
/// 2. Lit la commande de l'utilisateur.
/// 3. Parse la commande pour extraire les options et le chemin.
/// 4. Vérifie que la commande commence par `ls`.
/// 5. Effectue la liste selon les options.
/// 6. En cas d'erreur, affiche un message et recommence.
///
/// # Exemple
///
/// ```no_run
/// ls();
/// // L'utilisateur entre : ls "chemin\To\Logs"
/// // Lister tous les contenus du fichier Logs
/// ```
///
/// # Avertissement
///
/// ```text
/// ❌ Erreur : La commande doit commencer par 'ls'
/// ❌ Erreur : Chemin introuvable, veuillez vérifier le chemin.
/// ⚠️  Vous devez préciser un nom de dossier à lister.
/// Utilisez 'ls mon_dossier' pour lister
/// ```
pub fn ls() {
    loop {
        println!("\n=== Programme utilitaire ls ===");
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

        // Vérifier que la commande commence par ls
        if parts[0] != "ls" {
            println!("❌ Erreur : La commande doit commencer par 'ls'");
            println!("💡 Utilisez 'ls --help' pour plus d'informations");
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
/// let args = parse_command_line(r#"ls"dossier avec espaces""#);
/// assert_eq!(args, vec!["ls", "dossier avec espaces"]);
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

/// Traite la commande `ls` avec un chemin donné.
///
/// Cette fonction affiche le contenu d’un dossier sans générer d’erreur fatale.
/// Si une erreur survient (chemin manquant, inexistant, etc.), un message clair
/// est affiché pour informer l’utilisateur.
///
/// # Algorithme
/// 1. Gère l’option `--help` si elle est présente.
/// 2. Analyse les arguments pour extraire les options éventuelles et le chemin cible.
/// 3. Vérifie qu’un chemin a bien été fourni.
/// 4. Résout le chemin (par exemple, gère `.` pour le dossier courant).
/// 5. Vérifie l’existence du chemin.
/// 6. Si le chemin est valide et correspond à un dossier, affiche la liste de ses contenus.
///
/// # Arguments
/// * `args` – Les arguments passés à la commande (sans le mot-clé `ls`).
///
/// # Exemple
/// ```no_run
/// process_command(&["mon_dossier"]);
/// // ✅ Les contenus du dossier :
/// // - fichier1.txt
/// // - sous_dossier
/// // - ....
/// ```
fn process_command(args: &[&str]) {
    // Gérer --help
    if args.contains(&"--help") {
        display_help();
        return;
    }

    // Parser les arguments pour extraire le chemin
    let (recursive,target) = parse_arguments(args);

    // Si aucun chemin n'est fourni
    if target.is_none() {
        println!("⚠️  Vous devez préciser un nom de fichier ou de dossier à supprimer.");
        println!("💡 Utilisez 'ls --help' pour plus d'informations");
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

    match fs::read_dir(path_obj) {
        Ok(entries) => {
            println!("✅ Les contenus du dossier: ");
            for entry in entries.flatten() {
                println!(" - {}", entry.file_name().to_string_lossy());
            }
        }
        Err(e) => {
            println!("❌ Erreur lors de la suppression : {}", e);
            println!("💡 Vérifiez les permissions et réessayez");
        }
    }
}


/// Parse l'argument qui est  le chemin cible.
///
/// # Algorithme
/// - On considère l'argument comme le chemin cible.
/// - Seul le premier chemin trouvé est conservé.
///
/// # Arguments
/// * `args`.
///
/// # Retour
/// Retourne un tuple `(bool, Option<String>)` où : 
/// - L’`Option<String>` contient le chemin cible s’il est trouvé.
///
/// # Exemple
/// ```rust
/// let (_, path) = parse_arguments(&["dossier_test"]);
/// assert_eq!(path.unwrap(), "dossier_test");
/// ```
fn parse_arguments(args: &[&str]) -> (bool, Option<String>) {
    let mut target: Option<String> = None;
    let recursive = false;

    for arg in args {
        // Prendre le premier argument qui comme chemin
        target = Some(arg.to_string());
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


/// Affiche l'aide complète du programme `ls`.
///
/// # Exemple
/// ```no_run
/// display_help();
/// ```
fn display_help() {
    println!("Usage: ls FICHIER...");
    println!();
    println!("Lister tous les contenus d'un dossier avec un chemin spécifique.");
    println!();
    println!("Options:");
    println!("  .                       Afficher les contenus du dossier courant");
    println!("      --help               Affiche cette aide et quitte");
    println!();
    println!("Exemples:");
    println!("  ls \"fichier avec espaces.txt\"  Liste tous les fichiers ou dossiers avec des espaces dans le chemin");
    println!("  ls .                     Liste le contenu du dossier courant");
    println!();
    println!("Attention:");
    println!("  ⚠️  Attention avec le chemin et le dossier à lister !");
    println!("  Utilisez cette commande avec précaution.");
}