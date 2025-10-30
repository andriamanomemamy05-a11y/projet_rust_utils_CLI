// main.rs

mod cat; // Déclare le module cat.rs
mod ls;  // Déclare le module ls.rs (à créer)
mod wc;  // Déclare le module wc.rs (à créer)
mod cp;  // Déclare le module cp.rs
mod mv;  // Déclare le module mv.rs
mod rm;  // Déclare le module rm.rs (à créer)
mod head; // Déclare le module head.rs

use std::io::{self, Write};

/// Point d'entrée de l'application.
///
/// Cette fonction affiche un menu interactif permettant de choisir un utilitaire Linux
/// parmi : ls, cat, cp, mv, rm, wc. L'utilisateur peut entrer le numéro correspondant
/// ou `quit` pour quitter l'application.
///
/// L'application boucle tant que l'utilisateur ne choisit pas de quitter.
/// Elle gère les entrées invalides en affichant un message d'erreur.
///
/// # Algorithme
///  - Boucle infinie jusqu'à ce que l'utilisateur tape `quit`.
///  - Affiche le menu avec les différentes options disponibles.
///  - Demande à l'utilisateur de saisir son choix.
///  - Lit l'entrée utilisateur et supprime les espaces et retours à la ligne.
///  - Vérifie si l'entrée est `quit` : si oui, quitte la boucle et termine le programme.
///  - Sinon, effectue un `match` sur la saisie :
///    - `"1"` : appelle la fonction `ls::ls()`
///    - `"2"` : appelle la fonction `cat::cat()`
///    - `"3"` : appelle la fonction `cp::cp()`
///    - `"4"` : appelle la fonction `mv::mv()`
///    - `"5"` : appelle la fonction `rm::rm()`
///    - `"6"` : appelle la fonction `wc::wc()`
///    - Autre : affiche un message d'erreur et redemande une saisie.
///  - Affiche une ligne vide pour séparer les itérations.
///
/// Ce processus se répète jusqu'à ce que l'utilisateur décide de quitter.
pub fn main() {
    loop {
        // Affichage du menu
        println!("Bonjour et bienvenue dans l'utilitaire de commande linux.");
        println!();
        println!("Veuillez choisir votre utilitaire (tapez quit pour quitter) :");
        println!("1) ls");
        println!("2) cat");
        println!("3) cp");
        println!("4) mv");
        println!("5) rm");
        println!("6) wc");
        println!("7) head");
        println!();
        print!("Votre choix : ");
        io::stdout().flush().unwrap(); // Assure que le prompt s'affiche avant la saisie

        // Lecture de l'entrée utilisateur
        let mut choix = String::new();
        io::stdin()
            .read_line(&mut choix)
            .expect("Erreur lors de la lecture de l'entrée");
        let choix = choix.trim(); // Supprime les espaces et le retour à la ligne

        // Gestion de la commande "quit"
        if choix.eq_ignore_ascii_case("quit") {
            println!("A bientôt !");
            break;
        }

        // Match sur l'entrée utilisateur
        match choix {
            "1" => {
                println!("Exécution de ls...");
                ls::ls(); // Appel de la fonction ls (à implémenter)
            }
            "2" => {
                println!("Exécution de cat...");
                cat::cat(); // Appel de la fonction cat
            }
            "3" => {
                println!("Exécution de cp...");
                println!("Syntaxe : [option] <source> <destination>");
                println!("Options disponibles : -i (interactive), -v (verbose)");
                print!("Entrez vos arguments : ");
                io::stdout().flush().unwrap();
                
                let mut args_input = String::new();
                io::stdin()
                    .read_line(&mut args_input)
                    .expect("Erreur lors de la lecture de l'entrée");
                
                // Parse les arguments en Vec<String>
                let args: Vec<String> = args_input
                    .trim()
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                
                // Appelle handle_cp avec les arguments
                cp::handle_cp(&args);
            }
            "4" => {
                println!("Exécution de mv...");
                println!("Syntaxe : [option] <source> <destination>");
                println!("Options disponibles : -i (interactive), -v (verbose)");
                print!("Entrez vos arguments : ");
                io::stdout().flush().unwrap();

                let mut args_input = String::new();
                io::stdin()
                    .read_line(&mut args_input)
                    .expect("Erreur lors de la lecture de l'entrée");

                // Parse les arguments en Vec<String>
                let args: Vec<String> = args_input
                    .trim()
                    .split_whitespace()
                    .map(String::from)
                    .collect();

                // Appelle handle_mv avec les arguments
                mv::handle_mv(&args);
            }
            "5" => {
                println!("Exécution de rm...");
                rm::rm(); // Appel de la fonction rm (à implémenter)
            }
            "6" => {
                println!("Exécution de wc...");
                wc::wc(); // Appel de la fonction wc (à implémenter)
            }
            "7" => {
                println!("Exécution de head...");
                println!("Exécution de head...");
                println!("Syntaxe : [option] <fichier>");
                println!("Options disponibles : -n <nombre> (nombre de lignes), -v (verbose)");
                print!("Entrez vos arguments : ");
                io::stdout().flush().unwrap();

                let mut args_input = String::new();
                io::stdin()
                    .read_line(&mut args_input)
                    .expect("Erreur lors de la lecture de l'entrée");

                // Parse les arguments en Vec<String>
                let args: Vec<String> = args_input
                    .trim()
                    .split_whitespace()
                    .map(String::from)
                    .collect();

                // Appelle handle_head avec les arguments
                head::handle_head(&args);
            }

            _ => {
                // Gestion des entrées invalides
                println!("Option invalide, veuillez réessayer !");
            }
        }

        println!(); // Ligne vide avant le prochain tour
    }
}
