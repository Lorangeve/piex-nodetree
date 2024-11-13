use piex_nodetree::{registry_tree::*, tree::MakeTree};
use std::env;

macro_rules! reg {
    ($lit:literal) => {
        RegistriesTree::make_tree(RegistriesItem::new($lit)).unwrap().get($lit)
    };
    ($path:literal -> $item:literal) => {
        RegistriesTree::make_tree(RegistriesItem::new($path))
            .unwrap()
            .get_with($path, $item)
    };
}

fn main() {
    let key_path = dbg!(env::args()
        .nth(1)
        .unwrap_or(String::from(r"HKLM\SOFTWARE\Huorong")));
    let _is_print_json = dbg!(env::args().nth(2).map(|x| x == "json").unwrap_or(false));

    let Ok(tree) = RegistriesTree::make_tree(RegistriesItem::new(key_path)) else {
        return;
    };

    println!("{:?}", tree.get("HKLM\\Software\\Huorong\\Sysdiag\\WSC"));

    println!("{:?}", tree.get_with("HKEY_LOCAL_MACHINE\\Software\\Huorong\\Sysdiag\\app", "UpdateLink"));

    println!("{:?}", tree.get("HKEY_LOCAL_MACHINE\\Software\\Huorong\\SysClean").unwrap().get("autoscan"));

    println!("{:?}", reg!("HKEY_LOCAL_MACHINE\\Software\\Huorong\\SysClean"));
    println!("{:?}", reg!("HKEY_LOCAL_MACHINE\\Software\\Huorong\\SysClean" -> "autoscan"));
    
    println!("{}", tree);
}