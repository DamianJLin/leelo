use std::f64;
use csv::Reader;
use csv::Writer;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::cmp;

const INITIAL_RATING: f64 = 1000.;
// RATING_CONST determines how a expected_win_probability is inferred from a difference in rating.
// Set to 200/ln(3) such that a rating difference of 200 gives a 75/25 expected win probability.
const RATING_CONST: f64 = 182.047845;
const K: f64 = 40.; // Rating sensitivity (max. rating change from a single game or twice the rating change from an evenly matched game).

enum MatchResult {
    WhiteWin,
    BlackWin,
    Draw
}

enum Operation {
    Help,
    New,
    AddPlayer(String),
    Update { white_player_id: String, black_player_id: String, result: MatchResult},
    View,
}

pub struct Config {
    filename: String,
    operation: Operation,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments. Try leelo help.");
        }
        if args[1] == "help" {
            return Ok(Config { filename: "".to_string(), operation: Operation::Help });
        }

        let filename = args[1].clone();
        let command = args[2].clone();
        
        let operation = match command.as_str() {
            "help" | "h" => Operation::Help,
            "new" |"n" => {
                Operation::New
            }                    
            "player" | "p" => {
                if args.len() < 4 {
                    return Err("not enough arguments for this command.");
                }
                Operation::AddPlayer(args[3].clone()) 
            },
            "game" | "g" => {
                if args.len() < 6 {
                    return Err("not enough arguments for this command.");
                }
                let result = match args[5].as_str() {
                    "1-0" => MatchResult::WhiteWin,
                    "0-1" => MatchResult::BlackWin,
                    "0.5-0.5" => MatchResult::Draw,
                    _ => return Err("unable to interpret match_result argument.")
                };
                Operation::Update {
                    white_player_id: args[3].clone(),
                    black_player_id: args[4].clone(),
                    result: result
                }
            }
            "view" | "v" => {
                Operation::View
            }
            _ => return Err("unknown command. Try leelo help."),
        };

        Ok(Config { filename, operation })
    }
}

fn read_to_hashmap(filename: &str, data: &mut HashMap<String, f64>) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(filename)?;

    for result in rdr.deserialize() {
        let (player_id, rating): (String, f64) = result?;
        (*data).insert(player_id, rating);
    }

    Ok(())
}

fn write_to_csv(filename: &str, data: &mut HashMap<String, f64>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(filename)?;

    wtr.write_record(&["Player ID", "Rating"])?;
    for (player_id, rating) in (*data).iter() {
        let record = (player_id, rating);
        wtr.serialize(record)?;
        wtr.flush()?;
    }

    Ok(())
}

fn create_player(player_id: String, data: &mut HashMap<String, f64>) -> Result<(), Box<dyn Error>> {
    match (*data).entry(player_id) {
        Entry::Occupied(_) => {
           return Err("player_id already in use.".into()) 
        }
        Entry::Vacant(v) => {
           v.insert(INITIAL_RATING);
        }
    }

    Ok(())
}

fn update_ratings(white_player_id: String, black_player_id: String, result: MatchResult, data: &mut HashMap<String, f64>) -> Result<(), Box<dyn Error>> {
    let white_rating = match (*data).get(&white_player_id) {
        Some(rat) => f64::from(*rat),
        None => return Err("white player not found.".into())
    };
    let black_rating = match (*data).get(&black_player_id) {
        Some(rat) => f64::from(*rat),
        None => return Err("black player not found.".into())
    };
    let rating_difference = white_rating - black_rating;
    let white_score_expected = 1. / (f64::exp(-rating_difference / RATING_CONST) + 1.);
    let black_score_expected = 1. - white_score_expected;

    let (white_score, black_score) =  match result {
        MatchResult::WhiteWin => (1., 0.),
        MatchResult::BlackWin => (0., 1.),
        MatchResult::Draw => (0.5, 0.5),
    };

    let white_rating_change = K * (white_score - white_score_expected);
    let black_rating_change = K * (black_score - black_score_expected);
    let white_new_rating = white_rating + white_rating_change;
    let black_new_rating = black_rating + black_rating_change;
    
    (*data).insert(white_player_id, white_new_rating);
    (*data).insert(black_player_id, black_new_rating);

    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.operation {
        Operation::Help => {
            println!("A simple Elo rating implementation.");
            println!("");
            println!("USAGE:");
            println!("\tleelo [COMMAND] [ARGUMENTS]");
            println!("");
            println!("COMMANDS:");
            println!("\thelp");
            println!("\t\tPrint help information");
            println!("\tgame <file> <white player id> <black player id> <score>");
            println!("\t\tRecord results of a game and update ratings");
            println!("\tplayer <file> <new player id>");
            println!("\t\tcreate new player");
            println!("\tview <file>");
            println!("\t\tview players and ratings");
        }
        Operation::New => {
            let mut data: HashMap<String, f64> = HashMap::new();
            write_to_csv(&config.filename, &mut data)?;
        }
        Operation::Update{ white_player_id, black_player_id, result } => {
            let mut data: HashMap<String, f64> = HashMap::new();
            read_to_hashmap(&config.filename, &mut data)?;
            update_ratings(white_player_id, black_player_id, result, &mut data)?; 
            write_to_csv(&config.filename, &mut data)?;
        }
        Operation::AddPlayer(player_id) => {
            let mut data: HashMap<String, f64> = HashMap::new();
            read_to_hashmap(&config.filename, &mut data)?;
            create_player(player_id, &mut data)?; 
            write_to_csv(&config.filename, &mut data)?;
        }
        Operation::View => {
            let mut data: HashMap<String, f64> = HashMap::new();
            read_to_hashmap(&config.filename, &mut data)?;

            let mut data_vec: Vec<(&String, &f64)> = data.iter().collect();
            data_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            let mut max_player_id_len = 0;
            for (player_id, _) in &data_vec {
                max_player_id_len = cmp::max(max_player_id_len, (*player_id).len());
            }
            for (player_id, rating) in &data_vec {
                let player_id_len = player_id.len();
                let tabs = (max_player_id_len - player_id_len) / 8 + 2;
                print!("{}{}{}\n", player_id, "\t".repeat(tabs), (**rating).round() as u32);
                io::stdout().flush()?;
            }
        }
    };

    Ok(())
}
