use std::fs::File;
use std::io::{self, Read, Write, BufReader};
use std::path::Path;

const VERSION: &str = "1.0.0";
const BUFFER_SIZE: usize = 8192; // Taille du bloc pour la lecture

/// Implémentation Rust de la commande `cat`.
///
/// Ce module implémente la commande `cat` avec Rust. Elle permet de traiter
/// des fichiers et des flux stdin, et de gérer les multiples options de la commande.

/// Structure représentant les options de traitement pour la commande `cat`.
///
/// Chaque champ correspond à une option possible de `cat`.
#[derive(Default, Clone)]
struct Options {
    /// Affiche tous les caractères non imprimables (équivalent à `-vET` ou `-A`).
    show_all: bool,
    /// Numérote uniquement les lignes non vides (`-b`).
    number_nonblank: bool,
    /// Affiche `$` à la fin de chaque ligne (`-E`).
    show_ends: bool,
    /// Numérote toutes les lignes (`-n`).
    number: bool,
    /// Remplace plusieurs lignes vides consécutives par une seule (`-s`).
    squeeze_blank: bool,
    /// Affiche les tabulations sous la forme `^I` (`-T`).
    show_tabs: bool,
    /// Affiche les caractères non imprimables sauf les tabulations et fins de ligne (`-v`).
    show_nonprinting: bool,
}

/// Fonction principale du programme `cat`.
///
/// Démarre un shell interactif permettant de saisir des commandes `cat`. Si l'utilisateur
/// quitte l'utilitaire en tapant `quit`, il reviendra sur le menu principal.
///
/// # Exemple
/// ```no_run
/// cat();
/// ```
pub fn cat() {
    loop {
        println!("\n=== Programme utilitaire cat ===");
        println!("Entrez votre commande (ou 'quit' pour quitter) :");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Erreur lors de la lecture de l'entrée");
        
        let input = input.trim();
        
        if input == "quit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Vérifier si la commande contient un pipe
        if input.contains('|') {
            // Traiter la commande avec pipe (echo ... | cat ...)
            match process_piped_command(input) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Erreur : {}", e);
                    eprintln!("Stack trace : {:?}", e);
                }
            }
        } else {
            // Traiter la commande simple (cat ...)
            let parts = parse_command_line(input);
            
            if parts.is_empty() {
                continue;
            }

            // Vérifier que la commande commence par cat.rs
            if parts[0] != "cat" {
                println!("Erreur : La commande doit commencer par 'cat'");
                continue;
            }

            // Traiter la commande
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            match process_command(&args) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Erreur : {}", e);
                    eprintln!("Stack trace : {:?}", e);
                }
            }
        }
    }
}

/// Parse une ligne de commande en respectant les guillemets et échappements.
///
/// # Algorithme
/// - Parcours caractère par caractère.
/// - Bascule l’état `in_quotes` sur les guillemets.
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
/// let args = cat_rs::parse_command_line(r#"cat -A "fichier avec espaces.txt""#);
/// assert_eq!(args, vec!["cat", "-A", "fichier avec espaces.txt"]);
/// // Résultat : ["cat", "-A", "fichier avec espaces.txt"]
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

/// Interprète les séquences d’échappement dans une chaîne.
///
/// # Algorithme
/// - Parcours de la chaîne caractère par caractère.
/// - Si `\`, détermine la séquence (`n`, `t`, `r`, `xFF`, etc...).
/// - Remplace par le caractère correspondant.
///
/// # Arguments
/// * `input` - Chaîne avec séquences d’échappement.
///
/// # Retour
/// Chaîne transformée.
///
/// # Exemple
/// ```rust
/// let text = cat_rs::unescape("Hello\\nWorld");
/// assert_eq!(text, "Hello\nWorld");
/// // Affiche :
/// // Hello
/// // World
/// ```
fn unescape(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => { output.push('\n'); chars.next(); }
                Some('t') => { output.push('\t'); chars.next(); }
                Some('r') => { output.push('\r'); chars.next(); }
                Some('v') => { output.push('\u{000B}'); chars.next(); }
                Some('a') => { output.push('\u{0007}'); chars.next(); }
                Some('x') => {
                    chars.next();
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(val) = u8::from_str_radix(&hex, 16) {
                        output.push(val as char);
                    }
                }
                Some('\\') => { output.push('\\'); chars.next(); }
                _ => output.push(c),
            }
        } else {
            output.push(c);
        }
    }

    output
}

