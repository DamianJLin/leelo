use csv::Reader;
use csv::Writer;
use std::collections::HashMap;
use std::error::Error;

pub struct Config {
    pub filename: String,
    pub command: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments!");
        }

        let filename = args[1].clone();
        let command = args[2].clone();

        Ok(Config { filename, command })
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.command.as_str() {
        "write" => {
            let mut data = HashMap::new();
            data.insert("Damian".to_string(), 1000);
            data.insert("Daniel".to_string(), 800);
            write_to_csv(&config.filename, data)?;
        }
        "read" => {
            let mut data = read_to_hashmap(&config.filename)?;
            println!("{:?}", data);
        }
        _ => (),
    };

    Ok(())
}
