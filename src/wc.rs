use std::fs::File;
use std::io::{self, Write, BufReader, BufRead};
use std::path::Path;

const VERSION: &str = "1.0.0";

/// Implémentation Rust de la commande `wc`.
///
/// Ce module implémente la commande `wc` (word count) avec Rust. Elle permet de compter
/// les lignes, mots, caractères et octets dans des fichiers ou des flux stdin.

/// Structure représentant les options de comptage pour la commande `wc`.
///
/// Chaque champ correspond à une option possible de `wc`.
#[derive(Default, Clone)]
struct Options {
    /// Affiche le nombre d'octets (`-c`).
    show_bytes: bool,
    /// Affiche le nombre de caractères (`-m`).
    show_chars: bool,
    /// Affiche le nombre de lignes (`-l`).
    show_lines: bool,
    /// Affiche le nombre de mots (`-w`).
    show_words: bool,
    /// Affiche la longueur de la ligne la plus longue (`-L`).
    show_max_line_length: bool,
    /// Fichier contenant une liste de fichiers séparés par `\0` (`--files0-from=F`).
    files0_from: Option<String>,
}

/// Structure représentant les résultats du comptage.
///
/// Stocke tous les compteurs pour un fichier ou un flux.
#[derive(Default, Clone)]
struct CountResult {
    /// Nombre de lignes.
    lines: usize,
    /// Nombre de mots.
    words: usize,
    /// Nombre de caractères.
    chars: usize,
    /// Nombre d'octets.
    bytes: usize,
    /// Longueur maximale d'une ligne.
    max_line_length: usize,
}

/// Fonction principale du programme `wc`.
///
/// Démarre un shell interactif permettant de saisir des commandes `wc`. Si l'utilisateur
/// quitte l'utilitaire en tapant `quit`, il reviendra sur le menu principal.
///
/// # Exemple
/// ```no_run
/// wc();
/// ```
pub fn wc() {
    loop {
        println!("\n=== Programme utilitaire wc ===");
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
            // Traiter la commande avec pipe (echo ... | wc ...)
            match process_piped_command(input) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Erreur : {}", e);
                    eprintln!("Stack trace : {:?}", e);
                }
            }
        } else {
            // Traiter la commande simple (wc ...)
            let parts = parse_command_line(input);
            
            if parts.is_empty() {
                continue;
            }

            // Vérifier que la commande commence par wc
            if parts[0] != "wc" {
                println!("Erreur : La commande doit commencer par 'wc'");
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
/// let args = wc_rs::parse_command_line(r#"wc -l "fichier avec espaces.txt""#);
/// assert_eq!(args, vec!["wc", "-l", "fichier avec espaces.txt"]);
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

/// Interprète les séquences d'échappement dans une chaîne.
///
/// # Algorithme
/// - Parcours de la chaîne caractère par caractère.
/// - Si `\`, détermine la séquence (`n`, `t`, `r`, `xFF`, etc...).
/// - Remplace par le caractère correspondant.
///
/// # Arguments
/// * `input` - Chaîne avec séquences d'échappement.
///
/// # Retour
/// Chaîne transformée.
///
/// # Exemple
/// ```rust
/// let text = wc_rs::unescape("Hello\\nWorld");
/// assert_eq!(text, "Hello\nWorld");
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

/// Traite une commande contenant un pipe (`echo ... | wc ...`).
///
/// # Algorithme
/// 1. Sépare par `|`.
/// 2. Vérifie la partie echo.
/// 3. Extrait le texte après echo.
/// 4. Vérifie la partie wc.
/// 5. Parse les options.
/// 6. Applique le comptage sur le texte.
///
/// # Arguments
/// * `input` - Ligne de commande avec pipe.
///
/// # Retour
/// `io::Result<()>` indiquant succès ou erreur.
///
/// # Exemple
/// ```no_run
/// process_piped_command(r#"echo "Hello World" | wc -w"#)?;
/// // Affiche : 2
/// ```
fn process_piped_command(input: &str) -> io::Result<()> {
    let pipe_parts: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    
    if pipe_parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Format invalide : utilisez 'echo [texte] | wc [options]'"
        ));
    }

    let echo_part = pipe_parts[0];
    let wc_part = pipe_parts[1];

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

    // Parser la partie wc avec gestion des guillemets
    let wc_parsed = parse_command_line(wc_part);
    if wc_parsed.is_empty() || wc_parsed[0] != "wc" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Après le pipe, la commande doit être 'wc [options]'"
        ));
    }

    // Parser les options de wc (tout sauf le premier mot "wc")
    let wc_args: Vec<&str> = wc_parsed[1..].iter().map(|s| s.as_str()).collect();
    let (options, _) = parse_arguments(&wc_args)?;

    // Traiter le stdin avec les options
    let text_unescape = unescape(&stdin_text);
    process_stdin(&text_unescape, &options)?;

    Ok(())
}