/// Traite une commande contenant un pipe (`echo ... | cat ...`).
///
/// # Algorithme
/// 1. Sépare par `|`.
/// 2. Vérifie la partie echo.
/// 3. Extrait le texte après echo.
/// 4. Vérifie la partie cat.
/// 5. Parse les options.
/// 6. Applique les options sur le texte.
///
/// # Arguments
/// * `input` - Ligne de commande avec pipe.
///
/// # Retour
/// `io::Result<()>` indiquant succès ou erreur.
///
/// # Exemple
/// ```no_run
/// process_piped_command(r#"echo "Hello" | cat -n"#)?;
/// // Affiche :
/// //      1  Hello
/// ```
fn process_piped_command(input: &str) -> io::Result<()> {
    let pipe_parts: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    
    if pipe_parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Format invalide : utilisez 'echo [texte] | cat.rs [options]'"
        ));
    }

    let echo_part = pipe_parts[0];
    let cat_part = pipe_parts[1];

    // Parser la partie echo avec gestion des guillemets
    let echo_parsed = parse_command_line(echo_part);
    if echo_parsed.is_empty() || echo_parsed[0] != "echo" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "La commande doit commencer par 'echo'"
        ));
    }

    // Extraire le texte après echo (tout sauf le premier mot "echo")
    let stdin_text = echo_parsed[1..].join(" ");

    // Parser la partie cat avec gestion des guillemets
    let cat_parsed = parse_command_line(cat_part);
    if cat_parsed.is_empty() || cat_parsed[0] != "cat" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Après le pipe, la commande doit être 'cat [options]'"
        ));
    }

    // Parser les options de cat (tout sauf le premier mot "cat.rs")
    let cat_args: Vec<&str> = cat_parsed[1..].iter().map(|s| s.as_str()).collect();
    let (options, _) = parse_arguments(&cat_args)?;

    // Traiter le stdin avec les options
    let text_unescape = unescape(&stdin_text);
    process_stdin(&text_unescape, &options)?;

    Ok(())
}

/// Traite une commande `cat` avec ses arguments.
///
/// # Algorithme
/// - Gère `--help` et `--version`.
/// - Parse les options et le fichier.
/// - Traite le fichier ou affiche une erreur.
///
/// # Arguments
/// * `args` - Arguments de la commande.
///
/// # Retour
/// `io::Result<()>` indiquant succès ou erreur.
///
/// # Exemple
/// ```no_run
/// process_command(&["-n", "fichier.txt"])?;
/// // Affiche (exemple) :
/// //      1  Contenu ligne 1
/// //      2  Contenu ligne 2
/// ```
fn process_command(args: &[&str]) -> io::Result<()> {
    // Gérer --help
    if args.contains(&"--help") {
        display_help();
        return Ok(());
    }

    // Gérer --version
    if args.contains(&"--version") {
        display_version();
        return Ok(());
    }

    // Parser les options et le fichier
    let (options, file_path) = parse_arguments(args)?;

    // Traiter le fichier
    if let Some(path) = file_path {
        process_file(path, &options)?;
    } else {
        println!("Erreur : Aucun fichier spécifié");
        println!("Utilisez 'cat --help' pour plus d'informations");
    }

    Ok(())
}

