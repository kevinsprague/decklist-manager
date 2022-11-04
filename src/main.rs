use std::{
    collections::HashMap,
    io::{stdin},
};

/* Need to consider what actions users should be able to take from command line.
 * Also need to consider how to save user decklists.
 *      Thinking maybe as a json. That would allow easy parsing, and stuffing info
 *      like format, last known price, etc
 */

fn new_deck(_format: Option<&String>, in_method: Option<&String>, _savefile: String) -> Result<(),String> {
    let mut decklist: HashMap<String, u32> = HashMap::new();
    if in_method.is_none() {
        let mut buffer = String::new();
        while stdin().read_line(&mut buffer).expect("Line reading error!") > 0 {
            continue;
        }
        for line in buffer.lines() {
            let (qty, name) = deckliste_rs::parse_card_line(String::from(line))?;
            decklist.entry(name).and_modify(|copies| *copies += 1).or_insert(qty);
        }
    }
    else {
        decklist = deckliste_rs::parse_deck_file(in_method.unwrap().to_string())?;
        }
    for (key, val) in decklist.iter() {
        println!("{key}: {val}");
    }
    // figure out how to write it to file. format, etc.
    Ok(())
}

fn check_price(_in_method: String, _input: Option<&String>) -> Result<(), String> {
    println!("STUB: price is 0.00");
    Ok(())
}

fn check_legality(format: String, _in_method: String, _input: Option<&String>) -> Result<(), String> {
    /* I suppose if we parsee from stdin and a decklist the same way as new_deck
     * maybe it would make sense to do that in main, then pass in the format and the card list.*/
    formats = ["casual", "standard", "modern", "commander", "legacy", "vintage"];
    if format == "all" {
        for (f in formats) {
            /* parse list or file if provided. */
            println!("get legality card by card or something.");
        }
    }
}

fn main() {
    let matches = deckliste_rs::cli().get_matches();
    match matches.subcommand() {
        Some(("new-deck", sub_matches)) => {
            let outfile = sub_matches.get_one::<String>("savefile").unwrap().to_string();
            let format = sub_matches.get_one::<String>("format");
            let input_method = sub_matches.get_one::<String>("file");
            new_deck(format, input_method, outfile).unwrap();
        },
        Some(("check-price", sub_matches)) => {
            let input_method: String = sub_matches.ids()
                                                  .map(|id| id.as_str())
                                                  .take(1).collect();
            check_price(String::from(&input_method), 
                        sub_matches.try_get_one::<String>(&input_method).unwrap_or(None)).unwrap();
        },
        Some(("check-legality", sub_matches)) => {
            let input_method: String = sub_matches.ids()
                                                  .map(|id| id.as_str())
                                                  .take(1).collect();
            let format = sub_matches.get_one::<String>("format").unwrap();
            check_legality(format, String::from(&input_method),
                           sub_matches.try_get_one::<String>(&input_method).unwrap_or(None)).unwrap();
        }
        _ => unreachable!(),
    };
}
