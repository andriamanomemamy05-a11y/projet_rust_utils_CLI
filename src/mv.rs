//! # Module `mv`
//!
//! Ce module implémente la commande Unix **`mv`** en Rust.
//!
//! Il permet de **déplacer ou renommer** un fichier ou un dossier, avec la prise en charge
//! des options suivantes :
//!
//! - `-i` : demande confirmation avant d’écraser une destination existante (*interactive*).  
//! - `-v` : affiche le nom des fichiers déplacés ou renommés (*verbose*).

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// # Fonction : `move_file`
/// 
/// Déplace ou renomme un fichier ou dossier, en reproduisant le comportement de la commande Unix **`mv`**.
///
/// # Fonctionnement
/// - Vérifie si la source existe.
/// - Détermine si la destination est un dossier ou un fichier.
/// - Si la destination existe déjà :
///   - et que le flag `-i` est activé, demande confirmation avant d’écraser.
/// - Supprime la destination si nécessaire.
/// - Déplace ou renomme la source vers la destination.
/// - Si le flag `-v` est activé, affiche le déplacement effectué.
///
/// # Flags pris en charge
/// - `-i` : interactive → demande confirmation avant d’écraser un fichier existant.
/// - `-v` : verbose → affiche les fichiers déplacés ou renommés.
fn move_file(flag: Option<&str>, source: &str, destination: &str) {
    
    // Vérifie si le fichier source existe
    let source_path = Path::new(source);
    if !source_path.exists() {
        eprintln!("mv: cannot stat '{source}': No such file or directory");
        return;
    }

    
    //   Vérifie si la destination est un fichier ou un répertoire :
    //   - Si c’est un répertoire, on ajoute le nom du fichier source à la fin.
    //   - Sinon, on considère que la destination est un fichier et on garde son nom tel quel.
    let final_destination = if Path::new(destination).is_dir() {
        let name = source_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        format!("{}/{}", destination.trim_end_matches('/').trim_end_matches('\\'), name)
    } else {
        destination.to_string()
    };

    
    let final_dest_path = Path::new(&final_destination);

    
    
    //    Vérifie si la destination existe déjà :
    //    - Si oui, et que l’utilisateur a passé le flag -i (interactive),
    //      on lui demande s’il veut écraser le fichier ou dossier existant.
    //    - Si l’utilisateur tape 'y', le programme continue et remplacera
    //      la destination plus tard lors du déplacement final.
    //    - Sinon, l’opération est annulée immédiatement.
    if final_dest_path.exists() {
        if let Some(f) = flag {
            if f == "-i" {
                print!("mv: overwrite '{final_destination}'? ");
                io::stdout().flush().unwrap();
                let mut answer = String::new();
                io::stdin().read_line(&mut answer).unwrap();

                if !answer.trim().eq_ignore_ascii_case("y") {
                    println!("mv: not overwritten.");
                    return;
                }
            }
        }
        
        
        //    Si la destination existe et doit être écrasée :
        //    - Si c’est un dossier, on le supprime récursivement.
        //    - Si c’est un fichier, on le supprime directement.
        if final_dest_path.is_dir() {
            if let Err(e) = fs::remove_dir_all(&final_destination) {
                eprintln!("mv: cannot remove '{final_destination}': {e}");
                return;
            }
        } else {
            if let Err(e) = fs::remove_file(&final_destination) {
                eprintln!("mv: cannot remove '{final_destination}': {e}");
                return;
            }
        }
    }

    
    //    Déplace ou renomme le fichier ou dossier :
    //    - Si le flag "-v" est activé, affiche le déplacement effectué.
    //    - Si une erreur survient, affiche un message d’erreur.
    match fs::rename(source, &final_destination) {
        Ok(_) => {
            if let Some(f) = flag {
                if f == "-v" {
                    println!("renamed '{source}' -> '{final_destination}'");
                }
            }
        }
        Err(e) => eprintln!("mv: cannot move '{source}' to '{final_destination}': {e}"),
    }
}


/// # Fonction : `handle_mv`
/// Gère la commande **`mv`** en ligne de commande.
///
/// Analyse les arguments passés par l’utilisateur et appelle ensuite
/// [`move_file()`] pour effectuer le déplacement ou le renommage.
///
/// # Fonctionnement
/// 1. Vérifie qu’il y a suffisamment d’arguments.  
/// 2. Détermine si le premier argument est un flag (`-i` ou `-v`).  
/// 3. Identifie le fichier source et la destination.  
/// 4. Appelle la fonction [`move_file()`] avec les bons paramètres.
pub fn handle_mv(args: &[String]) {
    // Vérifie qu'il y a suffisamment d'arguments.
    if args.len() < 2 {
        eprintln!("mv: missing file operand");
        eprintln!("Try 'mv --help' for more information.");
        return;
    }

    let mut flag: Option<&str> = None;
    let (source, destination);

    // Si l’utilisateur a passé au moins 3 arguments :
    // - le premier est considéré comme un flag (ex. "-i" ou "-v").
    // Sinon :
    // - les deux premiers arguments correspondent directement à la source et la destination.
    if args.len() == 3 {
        flag = Some(args[0].as_str());
        source = &args[1];
        destination = &args[2];
    } else {
        source = &args[0];
        destination = &args[1];
    }

    move_file(flag, source, destination);
}