/// Traite une commande `wc` avec ses arguments.
///
/// # Algorithme
/// - Gère `--help` et `--version`.
/// - Parse les options et les fichiers.
/// - Traite le(s) fichier(s) ou stdin.
///
/// # Arguments
/// * `args` - Arguments de la commande.
///
/// # Retour
/// `io::Result<()>` indiquant succès ou erreur.
///
/// # Exemple
/// ```no_run
/// process_command(&["-l", "fichier.txt"])?;
/// // Affiche : 42 fichier.txt
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

    // Parser les options et les fichiers
    let (options, file_paths) = parse_arguments(args)?;

    // Si aucun fichier spécifié, erreur
    if file_paths.is_empty() {
        println!("Erreur : Aucun fichier spécifié");
        println!("Utilisez 'wc --help' pour plus d'informations");
        return Ok(());
    }

    // Traiter les fichiers
    let mut total = CountResult::default();
    let multiple_files = file_paths.len() > 1;

    for path in &file_paths {
        match process_file(path) {
            Ok(result) => {
                display_result(&result, &options, Some(path));
                
                // Accumuler pour le total
                if multiple_files {
                    total.lines += result.lines;
                    total.words += result.words;
                    total.chars += result.chars;
                    total.bytes += result.bytes;
                    total.max_line_length = total.max_line_length.max(result.max_line_length);
                }
            },
            Err(e) => {
                eprintln!("wc: {}: {}", path, e);
            }
        }
    }

    // Afficher le total si plusieurs fichiers
    if multiple_files {
        display_result(&total, &options, Some("total"));
    }

    Ok(())
}

/// Parse les arguments pour extraire options et chemins de fichiers.
///
/// # Algorithme
/// - Pour chaque argument :
///   - S'il commence par `-`, est traité comme option.
///   - Sinon, est considéré comme un chemin fichier.
///
/// # Arguments
/// * `args` - Slice des arguments.
///
/// # Retour
/// Tuple `(Options, Vec<String>)`.
///
/// # Exemple
/// ```rust
/// let (opts, files) = wc_rs::parse_arguments(&["-l", "fichier.txt"]).unwrap();
/// assert_eq!(files[0], "fichier.txt");
/// ```
fn parse_arguments(args: &[&str]) -> io::Result<(Options, Vec<String>)> {
    let mut options = Options::default();
    let mut file_paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i];
        
        if arg.starts_with("--files0-from=") {
            let file = arg.trim_start_matches("--files0-from=");
            options.files0_from = Some(file.to_string());
        } else if arg.starts_with('-') {
            parse_option(arg, &mut options)?;
        } else {
            file_paths.push(arg.to_string());
        }
        
        i += 1;
    }

    // Si aucune option de comptage n'est spécifiée, afficher lignes, mots et octets (comportement par défaut)
    if !options.show_bytes && !options.show_chars && !options.show_lines 
        && !options.show_words && !options.show_max_line_length {
        options.show_lines = true;
        options.show_words = true;
        options.show_bytes = true;
    }

    Ok((options, file_paths))
}

