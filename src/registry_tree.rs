use crate::tree::*;
use serde::Serialize;
use std::collections::HashMap;
use windows_registry::{Key, Result, Type, *};

#[cfg(windows)]
#[derive(Serialize, Clone, Debug)]
pub struct RegistriesItem {
    pub key_path: String,
    pub value_map: HashMap<String, Option<RegistriesType>>,
}

#[derive(Serialize, Clone, Debug)]
pub enum RegistriesType {
    U32(u32),
    U64(u64),
    String(String),
    MultiString(Vec<String>),
    Bytes,
}

impl MakeTree<String, RegistriesItem> for RegistriesItem {
    type Error = windows_result::Error;

    fn make_tree(key_path: String) -> Result<TreeNode<RegistriesItem>> {
        let mut arena = Arena::new();

        let root = arena.new_node(RegistriesItem {
            key_path: key_path,
            value_map: HashMap::new(),
        });

        let key_path = arena[root].get().key_path.clone();
        let (rootkey_name, key_path) = key_path.split_once("\\").unwrap();

        fill_regkey_to_arena(
            root_key(rootkey_name).unwrap_or(LOCAL_MACHINE),
            key_path,
            &root,
            &mut arena,
        )?;

        let tree_node = TreeNode::from_node_id(root, &arena).unwrap();

        Ok(tree_node)
    }
}

fn fill_regkey_to_arena<'a>(
    key: &'a Key,
    path: impl AsRef<str>,
    node: &NodeId,
    arena: &mut Arena<RegistriesItem>,
) -> Result<()> {
    let key = key.open(&path)?;
    let children_keys: Vec<String> = key.keys()?.collect();

    let value_map = &mut arena[*node].get_mut().value_map;
    for (key_name, key_value) in key.values()? {
        let value = get_value(key_value);
        value_map.insert(key_name, value);
    }

    for children_key_name in children_keys {
        let child_node = arena.new_node(RegistriesItem {
            key_path: children_key_name.clone(),
            value_map: HashMap::new(),
        });

        node.append(child_node, arena);

        if let Err(e) = fill_regkey_to_arena(&key, &children_key_name, &child_node, arena) {
            eprintln!("{}, Error: {}", &children_key_name, e);
        }
    }

    Ok(())
}

fn get_value(value: Value) -> Option<RegistriesType> {
    match value.ty() {
        Type::String => Some(RegistriesType::String(decode_utf16_lossy(value.as_wide()))),
        Type::MultiString => value
            .clone()
            .try_into()
            .ok()
            .map(RegistriesType::MultiString),
        Type::U32 => value.try_into().ok().map(RegistriesType::U32),
        Type::U64 => value.try_into().ok().map(RegistriesType::U64),
        Type::Bytes => Some(RegistriesType::Bytes),
        _ => None,
    }
}

/// 有损耗地解码 utf16
/// 即，当读取到 `\0` 时直接截断并使用标准库进行 utf16 解码
fn decode_utf16_lossy(utf16_codes: &[u16]) -> String {
    let utf16_codes: Vec<u16> = utf16_codes
        .into_iter()
        .cloned()
        .take_while(|&x| x != 0)
        .collect();
    String::from_utf16(utf16_codes.as_slice()).unwrap()
}

/// 将 `key_name` 字符串映射到**注册表键**
fn root_key<'a>(rootkey_name: impl AsRef<str>) -> Option<&'a Key> {
    let rootkey_name = rootkey_name.as_ref().to_ascii_uppercase();

    match rootkey_name.as_str() {
        "HKLM" | "HKEY_LOCAL_MACHINE" => Some(LOCAL_MACHINE),
        "HKCU" | "HKEY_CURRENT_USER" => Some(CURRENT_USER),
        "HKU" | "HKEY_USERS" => Some(USERS),
        _ => None,
    }
}
