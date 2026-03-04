use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaServer, FaDesktop, FaLaptop, FaCloud
};

/// 标签页类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetTab {
    Hardware,  // 硬件设备
    Cloud,     // 云服务资产
}

/// 硬件资产数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct HardwareAsset {
    pub id: i32,
    pub name: String,
    pub asset_type: String,
    pub ip_address: String,
    pub status: String,
    pub location: String,
    pub created_at: String,
}

/// 云服务资产数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct CloudAsset {
    pub id: i32,
    pub name: String,
    pub provider: String,
    pub instance_type: String,
    pub ip_address: String,
    pub status: String,
    pub region: String,
    pub created_at: String,
}

/// 资产管理页面
#[allow(non_snake_case)]
pub fn AssetManagement() -> Element {
    let mut active_tab = use_signal(|| AssetTab::Hardware);

    // 硬件资产数据
    let hardware_assets = use_signal(|| vec![
        HardwareAsset {
            id: 1,
            name: "服务器-01".to_string(),
            asset_type: "服务器".to_string(),
            ip_address: "192.168.1.10".to_string(),
            status: "在线".to_string(),
            location: "机房A-机柜1".to_string(),
            created_at: "2024-01-15".to_string(),
        },
        HardwareAsset {
            id: 2,
            name: "服务器-02".to_string(),
            asset_type: "服务器".to_string(),
            ip_address: "192.168.1.11".to_string(),
            status: "在线".to_string(),
            location: "机房A-机柜2".to_string(),
            created_at: "2024-01-16".to_string(),
        },
        HardwareAsset {
            id: 3,
            name: "工作站-01".to_string(),
            asset_type: "工作站".to_string(),
            ip_address: "192.168.2.100".to_string(),
            status: "离线".to_string(),
            location: "办公区".to_string(),
            created_at: "2024-01-20".to_string(),
        },
    ]);

    // 云服务资产数据
    let cloud_assets = use_signal(|| vec![
        CloudAsset {
            id: 1,
            name: "web-server-01".to_string(),
            provider: "阿里云".to_string(),
            instance_type: "ecs.g6.large".to_string(),
            ip_address: "47.96.123.45".to_string(),
            status: "运行中".to_string(),
            region: "华东1-杭州".to_string(),
            created_at: "2024-01-10".to_string(),
        },
        CloudAsset {
            id: 2,
            name: "api-server-01".to_string(),
            provider: "腾讯云".to_string(),
            instance_type: "S5.MEDIUM4".to_string(),
            ip_address: "119.29.67.89".to_string(),
            status: "运行中".to_string(),
            region: "广州".to_string(),
            created_at: "2024-01-12".to_string(),
        },
        CloudAsset {
            id: 3,
            name: "db-server-01".to_string(),
            provider: "华为云".to_string(),
            instance_type: "s6.xlarge.4".to_string(),
            ip_address: "119.8.123.234".to_string(),
            status: "已停止".to_string(),
            region: "华北区-北京4".to_string(),
            created_at: "2024-01-18".to_string(),
        },
    ]);

    let search_query = use_signal(String::new);
    let mut show_add_modal = use_signal(|| false);

    let is_hardware = *active_tab.read() == AssetTab::Hardware;

    rsx! {
        div { class: "space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "资产管理" }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加资产" }
                }
            }

            // 标签页
            div { class: "bg-white rounded-lg shadow p-1 inline-flex",
                div { class: "flex space-x-1",
                    button {
                        class: format!(
                            "flex items-center px-6 py-2.5 rounded-md text-sm font-medium transition-all duration-200 {}",
                            if is_hardware {
                                "bg-blue-600 text-white shadow-md"
                            } else {
                                "text-gray-600 hover:bg-gray-100"
                            }
                        ),
                        onclick: move |_| active_tab.set(AssetTab::Hardware),
                        Icon { icon: FaServer, width: 16, height: 16, class: "mr-2" }
                        "硬件设备"
                    }
                    button {
                        class: format!(
                            "flex items-center px-6 py-2.5 rounded-md text-sm font-medium transition-all duration-200 {}",
                            if !is_hardware {
                                "bg-blue-600 text-white shadow-md"
                            } else {
                                "text-gray-600 hover:bg-gray-100"
                            }
                        ),
                        onclick: move |_| active_tab.set(AssetTab::Cloud),
                        Icon { icon: FaCloud, width: 16, height: 16, class: "mr-2" }
                        "云服务资产"
                    }
                }
            }

            // 标签页内容
            if is_hardware {
                HardwareAssetsTab {
                    assets: hardware_assets,
                    search_query: search_query,
                    show_add_modal: show_add_modal
                }
            } else {
                CloudAssetsTab {
                    assets: cloud_assets,
                    search_query: search_query,
                    show_add_modal: show_add_modal
                }
            }
        }
    }
}

