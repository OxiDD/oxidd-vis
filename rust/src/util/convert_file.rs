use std::fs::{write, File};
use std::io::prelude::*;

fn main() {
    let path = r"C:\Users\tarva\Documents\Projects\Github\BDD-viz\data\";
    let name = "b02";
    let file_name = format!("{path}{name}.dddmp");
    let file_out_name = format!("{path}{name}.g");

    let mut file = File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let node_text = &contents[contents.find(".nodes").unwrap()..contents.find(".end").unwrap()];
    let nodes_data = node_text.split("\n");
    let out = nodes_data
        .filter_map(|node| {
            let parts = node.trim().split(" ").collect::<Vec<&str>>();
            if parts.len() == 4 && parts[1].as_bytes()[0].is_ascii_digit() {
                return Some(format!(
                    "{}->{}, {}->{}",
                    parts[0], parts[2], parts[0], parts[3]
                ));
            }
            None
        })
        .collect::<Vec<String>>()
        .join(",");
    write(file_out_name, out).unwrap();
    println!("hoi");
}
