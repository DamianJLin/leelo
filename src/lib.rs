use csv::Reader;
use csv::Writer;
use std::cmp;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::f64;
use std::io;
use std::io::Write;

const INITIAL_RATING: f64 = 1000.;
// RATING_CONST determines how a expected_win_probability is inferred from a difference in rating.
// Set to 200/ln(3) such that a rating difference of 200 gives a 75/25 expected win probability.
const RATING_CONST: f64 = 182.047845;
const K: f64 = 40.; // Rating sensitivity (max. rating change from a single game or twice the rating change from an evenly matched game).

enum MatchResult {
    WhiteWin,
    BlackWin,
    Draw,
}

enum Operation {
    Help,
    New,
    AddPlayer(String),
    Update {
        white_player_id: String,
        black_player_id: String,
        result: MatchResult,
    },
    View,
}

pub struct Config {
    filename: Option<String>,
    operation: Operation,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
        if args.len() < 2 {
            return Err("not enough arguments. Try leelo help.".into());
        }

        let command = args[1].clone();
        let mut filename = None;

        let operation = match command.as_str() {
            // leelo help
            "help" | "h" => Operation::Help,

            // leelo new <filename>
            "new" | "n" => {
                if args.len() < 3 {
                    return Err("not enough arguments for this command.".into());
                }
                filename = Some(args[2].clone());
                Operation::New
            }

            // leelo player <player_id> <filename>
            "player" | "p" => {
                if args.len() < 4 {
                    return Err("not enough arguments for this command.".into());
                }
                filename = Some(args[3].clone());
                Operation::AddPlayer(args[2].clone())
            }

            // leelo game <white_player_id> <black_player_id> <result> <filename>
            "game" | "g" => {
                if args.len() < 6 {
                    return Err("not enough arguments for this command.".into());
                }
                let result = match args[4].as_str() {
                    "1-0" => MatchResult::WhiteWin,
                    "0-1" => MatchResult::BlackWin,
                    "0.5-0.5" => MatchResult::Draw,
                    _ => return Err("unable to interpret score argument.".into()),
                };
                filename = Some(args[5].clone());
                Operation::Update {
                    white_player_id: args[2].clone(),
                    black_player_id: args[3].clone(),
                    result: result,
                }
            }

            // leelo view <filename>
            "view" | "v" => {
                if args.len() < 3 {
                    return Err("not enough arguments for this command.".into());
                };
                filename = Some(args[2].clone());
                Operation::View
            }

            // leelo _ *<args>
            _ => return Err("unknown command. Try leelo help.".into()),
        };

        Ok(
            Config {
                filename,
                operation,
            }
        )
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
        Entry::Occupied(_) => return Err("player_id already in use.".into()),
        Entry::Vacant(v) => {
            v.insert(INITIAL_RATING);
        }
    }

    Ok(())
}

fn update_ratings(
    white_player_id: String,
    black_player_id: String,
    result: MatchResult,
    data: &mut HashMap<String, f64>,
) -> Result<(), Box<dyn Error>> {
    let white_rating = match (*data).get(&white_player_id) {
        Some(rat) => f64::from(*rat),
        None => return Err("white player not found.".into()),
    };
    let black_rating = match (*data).get(&black_player_id) {
        Some(rat) => f64::from(*rat),
        None => return Err("black player not found.".into()),
    };
    let rating_difference = white_rating - black_rating;
    let white_score_expected = 1. / (f64::exp(-rating_difference / RATING_CONST) + 1.);
    let black_score_expected = 1. - white_score_expected;

    let (white_score, black_score) = match result {
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
            println!("\t\t\tPrint help information");
            println!("\tnew <file>");
            println!("\t\t\tCreate new leelo table");
            println!("\tgame <white> <black> <score> <file>");
            println!("\t\t\tRecord results of a game and update ratings");
            println!("\tplayer <id> <file>");
            println!("\t\t\tCreate new player");
            println!("\tview <file>");
            println!("\t\t\tView players and ratings");
        }
        Operation::New => {
            let mut data: HashMap<String, f64> = HashMap::new();
            let filename = config.filename.unwrap();
            write_to_csv(&filename, &mut data)?;
        }
        Operation::Update {
            white_player_id,
            black_player_id,
            result,
        } => {
            let mut data: HashMap<String, f64> = HashMap::new();
            let filename = config.filename.unwrap();
            read_to_hashmap(&filename, &mut data)?;
            update_ratings(white_player_id, black_player_id, result, &mut data)?;
            write_to_csv(&filename, &mut data)?;
        }
        Operation::AddPlayer(player_id) => {
            let mut data: HashMap<String, f64> = HashMap::new();
            let filename = config.filename.unwrap();
            read_to_hashmap(&filename, &mut data)?;
            create_player(player_id, &mut data)?;
            write_to_csv(&filename, &mut data)?;
        }
        Operation::View => {
            let mut data: HashMap<String, f64> = HashMap::new();
            let filename = config.filename.unwrap();
            read_to_hashmap(&filename, &mut data)?;

            let mut data_vec: Vec<(&String, &f64)> = data.iter().collect();
            data_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            let mut max_player_id_len = 0;
            for (player_id, _) in &data_vec {
                max_player_id_len = cmp::max(max_player_id_len, (*player_id).len());
            }
            for (player_id, rating) in &data_vec {
                let tabs = max_player_id_len / 8 + 1;
                print!(
                    "{}\r{}{}\n",
                    player_id,
                    "\t".repeat(tabs),
                    (**rating).round() as u32
                );
                io::stdout().flush()?;
            }
        }
    };

    Ok(())
}
