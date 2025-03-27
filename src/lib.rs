pub mod structs;
mod translator;
use crate::translator::translate_to_db_object;
use structs::{HasData, MirrorTrait, TranslatorGetterSetter};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn translate<Y: HasData, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    let translated = translate_to_db_object(sensor_data);
    translated
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