/// 硬件设备标签页
#[component]
fn HardwareAssetsTab(
    assets: Signal<Vec<HardwareAsset>>,
    search_query: Signal<String>,
    show_add_modal: Signal<bool>
) -> Element {
    let mut editing_asset = use_signal(|| None::<HardwareAsset>);

    // 计算统计数据
    let total_count = assets.read().len() as i32;
    let server_count = assets.read().iter().filter(|a| a.asset_type == "服务器").count() as i32;
    let workstation_count = assets.read().iter().filter(|a| a.asset_type == "工作站").count() as i32;
    let online_count = assets.read().iter().filter(|a| a.status == "在线").count() as i32;

    // 过滤资产
    let filtered_assets: Vec<HardwareAsset> = assets.read()
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

                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的资产"
                    }
                }
            }
        }

        // 编辑硬件资产模态框
        if let Some(asset) = editing_asset.read().clone() {
            EditHardwareAssetModal {
                asset: asset.clone(),
                on_close: move |_| editing_asset.set(None),
                on_save: {
                    let mut editing_asset_signal = editing_asset.clone();
                    move |updated: HardwareAsset| {
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

/// 云服务资产标签页
#[component]
fn CloudAssetsTab(
    assets: Signal<Vec<CloudAsset>>,
    search_query: Signal<String>,
    show_add_modal: Signal<bool>
) -> Element {
    let mut editing_asset = use_signal(|| None::<CloudAsset>);

    // 计算统计数据
    let total_count = assets.read().len() as i32;
    let alicloud_count = assets.read().iter().filter(|a| a.provider == "阿里云").count() as i32;
    let tencent_count = assets.read().iter().filter(|a| a.provider == "腾讯云").count() as i32;
    let running_count = assets.read().iter().filter(|a| a.status == "运行中").count() as i32;

    // 过滤资产
    let filtered_assets: Vec<CloudAsset> = assets.read()
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
        // 搜索栏
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center",
                Icon { icon: FaMagnifyingGlass, width: 20, height: 20 }
                input {
                    r#type: "text",
                    class: "ml-2 flex-1 border-0 focus:outline-none",
                    placeholder: "搜索实例名称或IP地址...",
                    value: search_query,
                    oninput: move |e| search_query.set(e.value()),
                }
            }
        }

        // 统计卡片
        div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-blue-500",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "总实例" }
                        p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-orange-500",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "阿里云" }
                        p { class: "text-xl font-bold text-gray-800", {alicloud_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-blue-600",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "腾讯云" }
                        p { class: "text-xl font-bold text-gray-800", {tencent_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-green-600",
                        Icon { icon: FaServer, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "运行中" }
                        p { class: "text-xl font-bold text-gray-800", {running_count.to_string()} }
                    }
                }
            }
        }

        // 资产列表表格
        div { class: "bg-white rounded-lg shadow overflow-hidden",
            table { class: "min-w-full divide-y divide-gray-200",
                thead { class: "bg-gray-50",
                    tr {
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "实例名称" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "云厂商" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "实例类型" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "IP地址" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "区域" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                    }
                }
                tbody { class: "bg-white divide-y divide-gray-200",
                    for asset in filtered_assets {
                        tr { class: "hover:bg-gray-50",
                            td { class: "px-6 py-4 whitespace-nowrap",
                                div { class: "flex items-center",
                                    div { class: "flex-shrink-0 h-10 w-10 bg-blue-50 rounded-full flex items-center justify-center",
                                        Icon { icon: FaCloud, width: 20, height: 20, class: "text-blue-500" }
                                    }
                                    div { class: "ml-4",
                                        div { class: "text-sm font-medium text-gray-900", {asset.name.clone()} }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.provider.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.instance_type.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.ip_address.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span {
                                    class: if asset.status == "运行中" {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                    } else {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800"
                                    },
                                    {asset.status.clone()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.region.clone()}
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

                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的云服务资产"
                    }
                }
            }
        }

        // 编辑云服务资产模态框
        if let Some(asset) = editing_asset.read().clone() {
            EditCloudAssetModal {
                asset: asset.clone(),
                on_close: move |_| editing_asset.set(None),
                on_save: {
                    let mut editing_asset_signal = editing_asset.clone();
                    move |updated: CloudAsset| {
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

/// 编辑硬件资产模态框
#[component]
fn EditHardwareAssetModal(asset: HardwareAsset, on_close: EventHandler<()>, on_save: EventHandler<HardwareAsset>) -> Element {
    let mut name = use_signal(|| asset.name.clone());
    let mut asset_type = use_signal(|| asset.asset_type.clone());
    let mut ip_address = use_signal(|| asset.ip_address.clone());
    let mut status = use_signal(|| asset.status.clone());
    let mut location = use_signal(|| asset.location.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑硬件资产" }
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
                                let updated = HardwareAsset {
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

/// 编辑云服务资产模态框
#[component]
fn EditCloudAssetModal(asset: CloudAsset, on_close: EventHandler<()>, on_save: EventHandler<CloudAsset>) -> Element {
    let mut name = use_signal(|| asset.name.clone());
    let mut provider = use_signal(|| asset.provider.clone());
    let mut instance_type = use_signal(|| asset.instance_type.clone());
    let mut ip_address = use_signal(|| asset.ip_address.clone());
    let mut status = use_signal(|| asset.status.clone());
    let mut region = use_signal(|| asset.region.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑云服务资产" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "实例名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "云厂商" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: provider,
                            onchange: move |e| provider.set(e.value()),
                            option { value: "阿里云", "阿里云" }
                            option { value: "腾讯云", "腾讯云" }
                            option { value: "华为云", "华为云" }
                            option { value: "AWS", "AWS" }
                            option { value: "Azure", "Azure" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "实例类型" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: ecs.g6.large",
                            value: instance_type,
                            oninput: move |e| instance_type.set(e.value()),
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
                            option { value: "运行中", "运行中" }
                            option { value: "已停止", "已停止" }
                            option { value: "已释放", "已释放" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "区域" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: 华东1-杭州",
                            value: region,
                            oninput: move |e| region.set(e.value()),
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
                                let updated = CloudAsset {
                                    id: asset_id,
                                    name: name.read().clone(),
                                    provider: provider.read().clone(),
                                    instance_type: instance_type.read().clone(),
                                    ip_address: ip_address.read().clone(),
                                    status: status.read().clone(),
                                    region: region.read().clone(),
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
