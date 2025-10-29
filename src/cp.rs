//! # Module `cp`
//!
//! Ce module implémente la commande Unix **`cp`** en Rust.
//!
//! Il permet de **copier un fichier** d’un emplacement à un autre,
//! avec la prise en charge des options suivantes :
//!
//! - `-i` : demande confirmation avant d’écraser un fichier existant (*interactive*).  
//! - `-v` : affiche le nom des fichiers copiés (*verbose*).

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// # Fonction : `copy_file`
///
/// Copie un fichier d’un emplacement à un autre, en reproduisant le comportement
/// de la commande Unix **`cp`**.
///
/// ## Fonctionnement :
/// 1. Vérifie si le fichier source existe.
/// 2. Détermine si la destination est un dossier ou un fichier.
/// 3. Si la destination existe déjà :  
///     - et que le flag `-i` est activé, demande confirmation avant d’écraser.
/// 4. Copie le fichier vers la destination.
/// 5. Si le flag `-v` est activé, affiche le nom du fichier copié.
///
/// ## Flags pris en charge :
/// - `-i` : *interactive* → demande confirmation avant d’écraser un fichier existant.  
/// - `-v` : *verbose* → affiche les fichiers copiés.
fn copy_file(flag: Option<&str>, source: &str, destination: &str) {
    
    // Vérifie si le fichier source existe
    if !Path::new(source).exists() {
        eprintln!("cp: cannot stat '{source}': No such file or directory");
        return;
    }

    
    //    Vérifie si la destination est un fichier ou un répertoire :
    //    - Si c’est un répertoire, on ajoute le nom du fichier source à la fin.
    //    - Sinon, on considère que la destination est un fichier et on garde son nom tel quel.
    let final_destination = if Path::new(destination).is_dir() {
        let file_name = Path::new(source)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        format!("{}/{}", destination.trim_end_matches('/'), file_name)
    } else {
        destination.to_string()
    };

    
    //    Vérifie si le fichier de destination existe déjà :
    //    - Si oui, et que l’utilisateur a passé le flag -i (interactive),
    //      on lui demande s’il veut écraser le fichier existant.
    //    - Si l’utilisateur tape 'y', le programme continue
    //      et effectuera la copie juste après.
    //   - Sinon, la copie est annulée.
    if Path::new(&final_destination).exists() {
        if let Some(f) = flag {
            if f == "-i" {
                print!("cp: overwrite '{final_destination}'? ");
                io::stdout().flush().unwrap();
                let mut answer = String::new();
                io::stdin().read_line(&mut answer).unwrap();

                // Si l'utilisateur ne confirme pas, on annule la copie
                if !answer.trim().eq_ignore_ascii_case("y") {
                    println!("cp: not overwritten.");
                    return;
                }
            }
        }
    }

    
    //    Copie du fichier (sauf si l’utilisateur a refusé précédemment).
    //    Si le flag -v (verbose) est activé, on affiche le déplacement effectué.
    match fs::copy(source, &final_destination) {
        Ok(_) => {
            if let Some(f) = flag {
                if f == "-v" {
                    println!("'{source}' -> '{final_destination}'");
                }
            }
        }
        Err(_) => eprintln!("cp: cannot copy '{source}' to '{final_destination}'"),
    }
}


/// # Fonction : `handle_cp`
///
/// Gère la commande **`cp`** en ligne de commande.
/// Elle analyse les arguments passés par l’utilisateur et appelle la fonction
/// [`copy_file`] pour exécuter la copie réelle du fichier.
///
/// ## Fonctionnement :
/// 1. Vérifie qu’il y a suffisamment d’arguments.  
/// 2. Détermine si le premier argument est un flag (`-i` ou `-v`).  
/// 3. Identifie le fichier source et la destination.  
/// 4. Appelle la fonction [`copy_file`] avec les bons paramètres.
pub fn handle_cp(args: &[String]) {
    
    
    //    Vérifie qu'il y a suffisamment d'arguments :
    //    - Si le nombre d'arguments est inférieur à 2,
    //      on affiche un message d'erreur et on arrête la fonction.
    if args.len() < 2 {
        eprintln!("cp: missing file operand");
        eprintln!("Try 'cp --help' for more information.");
        return;
    }

    let mut flag: Option<&str> = None;
    let (source, destination);

    //    Si l’utilisateur a passé au moins 3 arguments,
    //    le premier est considéré comme un flag (ex. "-i" ou "-v").
    //    Sinon, les deux premiers arguments correspondent
    //    directement à la source et à la destination.
    if args.len() == 3 {
        flag = Some(args[0].as_str());
        source = &args[1];
        destination = &args[2];
    } else {
        source = &args[0];
        destination = &args[1];
    }

    copy_file(flag, source, destination);
}