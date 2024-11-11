use piex_nodetree::{
    registry_tree::*,
    tree::MakeTree,
};
use std::env;

fn main() {
    let key_path = dbg!(env::args().nth(1).unwrap_or(String::from(r"HKLM\SOFTWARE\Huorong")));
    let _is_print_json = dbg!(env::args().nth(2).map(|x| x == "json").unwrap_or(false));

    let Ok(tree_node) = RegistriesTree::make_tree(RegistriesItem::new(key_path)) else {
        return;
    };

}