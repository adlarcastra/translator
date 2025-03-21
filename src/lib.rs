mod translator;
mod structs;
use structs::{ModbusSensorData, ToDatabase, HasData, DbModbusData, Mapping, ModbusDatapoint};
use crate::translator::translate_to_db_object;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn translate<Y: HasData>(sensor_data: Y) -> DbModbusData{
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
