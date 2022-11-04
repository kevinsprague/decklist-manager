#[cfg(test)]
mod tests {
    #[test]
    fn parse_card_test() {
        let lines = ["Abbey Matron", "1 Abbey Matron", "1x Abbey Matron",
                     "x1 Abbey Matron", "Abbey Matron x1", "Abbey Matron 1x"];
        for cardline in lines {
            let result = deckliste_rs::parse_card_line(String::from(cardline));
            assert_eq!(result, Ok((1, String::from("Abbey Matron"))));
        }
    }
}