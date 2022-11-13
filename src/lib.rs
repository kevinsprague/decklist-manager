use std::{
    collections::HashMap,
    fs,
    fs::{File},
    io::{BufRead, BufReader,},
    path::Path,
};
use clap::{Arg, arg, ArgGroup, Command};
use serde::Serialize;

#[derive(Serialize, serde::Deserialize, Debug)]
struct Card {
    name: String,
    type_line: String,
    mana_cost: String,
    cmc: f64,
    oracle: String,
    color_id: Vec<String>,
    legality: HashMap<String, bool>
}

pub fn build_lightweight_cards(filename: String) -> Result<(), String> {
    /* nonzero chance that this should validate the file existence
     * but this seems excessive for now.
     */
    let mut cards: Vec<Card> = Vec::new();
    let file = File::open(Path::new(&filename)).expect("File open issue!");
    let reader = BufReader::new(file);
    let card_json: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let card_list = card_json.as_array().unwrap();
    for card in card_list {
        let mut oracle = String::new();
        if card["name"].as_str().unwrap().contains("//") {
            /* FIXME: Need to add this functionality */
            continue;
        }
        oracle += card["oracle_text"].as_str().unwrap();
        let mut color_id = vec![];
        for j in card["color_identity"].as_array().unwrap() {
            color_id.push(j.as_str().unwrap().to_string());
        }
        let mut legality_map: HashMap<String, bool> = HashMap::new();
        for (k,v) in card["legalities"].as_object().unwrap() {
            let val: bool = v == "legal";
            legality_map.insert(k.to_string(), val);
        }
        let this_card = Card {
            name: card["name"].as_str().unwrap().to_string(),
            type_line: card["type_line"].as_str().unwrap().to_string(),
            mana_cost: card["mana_cost"].as_str().unwrap().to_string(),
            cmc: card["cmc"].as_f64().unwrap(),
            oracle,
            color_id,
            legality: legality_map
        };
        cards.push(this_card);
    }
    std::fs::write("cards.json",serde_json::to_string(&cards).unwrap().as_bytes()).unwrap();
    Ok(())
}
pub fn fetch_api() -> Result<(), String> {
    let body = reqwest::blocking::get("https://api.scryfall.com/bulk-data/oracle-cards").unwrap().text().unwrap();
    let save = Path::new("cards.gz");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let url = json["download_uri"].as_str().unwrap();
    let mut card_data = reqwest::blocking::get(url).unwrap();
    let mut buf: Vec<u8> = vec![];
    card_data.copy_to(&mut buf).unwrap();
    fs::write(save, buf).unwrap();
    Ok(())
}

pub fn cli() -> Command {
    Command::new("deckliste_rs")
    .about("Magic the Gathering Version Control CLI")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
        Command::new("update-cards")
        .about("Build new cache of cards from scryfall API")
    )
    .subcommand(
        /* new-deck */
        Command::new("new-deck")
        .about("create a new deck and save it in <savefile>")
        .arg(arg!(<savefile>))
        // .arg(arg!(--name <name> "name of the deck. (uses <savefile> as name if omitted.)"))
        .arg(Arg::new("format")
            .required(false)
            .short('f')
            .long("format")
            .default_value("casual")
            .value_parser(["casual", "standard", "modern", "commander", "legacy", "vintage"]))
        .arg(arg!(--file <filename> "read decklist from file (read from stdin if not present)")
        .required(false)))
    .subcommand(
        /* check-price */
        Command::new("check-price")
        .about("check price for a card/list/deckfile")
        .arg(arg!(--file <filename> "read from file"))
        .arg(arg!(--list            "read from stdin"))
        .arg(arg!(--single <cardname> "read cardname (in quotes) from command line"))
        .group(ArgGroup::new("input_type")
            .required(true)
            .args(["file", "list", "single"])))
    .subcommand(
        Command::new("check-legality")
        .about("check format legality for a card/list/deckfile")
        .arg(arg!(--file <filename> "read list from file"))
        .arg(arg!(--list            "read list from stdin"))
        .arg(arg!(--single <cardname> "read single cardname (in quotes) from command line"))
        .group(ArgGroup::new("input_type")
            .required(true)
            .args(["file", "list", "single"]))
        .arg(Arg::new("format")
            .required(false)
            .default_value("all")
            .value_parser(["all", "casual", "standard", 
                           "modern", "commander", "legacy", "vintage"])))
}