/// Parse les arguments pour extraire options et chemin de fichier.
///
/// # Algorithme
/// - Pour chaque argument :
///   - S’il commence par `-`, est traité comme option.
///   - Sinon, est considéré comme le chemin fichier.
///
/// # Arguments
/// * `args` - Slice des arguments.
///
/// # Retour
/// Tuple `(Options, Option<String>)`.
///
/// # Exemple
/// ```rust
/// let (opts, file) = cat_rs::parse_arguments(&["-n", "fichier.txt"]).unwrap();
/// assert_eq!(file.unwrap(), "fichier.txt");
/// ```
fn parse_arguments(args: &[&str]) -> io::Result<(Options, Option<String>)> {
    let mut options = Options::default();
    let mut file_path: Option<String> = None;

    for arg in args {
        if arg.starts_with('-') {
            parse_option(*arg, &mut options)?;
        } else if file_path.is_none() {
            // Prendre le premier argument qui n'est pas une option comme fichier
            file_path = Some(arg.to_string());
        }
    }

    Ok((options, file_path))
}

/// Parse une option et met à jour la structure `Options`.
///
/// # Algorithme
/// - Vérifie si l’option est simple ou combinée.
/// - Met à jour les champs correspondants dans `Options`.
///
/// # Arguments
/// * `opt` - Chaîne représentant l’option.
/// * `options` - Référence mutable de `Options`.
///
/// # Retour
/// `io::Result<()>` indiquant succès ou erreur.
///
/// # Exemple
/// ```rust
/// let mut opts = cat_rs::Options::default();
/// cat_rs::parse_option("-n", &mut opts).unwrap();
/// assert!(opts.number);
/// ```
fn parse_option(opt: &str, options: &mut Options) -> io::Result<()> {
    match opt {
        "-A" | "--show-all" => {
            options.show_all = true;
            options.show_nonprinting = true;
            options.show_ends = true;
            options.show_tabs = true;
        },
        "-b" | "--number-nonblank" => options.number_nonblank = true,
        "-e" => {
            options.show_nonprinting = true;
            options.show_ends = true;
        },
        "-E" | "--show-ends" => options.show_ends = true,
        "-n" | "--number" => options.number = true,
        "-s" | "--squeeze-blank" => options.squeeze_blank = true,
        "-T" | "--show-tabs" => options.show_tabs = true,
        "-v" | "--show-nonprinting" => options.show_nonprinting = true,
        _ => {
            // Gérer les options combinées (ex: -vET)
            if opt.starts_with('-') && opt.len() > 2 && !opt.starts_with("--") {
                for c in opt[1..].chars() {
                    let single_opt = format!("-{}", c);
                    parse_option(&single_opt, options)?;
                }
            }
        }
    }
    Ok(())
}

/// Traite un fichier avec les options spécifiées.
///
/// # Algorithme
/// - Vérifie l’existence du fichier.
/// - Lit le fichier par blocs de taille `BUFFER_SIZE`.
/// - Convertit les octets en `String`.
/// - Applique les options sur le texte.
/// - Affiche le résultat.
///
/// # Arguments
/// * `file_path` - Chemin vers le fichier.
/// * `options` - Options de traitement.
///
/// # Retour
/// `io::Result<()>`.
///
/// # Exemple
/// ```no_run
/// process_file("fichier.txt".to_string(), &Options::default())?;
/// ```
fn process_file(file_path: String, options: &Options) -> io::Result<()> {
    // Vérifier si le fichier existe
    if !Path::new(&file_path).exists() {
        eprintln!("cat: {}: Aucun fichier ou dossier de ce type", file_path);
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Fichier '{}' introuvable", file_path)
        ));
    }

    // Tenter d'ouvrir le fichier
    let file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cat: {}: {}", file_path, e);
            return Err(e);
        }
    };

    let mut reader = BufReader::new(file);
    let mut content = String::new();

    // Lire le fichier bloc par bloc
    let mut buffer = vec![0u8; BUFFER_SIZE];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // Fin du fichier
            Ok(n) => {
                // Convertir les octets lus en String
                match String::from_utf8(buffer[..n].to_vec()) {
                    Ok(text) => content.push_str(&text),
                    Err(e) => {
                        eprintln!("Erreur : Impossible de lire le contenu du fichier (encodage invalide)");
                        eprintln!("Détails : {}", e);
                        return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                    }
                }
            },
            Err(e) => {
                eprintln!("Erreur : Erreur lors de la lecture du fichier");
                eprintln!("Détails : {}", e);
                return Err(e);
            }
        }
    }

    // Traiter le contenu avec les options
    let result = apply_options(&content, options);
    print!("{}", result);

    Ok(())
}

