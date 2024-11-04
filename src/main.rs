use indextree::{Arena, NodeId};
use serde::{de::value, Serialize};
use serde_json;
use std::{env, io::Read, os::windows::raw::SOCKET};
use windows_registry::{Key, Type, *};

#[derive(Serialize)]
struct TreeNode<T: Serialize> {
    data: T,
    children: Vec<TreeNode<T>>,
}

// 递归地将 indextree 转换为 TreeNode 结构
impl<T> TreeNode<T>
where
    T: Serialize + Clone,
{
    // 自定义转换函数，用于将 NodeId 转换为 TreeNode
    fn from_node_id(node: NodeId, arena: &Arena<T>) -> Option<TreeNode<T>> {
        // 获取节点数据并克隆
        let data = arena[node].get().clone();

        // 递归构建子节点列表
        let children: Vec<TreeNode<T>> = node
            .children(arena)
            .filter_map(|child| TreeNode::from_node_id(child, arena))
            .collect();

        Some(TreeNode { data, children })
    }

    fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

#[cfg(windows)]
#[derive(Serialize, Clone)]
pub struct RegistriesItem {
    pub key_path: String,
    pub value_list: Vec<RegistriesType>,
}

#[derive(Serialize, Clone, Debug)]
pub enum RegistriesType {
    U32(u32),
    U64(u64),
    String(String),
    HString,
    MultiString(Vec<String>),
}

fn main() {
    // let _ = registry_demo();
    // indextree_demo();
    dbg!(registry_demo2());
}

fn registry_demo2() -> windows_registry::Result<()> {
    let mut arena = Arena::new();

    let key_path = env::args().nth(1).unwrap_or(String::from(r"HKLM\SOFTWARE"));

    let root = arena.new_node(RegistriesItem {
        key_path,
        value_list: vec![],
    });

    let key_path = &arena[root].get().key_path.split_once("\\");
    let (rootkey_name, key_path) = key_path.unwrap();

    print_all_registry_key_values(
        map_root_keyname_to_registry_key(rootkey_name).unwrap(),
        &key_path,
        0,
    )?;

    Ok(())
}

fn print_all_registry_key_values<'a>(
    key: &'a Key,
    path: &'a str,
    indent: i32,
) -> windows_registry::Result<()> {
    let key = key.open(path)?;

    let children_keys: Vec<String> = key.keys()?.collect();

    let tab_repeat = "\t".repeat(indent as usize);
    if children_keys.len() == 0 {
        for (key_name, key_value) in key.values()? {
            if let Some(value) = get_value(&key_value) {
                println!("{tab_repeat}::{}:\t{:?}", &key_name, value);
            } else {
                // println!("{tab_repeat}::{}:\t{:?}", &key_name, key.get_hstring(&key_name).unwrap());
                println!("{:?}", key_value.clone().trim_ascii_end())
            }
        }
    } else {
        for path in children_keys {
            println!("{tab_repeat}{}:", path);

            let indent = indent + 1;
            if let Err(e) = print_all_registry_key_values(&key, path.as_str(), indent) {
                println!("{tab_repeat}\tError: {}", e);
            }
        }
    }

    Ok(())
}

fn get_value(value: &Value) -> Option<RegistriesType> {
    let r = match value.ty() {
        // FIXME: 目前解析 String 类型时可能会出现问题（部分场景下中文 unicode 字符错误）
        // Type::String => value.clone().try_into().ok().map(RegistriesType::String),
        Type::String => Some(RegistriesType::String(value.clone().try_into().unwrap())),
        Type::MultiString => value.clone().try_into().ok().map(RegistriesType::MultiString),
        Type::U32 => value.clone().try_into().ok().map(RegistriesType::U32),
        Type::U64 => value.clone().try_into().ok().map(RegistriesType::U64),
        _ => None,
    };

    r
}

fn map_root_keyname_to_registry_key<'a>(rootkey_name: impl AsRef<str>) -> Option<&'a Key> {
    let rootkey_name = rootkey_name.as_ref().to_ascii_uppercase();

    match rootkey_name.as_str() {
        "HKLM" | "HKEY_LOCAL_MACHINE" => Some(LOCAL_MACHINE),
        "HKCU" | "HKEY_CURRENT_USER" => Some(CURRENT_USER),
        "HKU" | "HKEY_USERS" => Some(USERS),
        _ => None,
    }
}

fn registry_demo() -> windows_registry::Result<()> {
    let mut arena = Arena::new();

    let root = arena.new_node(String::from("HKLM\\CUR"));

    let key = LOCAL_MACHINE.open("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
    for sub_key_name in key.keys()? {
        let new_node = arena.new_node(sub_key_name.clone());
        root.append(new_node, &mut arena);
        println!("{}", sub_key_name);

        let Ok(sub_key) = key.open(&sub_key_name) else {
            continue;
        };

        for sub_key_name in sub_key.keys().unwrap() {
            new_node.append(arena.new_node(sub_key_name.clone()), &mut arena);
            println!("\t{}", sub_key_name);
        }
    }

    println!("{}", root.debug_pretty_print(&arena));

    // let tree_json = TreeNode::from_node_id(root, &arena);
    // let json_str = serde_json::to_string_pretty(&tree_json).unwrap();
    let tree_node = TreeNode::from_node_id(root, &arena).unwrap();
    let json_str = tree_node.to_json().unwrap();
    println!("{}", json_str);

    Ok(())
}

fn indextree_demo() {
    // 初始化 Arena 和节点
    let mut arena = Arena::new();

    // 创建根节点和子节点
    let root = arena.new_node("root".to_string());
    let child1 = arena.new_node("child1".to_string());
    let child2 = arena.new_node("child2".to_string());
    let child3 = arena.new_node("child1".to_string());

    // 组织树结构
    root.append(child1, &mut arena);
    root.append(child2, &mut arena);
    root.append(child3, &mut arena);

    // 转换为 JSON 格式
    // let tree_json = node_to_treenode(root, &arena);
    let tree_node = TreeNode::from_node_id(root, &arena).unwrap();
    // let json_str = serde_json::to_string_pretty(&tree_node).unwrap();
    let json_str = tree_node.to_json().unwrap();

    println!("arena = {:?}", &arena);
    println!("arena = {}", root.debug_pretty_print(&arena));
    println!("{}", json_str);
}
