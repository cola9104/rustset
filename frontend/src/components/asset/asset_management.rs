use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaServer, FaDesktop, FaLaptop
};

/// 资产数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub id: i32,
    pub name: String,
    pub asset_type: String,
    pub ip_address: String,
    pub status: String,
    pub location: String,
    pub created_at: String,
}

/// 资产管理页面
#[allow(non_snake_case)]
pub fn AssetManagement() -> Element {
    let mut assets = use_signal(|| vec![
        Asset {
            id: 1,
            name: "服务器-01".to_string(),
            asset_type: "服务器".to_string(),
            ip_address: "192.168.1.10".to_string(),
            status: "在线".to_string(),
            location: "机房A-机柜1".to_string(),
            created_at: "2024-01-15".to_string(),
        },
        Asset {
            id: 2,
            name: "服务器-02".to_string(),
            asset_type: "服务器".to_string(),
            ip_address: "192.168.1.11".to_string(),
            status: "在线".to_string(),
            location: "机房A-机柜2".to_string(),
            created_at: "2024-01-16".to_string(),
        },
        Asset {
            id: 3,
            name: "工作站-01".to_string(),
            asset_type: "工作站".to_string(),
            ip_address: "192.168.2.100".to_string(),
            status: "离线".to_string(),
            location: "办公区".to_string(),
            created_at: "2024-01-20".to_string(),
        },
    ]);

    let mut search_query = use_signal(String::new);
    let mut show_add_modal = use_signal(|| false);
    let mut editing_asset = use_signal(|| None::<Asset>);

    // 计算统计数据
    let total_count = assets.read().len() as i32;
    let server_count = assets.read().iter().filter(|a| a.asset_type == "服务器").count() as i32;
    let workstation_count = assets.read().iter().filter(|a| a.asset_type == "工作站").count() as i32;
    let online_count = assets.read().iter().filter(|a| a.status == "在线").count() as i32;

    // 过滤资产
    let filtered_assets: Vec<Asset> = assets.read()
        .iter()
        .filter(|asset| {
            let query = search_query.read().to_lowercase();
            query.is_empty() ||
            asset.name.to_lowercase().contains(&query) ||
            asset.ip_address.contains(&query)
        })
        .cloned()
        .collect();

    let is_empty = filtered_assets.is_empty();

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "资产管理" }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加资产" }
                }
            }

            // 搜索栏
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    Icon { icon: FaMagnifyingGlass, width: 20, height: 20 }
                    input {
                        r#type: "text",
                        class: "ml-2 flex-1 border-0 focus:outline-none",
                        placeholder: "搜索资产名称或IP地址...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                // 总资产
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaServer, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总资产" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                // 服务器
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaDesktop, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "服务器" }
                            p { class: "text-xl font-bold text-gray-800", {server_count.to_string()} }
                        }
                    }
                }
                // 工作站
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-yellow-500",
                            Icon { icon: FaLaptop, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "工作站" }
                            p { class: "text-xl font-bold text-gray-800", {workstation_count.to_string()} }
                        }
                    }
                }
                // 在线
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-600",
                            Icon { icon: FaServer, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "在线" }
                            p { class: "text-xl font-bold text-gray-800", {online_count.to_string()} }
                        }
                    }
                }
            }

            // 资产列表表格
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "名称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "类型" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "IP地址" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "位置" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for asset in filtered_assets {
                            tr { class: "hover:bg-gray-50",
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    div { class: "flex items-center",
                                        div { class: "flex-shrink-0 h-10 w-10 bg-gray-100 rounded-full flex items-center justify-center",
                                            Icon { icon: FaServer, width: 20, height: 20 }
                                        }
                                        div { class: "ml-4",
                                            div { class: "text-sm font-medium text-gray-900", {asset.name.clone()} }
                                        }
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800",
                                        {asset.asset_type.clone()}
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {asset.ip_address.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span {
                                        class: if asset.status == "在线" {
                                            "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                        } else {
                                            "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                        },
                                        {asset.status.clone()}
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {asset.location.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                    button {
                                        class: "text-blue-600 hover:text-blue-900 mr-3",
                                        onclick: {
                                            let asset = asset.clone();
                                            move |_| editing_asset.set(Some(asset.clone()))
                                        },
                                        Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                    }
                                    button {
                                        class: "text-red-600 hover:text-red-900",
                                        onclick: {
                                            let asset_id = asset.id;
                                            move |_| {
                                                let mut list = assets.write();
                                                list.retain(|a| a.id != asset_id);
                                            }
                                        },
                                        Icon { icon: FaTrash, width: 16, height: 16 }
                                    }
                                }
                            }
                        }
                    }
                }

                // 空状态
                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的资产"
                    }
                }
            }
        }

        // 添加资产模态框
        if *show_add_modal.read() {
            AddAssetModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |new_asset: Asset| {
                    assets.write().push(new_asset);
                    show_add_modal.set(false);
                }
            }
        }

        // 编辑资产模态框
        if let Some(asset) = editing_asset.read().clone() {
            EditAssetModal {
                asset: asset.clone(),
                on_close: move |_| editing_asset.set(None),
                on_save: {
                    let mut editing_asset_signal = editing_asset.clone();
                    move |updated: Asset| {
                        let mut list = assets.write();
                        if let Some(a) = list.iter_mut().find(|a| a.id == updated.id) {
                            *a = updated;
                        }
                        editing_asset_signal.set(None);
                    }
                }
            }
        }
    }
}

