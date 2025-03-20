use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use serde_pickle::{HashableValue, Value};


fn load_list<const S: usize>(value: &Value) -> [f64; S] {
    let mut result: [f64; S] = [0.0; S];
    if let Value::List(list) = value {
        for i in 0..S
        {
            if let Value::F64(f) = &list[i] {
                result[i] = *f as f64
            } else {
                panic!("Not float in list")
            }
        }
    } else {
        panic!("Not list")
    }
    result
}

fn load_list_of_lists<const X: usize, const Y: usize>(value: &Value) -> Vec<[f64; X]> {
    let mut result: Vec<[f64; X]> = Vec::new();
    if let Value::List(list) = value {
        for i in 0..Y
        {
            result.push(load_list(&list[i]));
        }
    } else {
        panic!("Not list")
    }
    result
}

pub fn load_state<
    const X0: usize,
    const X1: usize,
    const X2: usize>(path: &String) -> Option<(Vec<[f64; X0]>, [f64; X1], Vec<[f64; X1]>, [f64; X2], Vec<[f64; X2]>, [f64; 1])> {
    return if Path::new(path).exists() {
        let pickle = fs::read(path).unwrap();
        let deserialized: BTreeMap<HashableValue, Value> = serde_pickle::from_slice(&*pickle, Default::default()).unwrap();
        Some((load_list_of_lists::<X0, X1>(deserialized.get(&HashableValue::String(String::from("l1w"))).unwrap()),
              load_list(deserialized.get(&HashableValue::String(String::from("l1b"))).unwrap()),
              load_list_of_lists::<X1, X2>(deserialized.get(&HashableValue::String(String::from("l2w"))).unwrap()),
              load_list(deserialized.get(&HashableValue::String(String::from("l2b"))).unwrap()),
              load_list_of_lists::<X2, 1>(deserialized.get(&HashableValue::String(String::from("l3w"))).unwrap()),
              load_list(deserialized.get(&HashableValue::String(String::from("l3b"))).unwrap())))
    } else {
        None
    };
}

#[test]
fn test() {
    println!("{:?}", std::env::current_dir());
    let nnue_wb =load_state::<{ 2 * 40960}, 256, 32>(&String::from("resources\\halfkp2-cp-50.pth_wb.pt")).unwrap();
    let pickle = fs::read(&String::from("resources\\halfkp2-cp-50.pth_wb.pt")).unwrap();
    let deserialized: BTreeMap<HashableValue, Value> = serde_pickle::from_slice(&*pickle, Default::default()).unwrap();
    let list: [f64; 256] = load_list(deserialized.get(&HashableValue::String(String::from("l1b"))).unwrap());
    let listlist: Vec<[f64; 2 * 40960]> = load_list_of_lists::<81920, 256>(deserialized.get(&HashableValue::String(String::from("l1w"))).unwrap());
}