/// Parse une option et met à jour la structure `Options`.
///
/// # Algorithme
/// - Vérifie si l'option est simple ou combinée.
/// - Met à jour les champs correspondants dans `Options`.
///
/// # Arguments
/// * `opt` - Chaîne représentant l'option.
/// * `options` - Référence mutable de `Options`.
///
/// # Retour
/// `io::Result<()>` indiquant succès ou erreur.
///
/// # Exemple
/// ```rust
/// let mut opts = wc_rs::Options::default();
/// wc_rs::parse_option("-l", &mut opts).unwrap();
/// assert!(opts.show_lines);
/// ```
fn parse_option(opt: &str, options: &mut Options) -> io::Result<()> {
    match opt {
        "-c" | "--bytes" => options.show_bytes = true,
        "-m" | "--chars" => options.show_chars = true,
        "-l" | "--lines" => options.show_lines = true,
        "-w" | "--words" => options.show_words = true,
        "-L" | "--max-line-length" => options.show_max_line_length = true,
        _ => {
            // Gérer les options combinées (ex: -lwc)
            if opt.starts_with('-') && opt.len() > 2 && !opt.starts_with("--") {
                for c in opt[1..].chars() {
                    let single_opt = format!("-{}", c);
                    parse_option(&single_opt, options)?;
                }
            } else if !opt.starts_with("--files0-from=") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Option invalide : {}", opt)
                ));
            }
        }
    }
    Ok(())
}

/// Traite un fichier ligne par ligne.
///
/// # Algorithme
/// - Vérifie l'existence du fichier.
/// - Lit le fichier ligne par ligne avec BufRead.
/// - Compte les lignes, mots, caractères et octets au fur et à mesure.
/// - Trouve la ligne la plus longue.
/// - Retourne les résultats.
///
/// # Arguments
/// * `file_path` - Chemin vers le fichier.
///
/// # Retour
/// `io::Result<CountResult>`.
///
/// # Exemple
/// ```no_run
/// let result = process_file("fichier.txt")?;
/// println!("Lignes: {}", result.lines);
/// ```
fn process_file(file_path: &str) -> io::Result<CountResult> {
    // Vérifier si le fichier existe
    if !Path::new(file_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Aucun fichier ou dossier de ce type")
        ));
    }

    // Ouvrir le fichier
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    let mut result = CountResult::default();
    
    // Traiter ligne par ligne
    for line_result in reader.lines() {
        let line = line_result?;
        
        // Compter les lignes
        result.lines += 1;
        
        // Compter les mots dans la ligne
        result.words += line.split_whitespace().count();
        
        // Compter les caractères dans la ligne (+ 1 pour le \n)
        result.chars += line.chars().count() + 1;
        
        // Compter les octets dans la ligne (+ 1 pour le \n)
        result.bytes += line.as_bytes().len() + 1;
        
        // Trouver la longueur maximale
        let line_length = line.chars().count();
        if line_length > result.max_line_length {
            result.max_line_length = line_length;
        }
    }

    Ok(result)
}

/// Traite le texte provenant de stdin.
///
/// # Algorithme
/// - Compte les statistiques du texte.
/// - Affiche le résultat.
///
/// # Arguments
/// * `text` - Texte à analyser.
/// * `options` - Options.
///
/// # Retour
/// `io::Result<()>`.
fn process_stdin(text: &str, options: &Options) -> io::Result<()> {
    let result = count_content(text);
    display_result(&result, options, None);
    Ok(())
}

