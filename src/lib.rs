pub mod structs;
mod translator;
use std::collections::HashMap;

use crate::translator::translate_to_db_object;
use structs::{HasData, MirrorTrait};
use translator::{
    find_single_value, translate_single_value, translate_to_db_object_new,
    translate_to_front_end_object,
};

pub fn translate<Y: HasData, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    let translated = translate_to_db_object(sensor_data);
    translated
}

pub fn translate_to_hashmap<Y: MirrorTrait>(sensor_data: Y, path: &str) -> HashMap<String, f64> {
    let translated = translate_to_db_object_new(sensor_data, path);
    translated
}

pub fn translate_to_front_end<Y: MirrorTrait, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    let translated = translate_to_front_end_object(sensor_data);
    translated
}

pub fn translate_single_field_front_end<Y: MirrorTrait, T: MirrorTrait>(
    source: &Y,
    target: T,
    source_field: &str,
    path: &str,
) -> T {
    let translated = translate_single_value(source, target, source_field, path);
    translated
}

pub fn find_single_value_front_end<Y: MirrorTrait>(
    source: &Y,
    field_name: &str,
    path: &str,
) -> Result<std::option::Option<f32>, Box<dyn std::error::Error>> {
    let found_value = find_single_value(source, field_name, path);
    found_value
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_translate_to_hashmap() {
    //     let result = translate_to_hashmap(sensor_data);
    //     assert_eq!(result, 4);
    // }
}
