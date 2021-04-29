pub mod recorders;

use types;
use std::f64::consts::PI;

pub fn toml_table_to_params(table: toml::Value) -> types::Params {
    match table {
        toml::Value::Table(m) => {
            let initial_radius = m.get("initial_radius").unwrap().as_float().unwrap();
            let initial_thickness = m.get("initial_thickness").unwrap().as_float().unwrap();
            let initial_area = PI * (initial_radius.powf(2.0) - (initial_radius - initial_thickness).powf(2.0));
            types::Params {
                initial_thickness: initial_thickness,
                initial_radius: initial_radius,
                initial_gray_matter_area: initial_area,
                initial_num_points: m.get("initial_num_points").unwrap().as_integer().unwrap() as usize,
                initial_temperature: m.get("initial_temperature").unwrap().as_float().unwrap(),
                compression_factor: m.get("compression_factor").unwrap().as_float().unwrap(),
                softness_factor: m.get("softness_factor").unwrap().as_float().unwrap(),
                how_smooth: m.get("how_smooth").unwrap().as_integer().unwrap() as usize,
                node_addition_threshold: m.get("node_addition_threshold").unwrap().as_float().unwrap(),
                node_deletion_threshold: m.get("node_deletion_threshold").unwrap().as_float().unwrap(),
                low_high: (
                    m.get("low_high").unwrap().as_array().unwrap()[0].as_float().unwrap(),
                    m.get("low_high").unwrap().as_array().unwrap()[1].as_float().unwrap(),
                ),
                recorders: m
                    .get("recorders")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| String::from(x.as_str().unwrap()))
                    .collect(),
            }
        }
        _ => panic!("No key-value table found in parameters.toml"),
    }
}