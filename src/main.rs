use std::{
    collections::HashMap,
    env::args,
    fs::{File},
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(PartialEq)]
enum EntryMethod {
    FileName,
    MassEntry,
}
fn parse_card_line(line: String) -> Result<(u32, String), String> {
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

fn new_decklist(em: EntryMethod, filename: Option<String>) -> Result<HashMap<String, u32>, String> {
    /* creating a hashmap where the key is the cardname and the quantity is the value */
    let mut decklist: HashMap<String, u32> = HashMap::new();
    if em == EntryMethod::FileName {
        println!("Read the file, build the decklist");
        // deal with file existence
        let x = match filename {
            Some(filename) => filename,
            None => return Err(String::from("Please provide a filename.")),
        };
        let filepath = Path::new(&x);
        match filepath.try_exists() {
            Ok(tf) => match tf {
                true => (),
                false => return Err(String::from("File not found")),
            },
            Err(_) => return Err(String::from("File permission error (or some other weirdness)")),
        };
        let file = File::open(filepath).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines().map(|line| line.unwrap()) {
            let (qty, name) = parse_card_line(line)?;
            decklist.insert(name, qty);
        }
        Ok(decklist)
    }
    else if em == EntryMethod::MassEntry {
        println!("Read from stdin until blank line, build decklist");
        Ok(decklist)
    }
    else {
        Ok(decklist)
    }
}

fn main() -> Result<(), String> {
    let args: Vec<_> = args().collect();
    let dl = new_decklist(EntryMethod::FileName, Some(String::from(&args[1])))?;
    for (key, val) in dl.iter() {
        println!("{key}: {val}");
    }
    Ok(())
}