/// Compte les lignes, mots, caractères et longueur maximale dans un contenu.
///
/// # Algorithme
/// 1. Compte les lignes en parcourant le texte.
/// 2. Compte les mots en séparant par les espaces blancs.
/// 3. Compte les caractères Unicode.
/// 4. Trouve la ligne la plus longue.
///
/// # Arguments
/// * `content` - Texte à analyser.
///
/// # Retour
/// `CountResult` avec toutes les statistiques.
///
/// # Exemple
/// ```rust
/// use wc_rs::count_content;
/// let result = count_content("Hello World\nBonjour");
/// assert_eq!(result.lines, 2);
/// assert_eq!(result.words, 3);
/// ```
fn count_content(content: &str) -> CountResult {
    let mut result = CountResult::default();

    // Compter les lignes et trouver la ligne la plus longue
    let lines: Vec<&str> = content.lines().collect();
    result.lines = lines.len();
    
    // Si le contenu se termine par un retour à la ligne, on compte cette ligne vide
    if content.ends_with('\n') && !content.is_empty() {
        result.lines += 1;
    }

    for line in &lines {
        let line_length = line.chars().count();
        if line_length > result.max_line_length {
            result.max_line_length = line_length;
        }
    }

    // Compter les mots (séparés par des espaces blancs)
    result.words = content.split_whitespace().count();

    // Compter les caractères Unicode
    result.chars = content.chars().count();

    // Compter les octets
    result.bytes = content.as_bytes().len();

    result
}


/// Affiche les résultats du comptage selon les options.
///
/// # Algorithme
/// - Affiche chaque compteur demandé dans l'ordre : lignes, mots, caractères/octets, longueur max.
/// - Ajoute le nom du fichier si fourni.
///
/// # Arguments
/// * `result` - Résultats du comptage.
/// * `options` - Options indiquant quoi afficher.
/// * `filename` - Nom du fichier optionnel.
///
/// # Exemple
/// ```rust
/// let result = CountResult { lines: 10, words: 50, chars: 200, bytes: 200, max_line_length: 80 };
/// display_result(&result, &options, Some("fichier.txt"));
/// // Affiche : 10 50 200 fichier.txt
/// ```
fn display_result(result: &CountResult, options: &Options, filename: Option<&str>) {
    let mut output = String::new();

    if options.show_lines {
        output.push_str(&format!("{:7} ", result.lines));
    }

    if options.show_words {
        output.push_str(&format!("{:7} ", result.words));
    }

    // Si -m et -c sont tous les deux spécifiés, -m prend la priorité
    if options.show_chars {
        output.push_str(&format!("{:7} ", result.chars));
    } else if options.show_bytes {
        output.push_str(&format!("{:7} ", result.bytes));
    }

    if options.show_max_line_length {
        output.push_str(&format!("{:7} ", result.max_line_length));
    }

    if let Some(name) = filename {
        output.push_str(name);
    }

    println!("{}", output.trim_end());
}

/// Affiche l'aide complète du programme `wc`.
///
/// # Exemple
/// ```no_run
/// display_help();
/// ```
fn display_help() {
    println!("Usage: wc [OPTIONS] [FICHIER]...");
    println!();
    println!("Affiche le nombre de lignes, mots et octets pour chaque fichier.");
    println!();
    println!("Options:");
    println!("  -c, --bytes              Affiche le nombre d'octets");
    println!("  -m, --chars              Affiche le nombre de caractères (utile avec UTF-8)");
    println!("  -l, --lines              Affiche le nombre de lignes");
    println!("  -w, --words              Affiche le nombre de mots");
    println!("  -L, --max-line-length    Affiche la longueur de la ligne la plus longue");
    println!("      --help               Affiche cette aide et quitte");
    println!("      --version            Affiche la version et quitte");
    println!();
    println!("Sans options, wc affiche par défaut : lignes, mots et octets.");
    println!();
    println!("Exemples:");
    println!("  wc fichier.txt");
    println!("  wc -l fichier.txt");
    println!("  wc -lwc fichier1.txt fichier2.txt");
    println!("  wc \"chemin/avec des espaces/fichier.txt\"");
    println!("  echo \"Bonjour le monde\" | wc -w");
}

/// Affiche la version du programme.
///
/// # Exemple
/// ```no_run
/// display_version();
/// ```
fn display_version() {
    println!("wc version {}", VERSION);
    println!("Implémentation Rust de la commande wc");
}