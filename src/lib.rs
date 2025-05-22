pub mod structs;
mod translator;
use std::{collections::HashMap, fmt::Debug};

use crate::translator::translate_to_db_object;
use structs::{HasData, Mapping, MirrorTrait};
use translator::{
    find_single_value, translate_single_value, translate_to_db_object_new,
    translate_to_front_end_object,
};

pub fn create_mapping(path: &str) -> HashMap<String, Mapping> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap();

    let mut hashmap: HashMap<String, Mapping> = HashMap::new();

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        hashmap.insert(key, mapping);
    }
    hashmap
}

pub fn translate<Y: HasData, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    translate_to_db_object(sensor_data)
}

pub fn translate_to_hashmap<Y: MirrorTrait + Debug>(
    sensor_data: &Y,
    map: &HashMap<String, Mapping>,
) -> HashMap<String, Option<f32>> {
    translate_to_db_object_new(sensor_data, map)
}

pub fn translate_to_front_end<Y: MirrorTrait, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    translate_to_front_end_object(sensor_data)
}

pub fn translate_single_field_front_end<Y: MirrorTrait, T: MirrorTrait>(
    source: &Y,
    target: T,
    source_field: &str,
    path: &str,
) -> T {
    translate_single_value(source, target, source_field, path)
}

pub fn find_single_value_front_end<Y: MirrorTrait>(
    source: &Y,
    field_name: &str,
    path: &str,
) -> Result<std::option::Option<f32>, Box<dyn std::error::Error>> {
    find_single_value(source, field_name, path)
}

// struct Test {
//     test_vale: String,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

    // #[test]
    // fn test_translate_to_hashmap() {
    //     let mut hashmap: HashMap<String, Mapping> = HashMap::new();
    //     hashmap.insert(
    //         "test".to_owned(),
    //         Mapping {
    //             address: "p_103".to_owned(),
    //             mapping_type: structs::ValueType::Simple,
    //         },
    //     );
    //     let sensor_data = Test {
    //         test_vale: "hoi".to_owned(),
    //     };
    //     let result = translate_to_hashmap(sensor_data, hashmap);
    //     println!("{:?}", result);
    // }
// }