/// Traite le texte provenant de stdin.
///
/// # Algorithme
/// - Applique les options sur le texte entré par l'utilisateur.
/// - Affiche le résultat.
///
/// # Arguments
/// * `text` - Texte à afficher.
/// * `options` - Options.
///
/// # Retour
/// `io::Result<()>`.
fn process_stdin(text: &str, options: &Options) -> io::Result<()> {
    let result = apply_options(text, options);
    print!("{}", result);
    Ok(())
}

/// Applique toutes les options au contenu.
///
/// # Algorithme
/// 1. Réduit les lignes vides si `squeeze_blank`.
/// 2. Affiche caractères non imprimables si `show_nonprinting`.
/// 3. Affiche tabulations et fins de ligne si demandées.
/// 4. Numérote les lignes selon `number_nonblank` ou `number`.
///
/// # Arguments
/// * `content` - Texte à traiter.
/// * `options` - Options.
///
/// # Retour
/// Texte formaté.
///
/// # Exemple
/// ```rust
/// use cat_rs::{apply_options, Options};
/// let opts = Options { number: true, ..Default::default() };
/// let text = "Ligne1\nLigne2";
/// let result = apply_options(text, &opts);
/// println!("{}", result);
/// // Affiche :
/// //      1  Ligne1
/// //      2  Ligne2
/// ```
fn apply_options(content: &str, options: &Options) -> String {
    let mut result = content.to_string();

    // 1. D'abord, squeeze blank (réduire les lignes vides)
    if options.squeeze_blank {
        result = squeeze_blank_lines(&result);
    }

    // 2. Ensuite, traiter les caractères spéciaux
    if options.show_nonprinting {
        result = show_nonprinting_chars(&result, options.show_tabs, options.show_ends);
    }

    // 3. Afficher les tabulations si demandé (et pas déjà fait par show_nonprinting)
    if options.show_tabs && !options.show_nonprinting {
        result = show_tabs(&result);
    }

    // 4. Afficher les fins de ligne si demandé (et pas déjà fait par show_nonprinting)
    if options.show_ends && !options.show_nonprinting {
        result = show_ends(&result);
    }

    // 5. Numéroter les lignes (à la fin pour avoir les bons numéros)
    if options.number_nonblank {
        result = number_nonblank_lines(&result);
    } else if options.number {
        result = number_lines(&result);
    }

    result
}

