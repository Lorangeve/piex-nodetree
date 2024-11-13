use piex_nodetree::{
    registry_tree::{RegistriesItem, RegistriesTree},
    tree::MakeTree,
};
use std::env;

fn main() {
    let key_path = dbg!(env::args().nth(1).unwrap_or(String::from(r"HKLM\SOFTWARE\Adobe")));
    let _is_print_json = dbg!(env::args().nth(2).map(|x| x == "json").unwrap_or(false));

    let Ok(tree) = RegistriesTree::make_tree(RegistriesItem::new(key_path)) else {
        return;
    };

    // let json_str = tree_node.to_pretty_json().unwrap();
    let json_str = tree.to_pretty_json();
    println!("{}", json_str);
}
