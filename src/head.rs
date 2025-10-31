//! # Module `head`
//!
//! Ce module implémente la commande Unix **`head`** en Rust.
//!
//! Il permet d’afficher les premières lignes d’un fichier texte, avec prise en charge
//! des options suivantes :
//!
//! - `-n <nombre>` : permet d’afficher un nombre spécifique de lignes.  
//! - `-v` : affiche le nom du fichier avant son contenu (mode *verbose*).


use std::fs;
/// # Fonction : `head`
///
/// Affiche les premières lignes d’un fichier, en reproduisant le comportement
/// de la commande Unix **`head`**.
///
/// ## Fonctionnement :
/// - Lit le contenu d’un fichier.
/// - Par défaut, affiche les **10 premières lignes**.
/// - Si le flag `-n` est utilisé, affiche le nombre de lignes spécifié.
/// - Si le flag `-v` est utilisé, affiche le nom du fichier avant le contenu.
///
/// ## Flags pris en charge :
/// - `-n <nombre>` : affiche le nombre de lignes indiqué.  
/// - `-v` : *verbose* → affiche le nom du fichier avant son contenu.
fn head(flag: Option<&str>, num: Option<&str>, filename: &str) {
    // Définition du nombre de lignes à afficher par défaut
    let mut num_lines = 10;


    /*
        Si le flag -n est utilisé :
        - Vérifie qu’un argument numérique a bien été fourni après -n.
        - Convertit cet argument en entier.
        - En cas d’erreur (nombre manquant ou invalide), affiche un message d’erreur et quitte le programme.
    */
    if flag == Some("-n") {
        if num.is_none() {
            eprintln!("head: option requires an argument -- 'n'");
            std::process::exit(1);
        }

        // Conversion de l’argument en entier (nombre de lignes)
        match num.unwrap().parse::<usize>() {
            Ok(n) => num_lines = n,
            Err(_) => {
                eprintln!("head: invalid number of lines");
                std::process::exit(1);
            }
        }
    }

    /*
        Lecture du fichier spécifié :
        - Si la lecture réussit :
            - Affiche le nom du fichier si le flag -v est activé.
            - Affiche les premières lignes du fichier jusqu’à la limite définie.
        - En cas d’erreur, affiche un message d’erreur indiquant que le fichier est inaccessible.
    */
    match fs::read_to_string(filename) {
        // Affiche le nom du fichier si le flag -v est présent
        Ok(content) => {
            if flag == Some("-v") {
                println!("==> {} <==", filename);
            }

            // Afficher les premières lignes
            for (i, line) in content.lines().enumerate() {
                if i >= num_lines {
                    break;
                }
                println!("{}", line);
            }
        }
        Err(e) => {
            eprintln!("head: cannot open '{}' for reading: {}", filename, e);
        }
    }
}

/// # Fonction : `handle_head`
///
/// Gère la commande **`head`** en ligne de commande.
/// Elle analyse les arguments passés par l’utilisateur et appelle ensuite
/// la fonction [`head`] pour afficher le contenu du fichier.
///
/// ## Fonctionnement :
/// 1. Vérifie que l’utilisateur a bien passé un nom de fichier.  
/// 2. Détermine si un flag (`-n` ou `-v`) est présent.  
/// 3. Appelle la fonction [`head`] avec les bons paramètres.
pub fn handle_head(args: &[String]) {
    /*
        Vérifie qu'un fichier a été fourni en argument :
        - Si la liste des arguments est vide,
          affiche un message d'erreur et propose d'utiliser "head --help".
     */
    if args.is_empty() {
        eprintln!("head: missing file operand");
        eprintln!("Try 'head --help' for more information.");
        return;
    }

    let mut flag: Option<&str> = None;
    let mut num: Option<&str> = None;
    let filename;

    /*
        Analyse des arguments selon les cas possibles :
        1. head fichier.txt          → args.len() == 1
        2. head -v fichier.txt       → args.len() == 2
        3. head -n 5 fichier.txt     → args.len() == 3
    */
    
    // Cas 1 : Premier argument est un flag
    if args[0].starts_with('-') {
        if args[0] == "-n" {
            if args.len() < 3 {
                eprintln!("head: option requires an argument -- 'n'");
                eprintln!("Usage: head -n <nombre> <fichier>");
                return;
            }
            flag = Some("-n");
            num = Some(args[1].as_str());
            filename = &args[2];
        } else if args[0] == "-v" {
            if args.len() < 2 {
                eprintln!("head: missing file operand after '-v'");
                return;
            }
            flag = Some("-v");
            filename = &args[1];
        } else {
            eprintln!("head: invalid option -- '{}'", &args[0]);
            eprintln!("Try 'head --help' for more information.");
            return;
        }
    } else {
        filename = &args[0];
    }

    head(flag, num, filename);
}
