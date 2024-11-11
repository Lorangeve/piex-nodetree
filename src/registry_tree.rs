use std::{cmp, collections::HashMap, path::PathBuf};

pub use crate::tree::MakeTree;
use crate::tree::*;

use indextree::{Arena, NodeId};
use serde::Serialize;
use windows_registry::{Key, Type, *};

#[derive(Debug)]
pub struct RegistriesTree {
    root: NodeId,
    arena: Arena<RegistriesItem>,
    dict: HashMap<String, NodeId>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RegistriesItem {
    key_path: String,
    value_map: HashMap<String, Option<RegistriesData>>,
}

impl std::fmt::Display for RegistriesItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 打印 key_path
        writeln!(f, "key_path: {}", self.key_path)?;

        // 遍历 value_map 并打印每个键值对
        for (key, value) in &self.value_map {
            match value {
                Some(data) => {
                    if let RegistriesData::Bytes(bytes) = data {
                        let edge = bytes.len().saturating_sub(1);
                        let endpos = cmp::min(edge, 10);
                        let is_summary = endpos < edge;

                        writeln!(
                            f,
                            "\t- {}:\t Bytes({}{})",
                            key,
                            bytes
                                .iter()
                                .take(endpos)
                                .map(|&b| format!("0x{:02X}", b))
                                .collect::<Vec<String>>()
                                .join(","),
                            if is_summary { "..." } else { "" }
                        )?
                    } else {
                        writeln!(f, "\t- {}:\t{:?}", key, data)?
                    }
                }
                None => writeln!(f, "\t- {}: None", key)?,
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for RegistriesTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.to_pretty_tree())?;

        Ok(())
    }
}

impl RegistriesItem {
    pub fn new(key_path: String) -> Self {
        RegistriesItem {
            key_path,
            value_map: HashMap::new(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum RegistriesData {
    U32(u32),
    U64(u64),
    String(String),
    MultiString(Vec<String>),
    Bytes(Vec<u8>),
    Raw(u32),
}

impl MakeTree<RegistriesItem> for RegistriesTree {
    fn make_tree(speed: RegistriesItem) -> std::result::Result<Self, MakeTreeError> {
        let mut arena = Arena::new();

        let root = arena.new_node(speed);
        let key_path = arena.get(root).unwrap().get().key_path.to_owned();

        if let Some((rootkey_name, key_path)) = key_path.split_once("\\") {
            fill_regkey_to_arena(
                root_key(rootkey_name).unwrap_or(LOCAL_MACHINE),
                key_path,
                &root,
                &mut arena,
            )
            .map_err(|e| MakeTreeError(e.message()))?;

            let dict = make_search_dict(&root, &arena);

            Ok(RegistriesTree { root, arena, dict })
        } else {
            Err(MakeTreeError("注册表路径错误！".to_owned()))
        }
    }
}

fn make_search_dict(node_id: &NodeId, arena: &Arena<RegistriesItem>) -> HashMap<String, NodeId> {
    let mut dict: HashMap<String, NodeId> = HashMap::new();

    let mut key_path = PathBuf::from(arena.get(*node_id).unwrap().get().key_path.to_owned());
    for child_node_id in node_id.children(arena) {
        println!("{:?}", child_node_id);
        println!("{:?}", arena.get(child_node_id).unwrap().get().key_path);
        key_path.push(arena.get(child_node_id).unwrap().get().key_path.to_owned());
        dict.insert(key_path.to_str().unwrap().to_owned(), child_node_id);
        key_path.pop();
    }

    println!("dict: {:#?}", dict);

    dict
}

impl RegistriesTree {
    pub fn sub_tree(&self, node_id: NodeId) -> Option<TreeNode<RegistriesItem>> {
        TreeNode::from_node_id(&node_id, &self.arena)
    }

    fn make_search_dict(&self) {
        // for node in self.arena.iter() {
        //     println!("{:?}", self.arena.get_node_id(node));
        //     println!("{:?}", node.get().key_path);
        // }
        println!("dict: {:#?}", self.dict);

        println!(
            "dict: {}",
            TreeNode::from_node_id(&self.dict["HKLM"], &self.arena)
                .unwrap()
                .to_json()
        );
    }

    pub fn to_json(&self) -> String {
        TreeNode::from_node_id(&self.root, &self.arena)
            .unwrap()
            .to_json()
    }

    pub fn to_pretty_json(&self) -> String {
        TreeNode::from_node_id(&self.root, &self.arena)
            .unwrap()
            .to_pretty_json()
    }

    pub fn to_pretty_tree(&self) -> String {
        let root = self.root;
        format!("{}", root.debug_pretty_print(&self.arena))
    }
}

fn fill_regkey_to_arena<'a>(
    key: &'a Key,
    path: impl AsRef<str>,
    node_id: &NodeId,
    arena: &mut Arena<RegistriesItem>,
) -> Result<()> {
    let key = key.open(&path)?;
    let children_keys: Vec<String> = key.keys()?.collect();

    let value_map = &mut arena[*node_id].get_mut().value_map;
    for (key_name, key_value) in key.values()? {
        let value = key_value.try_into().ok();
        value_map.insert(key_name, value);
    }

    for children_key_name in children_keys {
        let child_node = arena.new_node(RegistriesItem {
            key_path: children_key_name.clone(),
            value_map: HashMap::new(),
        });

        node_id.append(child_node, arena);

        if let Err(e) = fill_regkey_to_arena(&key, &children_key_name, &child_node, arena) {
            eprintln!("{}, Error: {}", &children_key_name, e);
        }
    }

    Ok(())
}

impl TryFrom<Value> for RegistriesData {
    type Error = windows_result::Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value.ty() {
            Type::String => Ok(RegistriesData::String(decode_utf16_lossy(value.as_wide()))),
            Type::MultiString => value.clone().try_into().map(RegistriesData::MultiString),
            Type::ExpandString => Ok(RegistriesData::String(decode_utf16_lossy(value.as_wide()))),
            Type::U32 => value.try_into().map(RegistriesData::U32),
            Type::U64 => value.try_into().map(RegistriesData::U64),
            Type::Bytes => Ok(RegistriesData::Bytes(
                slice_u16_to_u8(value.as_wide()).to_vec(),
            )),
            Type::Other(data) => Ok(RegistriesData::Raw(data)),
        }
    }
}

fn slice_u16_to_u8(slice: &[u16]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * 2) }
}

