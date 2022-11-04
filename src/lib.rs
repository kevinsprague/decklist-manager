use std::{
    collections::HashMap,
    fs::{File},
    io::{BufRead, BufReader,},
    path::Path,
};
use clap::{Arg, arg, ArgGroup, Command};

pub fn cli() -> Command {
    Command::new("deckliste_rs")
    .about("Magic the Gathering Version Control CLI")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
        /* new-deck */
        Command::new("new-deck")
        .about("create a new deck and save it in <SAVEFILE>")
        .arg(arg!(<savefile>))
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
    let mut decklist: HashMap<String, u32> = HashMap::new();
    if !validate_file_existence(&filename) {
        return Err(String::from("File access issues. Does {filename} exist?"));
    }
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines().map(|line| line.unwrap()) {
        let (qty, name) = parse_card_line(line)?;
        decklist.entry(name).and_modify(|copies| *copies += 1).or_insert(qty);
    }
    Ok(decklist)
}

