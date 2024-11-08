use piex_nodetree::{registry_tree::*, tree::*};
use std::env;

fn main() {
    let key_path = dbg!(env::args().nth(1).unwrap_or(String::from(r"HKLM\SOFTWARE")));
    let _is_print_json = dbg!(env::args().nth(2).map(|x| x == "json").unwrap_or(false));

    if let Ok(tree_node) = RegistriesItem::make_tree(RegistriesItem::new(key_path)) {
        // let json_str = tree_node.to_pretty_json().unwrap();
        let json_str = tree_node.to_json();
        println!("{:?}", json_str);
    }
}
