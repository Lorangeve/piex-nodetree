pub mod registry_tree;
mod tree;

use registry_tree::*;
use std::env;
use tree::*;

fn main() {
    let key_path = env::args().nth(1).unwrap_or(String::from(r"HKLM\SOFTWARE"));
    let is_print_json = env::args().nth(2).map(|x| x == "json").unwrap_or(false);

    let tree_node = RegistriesItem::make_tree(key_path).unwrap();

    let json_str = tree_node.to_pretty_json().unwrap();
    println!("{}", json_str);

    // if is_print_json {
    //     let json_str = tree_node.to_pretty_json().unwrap();
    //     println!("{}", json_str);
    // } else {
    //     println!("{:?}", root.debug_pretty_print(&arena));
    // }
}
