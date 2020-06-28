use std::collections::hash_map;

pub fn get_data_from_header(header: &str) {
    let data: Vec<String> = header.split(':').collect();
    let main_key = data[0];
    let key_values: Vec<String> = data[1].split(';').collect();
    let res = hash_map::new();
    res.insert(main_key, )
}