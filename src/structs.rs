use borsh::{BorshDeserialize, BorshSerialize}; //TODO: haal dit weg en maak wrapper structs in scribe?
use field_accessor::FieldAccessor;
use serde::{Deserialize, Serialize};

// pub trait ToDatabase {
//     fn to_db_object<Y: HasData>(input_object: Y) -> DbSensorData;
// }

pub trait HasData {
    fn data(&self) -> &Vec<SensorDatapoint>;
}

pub trait TranslatorGetterSetter {
    fn field_names(&self) -> Vec<String>;
    fn insert<T: 'static>(&mut self, field_string: &String, value: T) -> Option<()>;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ValueType {
    Simple,
    Combined,
    Bit,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default)]
pub struct SensorDatapoint {
    pub address: u16,
    pub value: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default)]
pub struct ModbusSensorData {
    pub data: Vec<SensorDatapoint>,
    pub sensor_id: String,
    pub is_partial: bool,
}

impl HasData for ModbusSensorData {
    fn data(&self) -> &Vec<SensorDatapoint> {
        &self.data
    }
}

pub trait MirrorTrait {
    fn field_names(&self) -> &'static [&'static str];
    fn get<T: std::any::Any>(&self, field: &str) -> Option<&T>;
    fn set<T: std::any::Any>(&mut self, field: &str, new_value: T) -> Option<()>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mapping {
    pub address: String,
    pub mapping_type: ValueType,
}
