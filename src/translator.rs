use std::{collections::HashMap, fmt::Debug, ptr::null};

use crate::structs::{HasData, Mapping, MirrorTrait, ValueType};
use evalexpr::*;
use lookups::{HashLookup, LkupHashMap, Lookup};
use regex::Regex;

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
                    for ind in indices {
                        let temp = address.as_bytes();
                        let test = &temp[ind.0..ind.0 + 6];
                        addresses_clean.push(std::str::from_utf8(test).unwrap());
                    }

                    let precompiled = build_operator_tree::<DefaultNumericTypes>(address).unwrap();
                    let mut context = HashMapContext::<DefaultNumericTypes>::new();
                    for ad in addresses_clean {
                        //find value for address and add to context
                        let val_result = sensor_data.data().iter().find(|x| {
                            x.address
                                == u16::from_str_radix(ad.trim_start_matches("0X"), 16).unwrap()
                        });
                        let val;
                        if let Some(res) = val_result {
                            val = res.value;
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
                    for ind in indices {
                        let temp = address.as_bytes();
                        let test = &temp[ind.0..ind.0 + 6];
                        addresses_clean.push(std::str::from_utf8(test).unwrap());
                    }

                    let precompiled = build_operator_tree::<DefaultNumericTypes>(address).unwrap();
                    let mut context = HashMapContext::<DefaultNumericTypes>::new();
                    for ad in addresses_clean {
                        //find value for address and add to context
                        let val_result = sensor_data.data().iter().find(|x| {
                            x.address
                                == u16::from_str_radix(ad.trim_start_matches("0X"), 16).unwrap()
                        });
                        let val;
                        if let Some(res) = val_result {
                            val = res.value;
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
            object.set(field_name, Some(sensor_value as f32)).unwrap();
        }
    }
    object
}

pub fn translate_to_db_object_new<Y: MirrorTrait + Debug>(
    sensor_data: &Y,
    map: &HashMap<String, Mapping>,
) -> HashMap<String, Option<f32>> {
    let mut hashmap: HashMap<String, Option<f32>> = HashMap::new();

    for map_entry in map.iter() {
        let sensor_mapping = map_entry.1;
        let mut sensor_value: Option<f32>;
        match sensor_mapping.mapping_type {
            ValueType::Simple => {
                let modbus_data_option: Option<&Option<f32>> =
                    sensor_data.get(&sensor_mapping.address);
                println!(
                    "Value for {:?}, = {:?}",
                    &sensor_mapping.address,
                    sensor_data.get(&sensor_mapping.address) as Option<&f32>
                );
                if modbus_data_option.is_some() {
                    sensor_value = *modbus_data_option.unwrap();
                } else {
                    sensor_value = None;
                }
            }
            ValueType::Combined => {
                let mut skip = false;
                //Get addresses from mathematical expression
                let address = &sensor_mapping.address;
                let address = address.to_lowercase();

                let addresses_clean: Vec<&str> = parse_address(&address);

                let precompiled = build_operator_tree::<DefaultNumericTypes>(&address).unwrap();
                let mut context = HashMapContext::<DefaultNumericTypes>::new();
                for ad in addresses_clean {
                    // println!("hoi {:?}", sensor_data);
                    //find value for address and add to context
                    let val: Option<f32>;
                    if let Some(res) = sensor_data.get(ad) {
                        val = *res;
                        // println!("found value {:?} for {:?}", val, ad);
                    } else {
                        // println!("VALUE NOT FOUND");
                        val = None;
                    }
                    match val {
                        Some(new) => {
                            // println!("set {:?}", ad);
                            context
                                .set_value(
                                    ad.to_string().to_lowercase(),
                                    Value::from_float(new as f64),
                                )
                                .unwrap();
                        }
                        None => {
                            // println!("skipped");
                            skip = true;
                            sensor_value = None;
                            hashmap.insert(map_entry.0.to_string(), sensor_value);
                            continue;
                        }
                    };
                }
                if skip {
                    continue;
                }
                //calculate result
                //precompiled.
                // println!("{:?}", &context);
                // println!("{:#?}", context);
                match precompiled.eval_float_with_context_mut(&mut context) {
                    Ok(res) => {
                        sensor_value = {
                            // println!("Test {:?}", res);
                            Some(res as f32)
                        }
                    }
                    Err(e) => {
                        sensor_value = None;
                        println!("{:?}", e)
                    }
                }
                // println!("{:?}", sensor_value);
                // let res = precompiled.eval_float_with_context(&context).unwrap() as f32;

                // sensor_value = Some(res);
            }
            ValueType::Bit => {
                //Get addresses from mathematical expression
                let address = &sensor_mapping.address;
                // let mut addresses_clean = Vec::with_capacity(indices.len());
                // for ind in indices {
                //     let temp = address.as_bytes();
                //     let test = &temp[ind.0..ind.0 + 6];
                //     addresses_clean.push(std::str::from_utf8(test).unwrap());
                // }
                let addresses_clean: Vec<&str> = parse_address(&address);

                let precompiled = build_operator_tree::<DefaultNumericTypes>(address).unwrap();
                let mut context = HashMapContext::<DefaultNumericTypes>::new();
                for ad in addresses_clean {
                    //find value for address and add to context
                    let val: Option<i64>;
                    if let Some(res) = sensor_data.get(ad) {
                        let xd: Option<f32> = *res;
                        val = xd.map(|a| a as i64);
                    } else {
                        val = None;
                    }
                    if let Some(new) = val {
                        context
                            .set_value(ad.to_string().to_lowercase(), Value::from_int(new))
                            .unwrap();
                    }
                }
                //calculate result
                match precompiled.eval_float_with_context_mut(&mut context) {
                    Ok(res) => {
                        sensor_value = {
                            // println!("Test {:?}", res);
                            Some(res as f32)
                        }
                    }
                    Err(e) => {
                        sensor_value = None;
                        println!("{:?}", e)
                    }
                }
            }
        }
        //Add hier een add hier een hashmap
        hashmap.insert(map_entry.0.to_string(), sensor_value);
    }
    //Doe hier hashmap in object
    hashmap
}

pub fn translate_to_db_object_hermes(
    sensor_data: &HashMap<String, f32>,
    map: &HashMap<String, Mapping>,
) -> HashMap<String, Option<f32>> {
    let mut hashmap: HashMap<String, Option<f32>> = HashMap::new();

    for map_entry in map.iter() {
        let sensor_mapping = map_entry.1;
        let mut sensor_value: Option<f32>;
        match sensor_mapping.mapping_type {
            ValueType::Simple => {
                let modbus_data_option = sensor_data.get(&sensor_mapping.address);
                if modbus_data_option.is_some() {
                    sensor_value = Some(*modbus_data_option.unwrap());
                } else {
                    sensor_value = None;
                }
            }
            ValueType::Combined => {
                let mut skip = false;
                //Get addresses from mathematical expression
                let address = &sensor_mapping.address;
                let address = address.to_lowercase();

                let addresses_clean: Vec<&str> = parse_address(&address);

                let precompiled = build_operator_tree::<DefaultNumericTypes>(&address).unwrap();
                let mut context = HashMapContext::<DefaultNumericTypes>::new();
                for ad in addresses_clean {
                    // println!("hoi {:?}", sensor_data);
                    //find value for address and add to context
                    let val: Option<f32>;
                    if let Some(res) = sensor_data.get(ad) {
                        val = Some(*res);
                        // println!("found value {:?} for {:?}", val, ad);
                    } else {
                        // println!("VALUE NOT FOUND");
                        val = None;
                    }
                    match val {
                        Some(new) => {
                            // println!("set {:?}", ad);
                            context
                                .set_value(
                                    ad.to_string().to_lowercase(),
                                    Value::from_float(new as f64),
                                )
                                .unwrap();
                        }
                        None => {
                            // println!("skipped");
                            skip = true;
                            sensor_value = None;
                            hashmap.insert(map_entry.0.to_string(), sensor_value);
                            continue;
                        }
                    };
                }
                if skip {
                    continue;
                }
                //calculate result
                //precompiled.
                // println!("{:?}", &context);
                // println!("{:#?}", context);
                match precompiled.eval_float_with_context_mut(&mut context) {
                    Ok(res) => {
                        sensor_value = {
                            // println!("Test {:?}", res);
                            Some(res as f32)
                        }
                    }
                    Err(e) => {
                        sensor_value = None;
                        // println!("{:?}", e)
                    }
                }
                // println!("{:?}", sensor_value);
                // let res = precompiled.eval_float_with_context(&context).unwrap() as f32;

                // sensor_value = Some(res);
            }
            ValueType::Bit => {
                //Get addresses from mathematical expression
                let address = &sensor_mapping.address;
                // let mut addresses_clean = Vec::with_capacity(indices.len());
                // for ind in indices {
                //     let temp = address.as_bytes();
                //     let test = &temp[ind.0..ind.0 + 6];
                //     addresses_clean.push(std::str::from_utf8(test).unwrap());
                // }
                let addresses_clean: Vec<&str> = parse_address(&address);

                let precompiled = build_operator_tree::<DefaultNumericTypes>(address).unwrap();
                let mut context = HashMapContext::<DefaultNumericTypes>::new();
                for ad in addresses_clean {
                    //find value for address and add to context
                    let val: Option<i64>;
                    if let Some(res) = sensor_data.get(ad) {
                        let xd: Option<f32> = Some(*res);
                        val = xd.map(|a| a as i64);
                    } else {
                        val = None;
                    }
                    if let Some(new) = val {
                        context
                            .set_value(ad.to_string().to_lowercase(), Value::from_int(new))
                            .unwrap();
                    }
                }
                //calculate result
                match precompiled.eval_float_with_context_mut(&mut context) {
                    Ok(res) => {
                        sensor_value = {
                            // println!("Test {:?}", res);
                            Some(res as f32)
                        }
                    }
                    Err(e) => {
                        sensor_value = None;
                        // println!("{:?}", e)
                    }
                }
            }
        }
        //Add hier een add hier een hashmap
        hashmap.insert(map_entry.0.to_string(), sensor_value);
    }
    //Doe hier hashmap in object
    hashmap
}

pub fn parse_address(input: &str) -> Vec<&str> {
    let char_array = input.chars().collect::<Vec<char>>();
    let mut iter = char_array.as_slice().windows(2).enumerate().peekable();

    let mut indexes = vec![];
    let mut addresses = vec![];

    while iter.peek().is_some() {
        let xd = iter.next();
        let xdd = xd.map(|a| a.1);
        match xdd {
            Some(&['m', 'b']) => indexes.push(xd.unwrap().0),
            Some(&['i', 'n']) => indexes.push(xd.unwrap().0), //Also search for input_ or holding_
            Some(&['h', 'o']) => indexes.push(xd.unwrap().0),
            Some(&['p', '_']) => indexes.push(xd.unwrap().0),
            Some(_) => (),
            None => unreachable!(),
        };
    }

    for idx in indexes {
        let subset = input[idx..].chars();
        // println!("{:?}", subset);
        let end = subset
            .take_while(|c| c.is_ascii_lowercase() || *c == '_' || c.is_digit(10))
            .count();
        // println!("{:?}, {:?}", idx, end + idx);
        addresses.push(&input[idx..end + idx]);
    }

    addresses
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
            object.set(field_name, Some(sensor_value as f32)).unwrap();
        }
    }
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

    let found_value: f32 = *source.get(source_field).unwrap();
    let target_field = map.get(source_field).unwrap();
    target.set(&target_field.address, found_value);

    target
}

pub fn find_single_value<Y: MirrorTrait>(
    source: &Y,
    field_name: &str,
    path: &str,
) -> Result<std::option::Option<f32>, Box<dyn std::error::Error>> {
    let mut rdr = match csv::ReaderBuilder::new().has_headers(false).from_path(path) {
        Ok(res) => res,
        Err(_) => return Err(("error reading file in find_single_vale").into()),
    };

    let mut map = LkupHashMap::new(HashLookup::with_multi_keys(), |key: &Mapping| {
        key.address.to_string()
    });

    for result in rdr.deserialize() {
        let (key, mapping): (String, Mapping) = result.unwrap();
        map.insert(key.to_lowercase(), mapping);
    }

    let target_field = map.get(field_name).unwrap();

    if let Some(found_value) = source.get(&target_field.address) {
        Ok(*found_value)
    } else {
        Ok(Some(0.0))
    }
}
