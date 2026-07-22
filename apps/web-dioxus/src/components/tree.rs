use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeNode>,
}

#[component]
pub fn Tree(
    nodes: Vec<TreeNode>,
    selected: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "ant-tree",
            for node in &nodes {
                TreeNodeItem { node: node.clone(), selected: selected.clone(), on_select, level: 0, last: is_last(node, &nodes) }
            }
        }
    }
}

fn is_last(node: &TreeNode, siblings: &[TreeNode]) -> bool {
    siblings.last().map(|n| n.id == node.id).unwrap_or(false)
}

#[component]
fn TreeNodeItem(
    node: TreeNode,
    selected: Option<String>,
    on_select: EventHandler<String>,
    level: u32,
    last: bool,
) -> Element {
    let mut expanded = use_signal(|| false);
    let has_children = !node.children.is_empty();
    let is_selected = selected.as_ref() == Some(&node.id);

    rsx! {
        div {
            // Tree node row
            div {
                class: format!("ant-tree-node {} {}",
                    if is_selected { "ant-tree-node-selected" } else { "" },
                    if last && !has_children { "ant-tree-node-leaf-last" } else { "" },
                ),
                style: "padding-left: {level * 24}px",
                onclick: move |e| {
                    e.stop_propagation();
                    if has_children { expanded.toggle(); }
                    on_select.call(node.id.clone());
                },
                // Indent lines
                for _ in 0..level {
                    span { class: "ant-tree-indent" }
                }
                // Expand icon
                span {
                    class: format!("ant-tree-switcher {} {}",
                        if has_children { "ant-tree-switcher-has-children" } else { "" },
                        if has_children && expanded() { "ant-tree-switcher-open" } else { "" },
                    ),
                    if has_children {
                        span { class: "ant-tree-switcher-icon",
                            if expanded() { "▾" } else { "▸" }
                        }
                    }
                }
                // Node icon
                span { class: "ant-tree-node-icon",
                    if has_children {
                        if expanded() { "📂" } else { "📁" }
                    } else {
                        if is_selected { "📄" } else { "📄" }
                    }
                }
                // Label
                span { class: "ant-tree-title", "{node.label}" }
            }
            // Children
            if has_children && expanded() {
                for (i, child) in node.children.iter().enumerate() {
                    TreeNodeItem {
                        node: child.clone(),
                        selected: selected.clone(),
                        on_select,
                        level: level + 1,
                        last: i == node.children.len() - 1,
                    }
                }
            }
        }
    }
}
