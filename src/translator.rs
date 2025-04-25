use std::path;

use crate::structs::{HasData, Mapping, MirrorTrait, ValueType};
use evalexpr::*;
use lookups::{HashLookup, LkupHashMap, Lookup};

pub fn translate_to_db_object<Y: HasData, T: MirrorTrait + Default>(sensor_data: Y) -> T {
    //TODO: panicked als er geen mapping is.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("mapping.csv")
        .unwrap();
    let mut map = LkupHashMap::new(HashLookup::with_multi_keys(), |key: &Mapping| {
        key.address.to_string()
    });

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        map.insert(key.to_lowercase(), mapping);
    }

    let mut object = T::default();
    let field_names = object.field_names();

    for field_name in field_names {
        let sensor_mapping_result = map.get(&field_name.to_lowercase());
        if sensor_mapping_result.is_some() {
            let sensor_mapping = sensor_mapping_result.unwrap();
            let sensor_value: f64;
            match sensor_mapping.mapping_type {
                ValueType::Simple => {
                    let modbus_data_option = sensor_data.data().iter().find(|x| {
                        x.address
                            == u16::from_str_radix(
                                sensor_mapping.address.trim_start_matches("0X"),
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

                    let precompiled = build_operator_tree::<DefaultNumericTypes>(address).unwrap();
                    let mut context = HashMapContext::<DefaultNumericTypes>::new();
                    for ad in addresses_clean {
                        let val;
                        //find value for address and add to context
                        let val_result = sensor_data.data().iter().find(|x| {
                            x.address
                                == u16::from_str_radix(ad.trim_start_matches("0X"), 16).unwrap()
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

                    let precompiled = build_operator_tree::<DefaultNumericTypes>(address).unwrap();
                    let mut context = HashMapContext::<DefaultNumericTypes>::new();
                    for ad in addresses_clean {
                        let val;
                        //find value for address and add to context
                        let val_result = sensor_data.data().iter().find(|x| {
                            x.address
                                == u16::from_str_radix(ad.trim_start_matches("0X"), 16).unwrap()
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
            object.set(&field_name, Some(sensor_value as f32)).unwrap();
        }
    }
    //TODO: dit weghalen, is voor demo
    // object.identifier = 9999;
    // object.expires = Some(Utc::now().naive_utc());
    // object.insmartmoduleid = 4424;
    // object.name = "demo".to_string();
    // object.boilertypeid = 0000;
    // object.boilertypename = "demo".to_string();
    // object.systemid = 0000;
    // object.systemname = "demo".to_string();
    object
}

pub fn translate_to_front_end_object<Y: MirrorTrait, T: MirrorTrait + Default>(
    sensor_data: Y,
) -> T {
    //TODO: panicked als er geen mapping is.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("mapping.csv")
        .unwrap();
    let mut map = LkupHashMap::new(HashLookup::with_multi_keys(), |key: &Mapping| {
        key.address.to_string()
    });

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        map.insert(key.to_lowercase(), mapping);
    }

    let mut object = T::default();
    let field_names = object.field_names();

    for field_name in field_names {
        let sensor_mapping_result = map.get(&field_name.to_lowercase());
        if sensor_mapping_result.is_some() {
            let sensor_mapping = sensor_mapping_result.unwrap();
            let sensor_value: f64;
            match sensor_mapping.mapping_type {
                ValueType::Simple => {
                    let sensor_value_option = sensor_data.get(&sensor_mapping.address);

                    if sensor_value_option.is_some() {
                        sensor_value = *sensor_value_option.unwrap();
                    } else {
                        sensor_value = 0.0;
                    }
                }
                ValueType::Combined => {
                    println!("Not supported");
                    sensor_value = 0.0;
                }
                ValueType::Bit => {
                    println!("Not supported");
                    sensor_value = 0.0;
                }
            }
            object.set(&field_name, Some(sensor_value as f32)).unwrap();
        }
    }
    //TODO: dit weghalen, is voor demo
    // object.identifier = 9999;
    // object.expires = Some(Utc::now().naive_utc());
    // object.insmartmoduleid = 4424;
    // object.name = "demo".to_string();
    // object.boilertypeid = 0000;
    // object.boilertypename = "demo".to_string();
    // object.systemid = 0000;
    // object.systemname = "demo".to_string();
    object
}

pub fn translate_single_value<Y: MirrorTrait, T: MirrorTrait>(
    source: &Y,
    mut target: T,
    source_field: &str,
    path: &str,
) -> T {
    //TODO: panicked als er geen mapping is.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap();
    let mut map = LkupHashMap::new(HashLookup::with_multi_keys(), |key: &Mapping| {
        key.address.to_string()
    });

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        map.insert(key.to_lowercase(), mapping);
    }

    let found_value: f32 = *source.get(&source_field).unwrap();
    let target_field = map.get(source_field).unwrap();
    target.set(&target_field.address, found_value);

    target
}

pub fn find_single_value<Y: MirrorTrait>(
    source: &Y,
    field_name: &str,
    path: &str,
) -> std::option::Option<f32> {
    //TODO: panicked als er geen mapping is.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap();
    let mut map = LkupHashMap::new(HashLookup::with_multi_keys(), |key: &Mapping| {
        key.address.to_string()
    });

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        map.insert(key.to_lowercase(), mapping);
    }

    let target_field = map.get(field_name).unwrap();

    let found_value: Option<f32>;
    let found_value_option = source.get(&target_field.address);

    if found_value_option.is_some() {
        found_value = *found_value_option.unwrap();
    } else {
        found_value = Some(0.0);
    }

    found_value
}
