use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize}; //TODO: haal dit weg en maak wrapper structs in scribe?
use serde::{Deserialize, Serialize};

pub trait HasData {
    fn data(&self) -> &Vec<SensorDatapoint>;
}

pub trait SetData {
    fn set_data(&self, data: HashMap<String, f64>);
}

pub trait TranslatorGetterSetter {
    fn field_names(&self) -> Vec<String>;
    fn insert<T: 'static>(&mut self, field_string: &str, value: T) -> Option<()>;
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