/// Numérote toutes les lignes.
///
/// # Algorithme
/// - Itère sur toutes les lignes.
/// - Ajoute un numéro à gauche.
///
/// # Arguments
/// * `content` - Texte.
///
/// # Retour
/// Texte avec lignes numérotées.
fn number_lines(content: &str) -> String {
    content.lines()
        .enumerate()
        .map(|(i, line)| format!("{:6}\t{}", i + 1, line))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Numérote uniquement les lignes non vides.
///
/// # Algorithme
/// - Itère sur chaque ligne.
/// - Numérote seulement si non vide.
///
/// # Arguments
/// * `content` - Texte.
///
/// # Retour
/// Texte avec lignes non vides numérotées.
fn number_nonblank_lines(content: &str) -> String {
    let mut line_number = 1;
    content.lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                let numbered = format!("{:6}\t{}", line_number, line);
                line_number += 1;
                numbered
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Affiche `$` à la fin de chaque ligne.
///
/// # Arguments
/// * `content` - Texte à traiter.
///
/// # Retour
/// Texte avec `$` ajouté à la fin de chaque ligne.
fn show_ends(content: &str) -> String {
    content.lines()
        .map(|line| format!("{}$", line))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Remplace les tabulations par `^I`.
///
/// # Arguments
/// * `content` - Texte à traiter.
///
/// # Retour
/// Texte avec les tabulations remplacées.
fn show_tabs(content: &str) -> String {
    content.replace('\t', "^I")
}

/// Remplace plusieurs lignes vides consécutives par une seule.
///
/// # Algorithme
/// - Parcours les lignes.
/// - Conserve la première ligne vide consécutive.
/// - Ignore les suivantes.
///
/// # Arguments
/// * `content` - Texte.
///
/// # Retour
/// Texte compressé.
fn squeeze_blank_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut previous_blank = false;

    for line in lines {
        let is_blank = line.trim().is_empty();
        
        if is_blank {
            if !previous_blank {
                result.push(line);
            }
            previous_blank = true;
        } else {
            result.push(line);
            previous_blank = false;
        }
    }

    result.join("\n")
}

/// Affiche les caractères non imprimables.
///
/// # Algorithme
/// - Parcours les lignes et les caractères.
/// - Pour les caractères de contrôle, affiche `^X`.
/// - Affiche `$` en fin de ligne si demandé.
/// - Affiche `^I` pour les tabulations si demandé.
///
/// # Arguments
/// * `content` - Texte.
/// * `include_tabs` - Affiche tabulations.
/// * `include_ends` - Affiche `$`.
///
/// # Retour
/// Texte transformé.
fn show_nonprinting_chars(content: &str, include_tabs: bool, include_ends: bool) -> String {
    let mut result = String::new();
    
    for line in content.lines() {
        for ch in line.chars() {
            match ch {
                '\t' => {
                    if include_tabs {
                        result.push_str("^I");
                    } else {
                        result.push(ch);
                    }
                },
                c if c.is_control() && c != '\n' && c != '\r' => {
                    // Caractères de contrôle (ASCII 0-31 et 127)
                    if (c as u32) < 32 {
                        result.push('^');
                        result.push((c as u8 + 64) as char);
                    } else if c as u32 == 127 {
                        result.push_str("^?");
                    } else {
                        result.push(c);
                    }
                },
                _ => result.push(ch),
            }
        }
        
        if include_ends {
            result.push('$');
        }
        result.push('\n');
    }

    // Retirer le dernier \n ajouté en trop
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Affiche l’aide complète du programme `cat`.
///
/// # Exemple
/// ```no_run
/// display_help();
/// ```
fn display_help() {
    println!("Usage: cat [OPTIONS] [FICHIER]...");
    println!();
    println!("Concatène et affiche le contenu des fichiers ou de stdin.");
    println!();
    println!("Options:");
    println!("  -A, --show-all           Affiche tous les caractères non imprimables (équivalent à -vET)");
    println!("  -b, --number-nonblank    Numérote uniquement les lignes non vides");
    println!("  -e                       Affiche $ à la fin de chaque ligne et rend visibles les caractères");
    println!("                           non imprimables (équivalent à -vE)");
    println!("  -E, --show-ends          Affiche $ à la fin de chaque ligne");
    println!("  -n, --number             Numérote toutes les lignes");
    println!("  -s, --squeeze-blank      Remplace plusieurs lignes vides consécutives par une seule");
    println!("  -T, --show-tabs          Affiche les tabulations sous la forme ^I");
    println!("  -v, --show-nonprinting   Affiche les caractères non imprimables sauf les tabulations");
    println!("                           et les fins de ligne");
    println!("      --help               Affiche cette aide et quitte");
    println!("      --version            Affiche la version et quitte");
    println!();
    println!("Exemples:");
    println!("  cat fichier.txt");
    println!("  cat -n fichier.txt");
    println!("  cat -vET fichier.txt");
    println!("  cat \"chemin/avec des espaces/fichier.txt\"");
    println!("  echo \"Bonjour le monde\" | cat -n");
    println!("  echo \"Texte avec \\t tabulation\" | cat -T");
}

/// Affiche la version du programme.
///
/// # Exemple
/// ```no_run
/// display_version();
/// ```
fn display_version() {
    println!("cat version {}", VERSION);
    println!("Implémentation Rust de la commande cat");
}