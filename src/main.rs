use evalexpr::*;
use field_accessor::FieldAccessor;
use lookups::{HashLookup, LkupHashMap, Lookup};
use serde::{Deserialize, Serialize};

pub struct ModbusDatapoint {
    pub address: u16,
    pub value: u16,
}

struct ModbusSensorData {
    data: Vec<ModbusDatapoint>,
}

#[derive(Default, Debug, FieldAccessor)]
struct DbModbusData {
    set_temp: f64,
    valve_outlet_temp: f64,
    valve_inlet_temp: f64,
    room_temp: f64,
    test_value: f64,
    test_value_2: f64,
    ac_input_current: f64,
}

#[derive(Serialize, Deserialize, Debug)]
enum ValueType {
    Simple,
    Combined,
    Bit,
}

#[derive(Serialize, Deserialize, Debug)]
struct Mapping {
    address: String,
    mapping_type: ValueType,
}

fn main() {
    let sensor_data = ModbusSensorData {
        data: vec![
            ModbusDatapoint {
                address: 65535,
                value: 32,
            },
            ModbusDatapoint {
                address: 0,
                value: 16,
            },
            ModbusDatapoint {
                address: 43690,
                value: 32,
            },
            ModbusDatapoint {
                address: 39321,
                value: 60,
            },
            // ModbusDatapoint {
            //     address: "0x1111".to_uppercase(),
            //     value: 100,
            // },
            ModbusDatapoint {
                address: 69,
                value: 6000,
            },
        ],
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("mapping.csv")
        .unwrap();
    let mut map = LkupHashMap::new(HashLookup::with_multi_keys(), |key: &Mapping| {
        key.address.to_string()
    });

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        map.insert(key, mapping);
    }

    let mut object = DbModbusData::default();
    let field_names = object.getstructinfo().field_names;

    for field_name in field_names {
        let sensor_mapping_result = map.get(&field_name.to_lowercase());
        if sensor_mapping_result.is_some() {
            let sensor_mapping = sensor_mapping_result.unwrap();
            let sensor_value: f64;
            match sensor_mapping.mapping_type {
                ValueType::Simple => {
                    let modbus_data_option = sensor_data.data.iter().find(|x| {
                        x.address
                            == u16::from_str_radix(
                                &sensor_mapping.address.trim_start_matches("0X"),
                                16,
                            )
                            .unwrap()
                    });
                    if modbus_data_option.is_some() {
                        let modbus_data = modbus_data_option.unwrap();
                        sensor_value = modbus_data.value as f64;
                    } else {
                        sensor_value = 0.0;
                    }
                }
                ValueType::Combined => {
                    //Get addresses from mathematical expression
                    let address = &sensor_mapping.address;
                    let indices: Vec<_> = address.match_indices("0X").collect();
                    let mut addresses_clean = Vec::with_capacity(indices.len());
                    for i in 0..indices.len() {
                        let ind = indices[i].0;
                        let temp = address.as_bytes();
                        let test = &temp[ind..ind + 6];
                        addresses_clean.push(std::str::from_utf8(test).unwrap());
                    }

                    let precompiled = build_operator_tree::<DefaultNumericTypes>(&address).unwrap();
                    let mut context = HashMapContext::<DefaultNumericTypes>::new();
                    for ad in addresses_clean {
                        let val;
                        //find value for address and add to context
                        let val_result = sensor_data.data.iter().find(|x| {
                            x.address
                                == u16::from_str_radix(&ad.trim_start_matches("0X").to_string(), 16)
                                    .unwrap()
                        });
                        if val_result.is_some() {
                            val = val_result.unwrap().value;
                        } else {
                            val = 0;
                        }
                        context
                            .set_value(ad.to_string().to_uppercase(), Value::from_float(val as f64))
                            .unwrap();
                    }
                    //calculate result
                    //precompiled.
                    let res = precompiled.eval_float_with_context(&context).unwrap();

                    sensor_value = res;
                }
                ValueType::Bit => {
                    //Get addresses from mathematical expression
                    let address = &sensor_mapping.address;
                    let indices: Vec<_> = address.match_indices("0X").collect();
                    let mut addresses_clean = Vec::with_capacity(indices.len());
                    for i in 0..indices.len() {
                        let ind = indices[i].0;
                        let temp = address.as_bytes();
                        let test = &temp[ind..ind + 6];
                        addresses_clean.push(std::str::from_utf8(test).unwrap());
                    }

                    let precompiled = build_operator_tree::<DefaultNumericTypes>(&address).unwrap();
                    let mut context = HashMapContext::<DefaultNumericTypes>::new();
                    for ad in addresses_clean {
                        let val;
                        //find value for address and add to context
                        let val_result = sensor_data.data.iter().find(|x| {
                            x.address
                                == u16::from_str_radix(&ad.trim_start_matches("0X").to_string(), 16)
                                    .unwrap()
                        });
                        if val_result.is_some() {
                            val = val_result.unwrap().value;
                        } else {
                            val = 0;
                        }
                        context
                            .set_value(ad.to_string().to_uppercase(), Value::from_int(val as i64))
                            .unwrap();
                    }
                    //calculate result
                    let res = precompiled.eval_with_context(&context);
                    sensor_value = res.unwrap().as_int().unwrap() as f64;
                }
            }
            object.set(&field_name, sensor_value).unwrap();
        }
    }
}