/// 有损耗地解码 utf16
/// 即，当读取到 `\0` 时直接截断并使用标准库进行 utf16 解码
fn decode_utf16_lossy(utf16_codes: &[u16]) -> String {
    // 查找第一个 `0` 的位置，或者数组的末尾
    let end_pos = utf16_codes
        .iter()
        .position(|&x| x == 0)
        .unwrap_or(utf16_codes.len());

    // 截取到 `end_pos` 并进行 UTF-16 解码
    String::from_utf16(&utf16_codes[..end_pos]).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf16_lossy_test() {
        // Arrange
        let utf16_codes: &[u16] = &[
            0x0041, 0x0075, 0x0074, 0x006F, 0x0064, 0x0065, 0x0073, 0x006B, 0x0020, 0x0041, 0x0075,
            0x0074, 0x006F, 0x0043, 0x0041, 0x0044, 0x0020, 0x0032, 0x0030, 0x0032, 0x0033, 0x0020,
            0x002D, 0x0020, 0x7B80, 0x4F53, 0x4E2D, 0x6587, 0x0020, 0x0028, 0x0053, 0x0069, 0x006D,
            0x0070, 0x006C, 0x0069, 0x0066, 0x0069, 0x0065, 0x0064, 0x0020, 0x0043, 0x0068, 0x0069,
            0x006E, 0x0065, 0x0073, 0x0065, 0x0029, 0x0000, 0x736D, 0x0069, 0xA28C, 0x0FCB, 0x24EB,
            0x9000, 0xDA20, 0x0000,
        ];

        // Act
        let code = decode_utf16_lossy(utf16_codes);
        println!("{}", &code);

        // Assert
        assert_eq!(
            code,
            "Autodesk AutoCAD 2023 - 简体中文 (Simplified Chinese)"
        );
    }
}