/// 添加资产模态框
#[component]
fn AddAssetModal(on_close: EventHandler<()>, on_save: EventHandler<Asset>) -> Element {
    let mut name = use_signal(String::new);
    let mut asset_type = use_signal(|| "服务器".to_string());
    let mut ip_address = use_signal(String::new);
    let mut status = use_signal(|| "在线".to_string());
    let mut location = use_signal(String::new);

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加资产" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "输入资产名称",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: asset_type,
                            onchange: move |e| asset_type.set(e.value()),
                            option { value: "服务器", "服务器" }
                            option { value: "工作站", "工作站" }
                            option { value: "网络设备", "网络设备" }
                            option { value: "存储设备", "存储设备" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "IP地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: 192.168.1.1",
                            value: ip_address,
                            oninput: move |e| ip_address.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "在线", "在线" }
                            option { value: "离线", "离线" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "位置" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: 机房A-机柜1",
                            value: location,
                            oninput: move |e| location.set(e.value()),
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| {
                            let new_asset = Asset {
                                id: chrono::Utc::now().timestamp() as i32,
                                name: name.read().clone(),
                                asset_type: asset_type.read().clone(),
                                ip_address: ip_address.read().clone(),
                                status: status.read().clone(),
                                location: location.read().clone(),
                                created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            };
                            on_save.call(new_asset);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 编辑资产模态框
#[component]
fn EditAssetModal(asset: Asset, on_close: EventHandler<()>, on_save: EventHandler<Asset>) -> Element {
    let mut name = use_signal(|| asset.name.clone());
    let mut asset_type = use_signal(|| asset.asset_type.clone());
    let mut ip_address = use_signal(|| asset.ip_address.clone());
    let mut status = use_signal(|| asset.status.clone());
    let mut location = use_signal(|| asset.location.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑资产" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: asset_type,
                            onchange: move |e| asset_type.set(e.value()),
                            option { value: "服务器", "服务器" }
                            option { value: "工作站", "工作站" }
                            option { value: "网络设备", "网络设备" }
                            option { value: "存储设备", "存储设备" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "IP地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: ip_address,
                            oninput: move |e| ip_address.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "在线", "在线" }
                            option { value: "离线", "离线" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "位置" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: location,
                            oninput: move |e| location.set(e.value()),
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: {
                            let asset_id = asset.id;
                            let created_at = asset.created_at.clone();
                            move |_| {
                                let updated = Asset {
                                    id: asset_id,
                                    name: name.read().clone(),
                                    asset_type: asset_type.read().clone(),
                                    ip_address: ip_address.read().clone(),
                                    status: status.read().clone(),
                                    location: location.read().clone(),
                                    created_at: created_at.clone(),
                                };
                                on_save.call(updated);
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
