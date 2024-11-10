use std::{clone, error::Error};

pub use indextree::{Arena, NodeId};
use serde::Serialize;
use serde_json;

#[derive(Serialize)]
pub struct TreeNode<T: Serialize> {
    data: T,
    children: Vec<TreeNode<T>>,
}

// 递归地将 indextree 转换为 TreeNode 结构
impl<T> TreeNode<T>
where
    T: Serialize + Clone,
{
    // 自定义转换函数，用于将 NodeId 转换为 TreeNode
    pub fn from_node_id(node: NodeId, arena: &Arena<T>) -> Option<TreeNode<T>> {
        // 获取节点数据并克隆
        // let data = arena[node].get().clone();
        let data = arena.get(node)?.get().to_owned();

        // 递归构建子节点列表
        let children: Vec<TreeNode<T>> = node
            .children(arena)
            .filter_map(|child| TreeNode::from_node_id(child, arena))
            .collect();

        Some(TreeNode { data, children })
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self)
    }
}

// 定义 MakeTree 特征，使用泛型参数 T
pub trait MakeTree<T> {
    type Error;

    fn make_tree(speed: T) -> Result<TreeNode<T>, Self::Error>
    where
        T: Serialize + Clone,
        Self::Error: Error;
}
