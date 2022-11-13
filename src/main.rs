use serde_json::{json, };
use std::{
    collections::HashMap,
    io::{stdin},
};

fn new_deck(fmt: Option<&String>, in_method: Option<&String>, outfile: String) -> Result<(),String> {
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
    println!("Your list is as follows:");
    for (key, val) in decklist.iter() {
        println!("  {val}x: {key}");
    }
    let deck_json = json!({
        "format": fmt,
        "name": outfile,
        "list": decklist 
    }).to_string();
    deckliste_rs::save_deck_file(outfile, deck_json)
    // Ok(())
}
fn update_card_list() {
    deckliste_rs::build_lightweight_cards(String::from("cards.gz")).unwrap();
}

fn check_price(in_method: String, input: Option<&String>) -> Result<(), String> {
    println!("STUB: price is 0.00");
    if in_method == *"file" {
        deckliste_rs::parse_deck_file(input.unwrap().to_string())?;
    }
    Ok(())
}

fn check_legality(format: String, _in_method: String, _input: Option<&String>) -> Result<(), String> {
    /* I suppose if we parse from stdin and a decklist the same way as new_deck
     * maybe it would make sense to do that in main, then pass in the format and the card list.*/
    let _formats = ["casual", "standard", "modern", "commander", "legacy", "vintage"];
    if format == "all" {
        /* parse list or file if provided. */
        println!("get legality card by card or something.");
    }
    Ok(())
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
            let format = sub_matches.get_one::<String>("format").unwrap().to_string();
            check_legality(format, String::from(&input_method),
                           sub_matches.try_get_one::<String>(&input_method).unwrap_or(None)).unwrap();
        },
        Some(("update-cards", _)) => update_card_list(),
        _ => unreachable!(),
    };
}
