use indextree::{Arena, NodeId};
use serde::Serialize;
use serde_json;
use windows_registry::*;

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
    pub value: Option<RegistriesChiren>,
}

impl RegistriesItem {
    fn from<T: AsRef<str>>(path: T) -> Self {
        let path = path.as_ref();

        RegistriesItem {
            key_path: path.to_string(),
            value: None,
        }
    }
}

fn fill_registry_arena(
    item: RegistriesItem,
    arena: Arena<RegistriesItem>,
) -> Arena<RegistriesItem> {
    let path = item.key_path;
    let path_segments = path.split("\\");

    // 如果注册表路径带有根节点名称，则打开相应的根节点
    if let Some(root_key_name) = path_segments.next() {
        let path: Vec<&str> = path_segments.collect();
        let path = path.join("\\");

        if let Some(root_key ) = match root_key_name {
            "HKLM" | "HKEY_LOCAL_MACHINE" => Some(LOCAL_MACHINE),
            "HKCU" | "HKEY_CURRENT_USER" => Some(CURRENT_USER),
            _ => None,
        } {
            match root_key.open(path) {
                Ok(key) => { key },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }

    }


    arena
}

#[cfg(windows)]
#[derive(Serialize, Clone)]
pub enum RegistriesChilren {
    Item(RegistriesItem),
    Value(RegistriesType),
}

#[derive(Serialize, Clone)]
pub enum RegistriesType {}

fn main() {
    // let _ = registry_demo();
    // indextree_demo();
    registry_demo2();
}

fn registry_demo2() -> windows_registry::Result<()> {
    Ok(())
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
