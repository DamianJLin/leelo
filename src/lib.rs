use std::f64;
use csv::Reader;
use csv::Writer;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;

const INITIAL_RATING: u32 = 200;
const RATING_CONST: f64 = 400.; // Constant used in calculating probability from rating difference.
const K: f64 = 20.; // Sensitivity when updating ratings.

pub struct Config {
    pub filename: String,
    operation: Operation,
}

enum MatchResult {
    WhiteWin,
    BlackWin,
    Draw
}

enum Operation {
    Help,
    AddPlayer(String),
    Update { white_player_id: String, black_player_id: String, result: MatchResult},
    Reset,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments!");
        }

        let filename = args[1].clone();
        let command = args[2].clone();
        
        let operation = match command.as_str() {
            "help" => Operation::Help,
            "add" => {
                if args.len() < 4 {
                    return Err("not enough arguments for this command");
                }
                Operation::AddPlayer(args[3].clone()) 
            },
            "result" => {
                if args.len() < 6 {
                    return Err("not enough arguments for this command");
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
            "test::reset" => Operation::Reset,
            _ => return Err("unknown command"),
        };

        Ok(Config { filename, operation })
    }
}

type Record = (String, u32);

fn read_to_hashmap(filename: &str, data: &mut HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(filename)?;

    for result in rdr.deserialize() {
        let (player_id, rating): Record = result?;
        (*data).insert(player_id, rating);
    }

    Ok(())
}

fn write_to_csv(filename: &str, data: &mut HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(filename)?;

    wtr.write_record(&["Player ID", "Rating"])?;
    for (player_id, rating) in (*data).iter() {
        let record = (player_id, rating);
        wtr.serialize(record)?;
        wtr.flush()?;
    }

    Ok(())
}

fn create_player(player_id: String, data: &mut HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    match (*data).entry(player_id) {
        Entry::Occupied(_) => {
           return Err("player_id already in use".into()) 
        }
        Entry::Vacant(v) => {
           v.insert(INITIAL_RATING);
        }
    }

    Ok(())
}

fn update_ratings(white_player_id: String, black_player_id: String, result: MatchResult, data: &mut HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    let white_rating = match (*data).get(&white_player_id) {
        Some(rat) => f64::from(*rat),
        None => return Err("white player not found".into())
    };
    let black_rating = match (*data).get(&black_player_id) {
        Some(rat) => f64::from(*rat),
        None => return Err("black player not found".into())
    };
    let rating_difference = white_rating - black_rating;
    let white_score_expected = 1. / (f64::exp(rating_difference / RATING_CONST) + 1.);
    let black_score_expected = 1. - white_score_expected;

    let (white_score, black_score) =  match result {
        MatchResult::WhiteWin => (1., 0.),
        MatchResult::BlackWin => (0., 1.),
        MatchResult::Draw => (0.5, 0.5),
    };

    let white_new_rating = white_rating + K * (white_score - white_score_expected);
    let black_new_rating = black_rating + K * (black_score - black_score_expected);
    let white_new_rating = white_new_rating.round() as u32;
    let black_new_rating = black_new_rating.round() as u32;
    
    (*data).insert(white_player_id, white_new_rating);
    (*data).insert(black_player_id, black_new_rating);

    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.operation {
        Operation::Help => {
            return Err("not implemented yet".into())
        }
        Operation::Update{ white_player_id, black_player_id, result } => {
            let mut data: HashMap<String, u32> = HashMap::new();
            read_to_hashmap(&config.filename, &mut data)?;
            update_ratings(white_player_id, black_player_id, result, &mut data)?; 
            write_to_csv(&config.filename, &mut data)?;
        }
        Operation::AddPlayer(player_id) => {
            let mut data: HashMap<String, u32> = HashMap::new();
            read_to_hashmap(&config.filename, &mut data)?;
            create_player(player_id, &mut data)?; 
            write_to_csv(&config.filename, &mut data)?;
        }
        Operation::Reset => {
            let mut data = HashMap::new();
            create_player("Damian".to_string(), &mut data)?;
            create_player("Daniel".to_string(), &mut data)?;
            write_to_csv(&config.filename, &mut data)?;
        }
    };

    Ok(())
}
