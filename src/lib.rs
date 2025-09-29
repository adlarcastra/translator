pub mod structs;
mod translator;
use mirror::Mirror;
use std::{collections::HashMap, fmt::Debug};

use crate::translator::translate_to_db_object;
use structs::{HasData, Mapping, MirrorTrait};
use translator::{
    find_single_value, translate_single_value, translate_to_db_object_new,
    translate_to_front_end_object,
};

pub fn create_mapping(data: &[u8]) -> HashMap<String, Mapping> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(data);

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

#[derive(Mirror, Debug)]
struct Test {
    p_103: f32,
}

#[cfg(test)]
mod tests {
    use crate::translator::parse_address;

    use super::*;

    #[test]
    fn test_translate_to_hashmap() {
        let mut hashmap: HashMap<String, Mapping> = HashMap::new();
        hashmap.insert(
            "test_value".to_owned(),
            Mapping {
                address: "p_103".to_owned(),
                mapping_type: structs::ValueType::Simple,
            },
        );
        let sensor_data = Test { p_103: 10.0 };
        let result = translate_to_hashmap(&sensor_data, &hashmap);
        println!("{:?}", result);
        assert_eq!(result.get("test_value").unwrap().unwrap(), 10.0);
    }

    #[test]
    fn test_parse_address() {
        let temp = parse_address("mb_1_176_input_66/10");
        println!("{:?}", temp);
        assert_eq!(vec!["mb_1_176_input_66"], temp);
        let temp = parse_address("math::abs(mb_1_176_holding_2102 - 2) * (mb_1_176_holding_6189 * (1 - math.min(1, math::abs(mb_1_176_holding_6185))) + mb_1_176_holding_6190 * math.min(1, math::abs(mb_1_176_holding_6185))) + (mb_1_176_holding_2102 - 1) * (mb_1_176_holding_6191 * (1 - (1 - math::max(0, mb_1_176_holding_6186 - 1))) + mb_1_176_holding_6192 * (1 - math::max(0, mb_1_176_holding_6186 - 1)))");
        assert_eq!(
            vec![
                "mb_1_176_holding_2102",
                "mb_1_176_holding_6189",
                "mb_1_176_holding_6185",
                "mb_1_176_holding_6190",
                "mb_1_176_holding_6185",
                "mb_1_176_holding_2102",
                "mb_1_176_holding_6191",
                "mb_1_176_holding_6186",
                "mb_1_176_holding_6192",
                "mb_1_176_holding_6186"
            ],
            temp
        );
    }
}