fn validate_file_existence(filename: &str) -> bool {
    /* Returns true if the file exists and ?can be opened?.
     * Returns false if the file doesn't exist or user doesn't have
     * permission to read the file. */
    let filepath = Path::new(filename);
    filepath.try_exists().unwrap_or(false)
}

pub fn parse_card_line(line: String) -> Result<(u32, String), String> {
    let numeric_cards = ["borrowing 100,000 arrows", "borrowing 100000 arrows",
                         "guan-yu's 1,000-li march", "guan-yu's 1000-li march",
                         "+2 mace", "5 alarm fire", "50 feet of rope",
                         "celebr-8000", "d00-dl, caricaturist"];
    for cardname in numeric_cards {
        if line.contains(cardname) {
            let (a, b) = line.split_once(cardname).unwrap();
            let qty = if a.len() < b.len() { a } else { b };
            let q = qty.trim().trim_matches('x').parse().unwrap_or(1);
            return Ok((q, String::from(cardname)));
        }
    }
    let split_point = line.find(|c: char| c.is_ascii_digit());
    if let Some(..) = split_point {
        let split_point = split_point.unwrap();
        let (qty, name) = if line.len() - split_point < line.len() / 2 {
            // if the quantity is in second half of the line, we can just split at
            // the number itself
            let (name, qty) = line.split_at(split_point);
            (qty, name.trim_matches('x').trim())
        } else {
            // if not, then we should split at the first space.
            line.split_once(' ').unwrap()
        };
        let quantity = qty.trim().trim_matches('x').parse().unwrap();
        Ok((quantity, String::from(name)))
    } else {
        Ok((1, line))
    }
}

pub fn parse_deck_file(filename: String) -> Result<HashMap<String, u32>, String> {
    if !validate_file_existence(&filename) {
        return Err(String::from("File access issues. Does {filename} exist?"));
    }
    let mut decklist: HashMap<String, u32> = HashMap::new();
    let file = File::open(&filename).unwrap();
    let reader = BufReader::new(file);
    if filename.strip_suffix(".dk").is_some() {
        let json: serde_json::Value = serde_json::from_reader(reader).unwrap();
        let list = json.get("list")
            .expect("No \"list\" attribute in .dk file, rebuild needed.")
            .as_object().unwrap();
        for k in list.keys() {
            decklist.insert(k.to_string(), list[k].as_u64().unwrap() as u32);
        }
    } else {
        for line in reader.lines().map(|line| line.unwrap()) {
            let (qty, name) = parse_card_line(line)?;
            decklist.entry(name).and_modify(|copies| *copies += 1).or_insert(qty);
        }
    }
    Ok(decklist)
}


pub fn save_deck_file(filename: String, json_string: String) -> Result<(), String> {
    let filepath = Path::new(&filename);
    match filepath.try_exists() {
        Ok(true) => {
            println!("Warning: File exists! Overwrite? (y/n)");
            let stdin = std::io::stdin();
            loop {
                let mut buf = String::new();
                match stdin.read_line(&mut buf) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{e}");
                        return Err(String::from("read error"));
                    },
                };
                if buf.is_empty() {
                    continue;
                }
                match buf.chars().next().unwrap_or('?').to_ascii_lowercase() {
                    'y' => break,
                    'n' => return Err(String::from("File Overwrite Prevented")),
                    _ => continue,
                };
            }
        },
        Ok(false) => (),
        Err(_) => return Err(String::from("Permissions problem")),
    };
    match fs::write(filepath, json_string) {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Write problem")),
    }
}
