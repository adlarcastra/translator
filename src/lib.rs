pub mod structs;
mod translator;
use crate::translator::translate_to_db_object;
use structs::{HasData, MirrorTrait};
use translator::{find_single_value, translate_single_value, translate_to_front_end_object};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn translate<Y: HasData, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    let translated = translate_to_db_object(sensor_data);
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
) -> f32 {
    let found_value = find_single_value(source, field_name, path);
    found_value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
