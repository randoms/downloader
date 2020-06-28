use std::collections::HashMap;

trait Trim2 {
    fn trim2(&self) -> String;
}

impl Trim2 for String {
    fn trim2(&self) -> String {
        let res = self.trim();
        let mut res = String::from(res);
        if res.starts_with('"') {
            res = (&res[1..(res.len() - 1)]).to_owned();
        }
        if res.ends_with('"') {
            res = (&res[0..(res.len() - 2)]).to_owned();
        }
        return res;
    }
}

pub fn get_data_from_header(header: &str) -> HashMap<String, String> {
    let key_values: Vec<String> = header.split(';').map(String::from).collect();
    let mut values_map = HashMap::new();
    for key_value in key_values.into_iter() {
        if key_value.find("=").is_some() {
            let key_value_list: Vec<String> = key_value.split('=').map(String::from).collect();
            values_map.insert((&key_value_list[0]).to_owned(), key_value_list[1].trim2());
        }
    }
    return values_map;
}