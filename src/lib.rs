use csv::Reader;
use csv::Writer;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;

const INITIAL_RATING: u32 = 200;

pub struct Config {
    pub filename: String,
    operation: Operation,
}

enum Operation {
    Help,
    AddPlayer(String),
    Update { white_player: String, black_player: String, score: String},
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
            "test::reset" => Operation::Reset,
            _ => return Err("unknown command"),
        };

        Ok(Config { filename, operation })
    }
}

type Record = (String, u32);

fn read_to_hashmap(filename: &str) -> Result<HashMap<String, u32>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(filename)?;
    let mut data = HashMap::new();

    for result in rdr.deserialize() {
        let (player_id, rating): Record = result?;
        data.insert(player_id, rating);
    }

    Ok(data)
}

fn write_to_csv(filename: &str, data: HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(filename)?;

    wtr.write_record(&["Player ID", "Rating"])?;
    for (player_id, rating) in data.iter() {
        let record = (player_id, rating);
        wtr.serialize(record)?;
        wtr.flush()?;
    }

    Ok(())
}

fn create_player(player_id: String, mut data: HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    match data.entry(player_id) {
        Entry::Occupied(_) => {
           return Err("player_id already in use".into()) 
        }
        Entry::Vacant(v) => {
           v.insert(INITIAL_RATING);
        }
    }

    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut data = read_to_hashmap(&config.filename)?;
    match config.operation {
        Operation::Help => {
            return Err("not implemented yet".into())
        }
        Operation::Update{ .. } => {
            return Err("not implemented yet".into())
        }
        Operation::AddPlayer(player_id) => {
           create_player(player_id, data)?; 
           // TODO write_to_csv(&config.filename, data)?;
        }
        Operation::Reset => {
            let mut data = HashMap::new();
            data.insert("Damian".to_string(), 1000);
            data.insert("Daniel".to_string(), 800);
            write_to_csv(&config.filename, data)?;
        }
    };

    Ok(())
}
