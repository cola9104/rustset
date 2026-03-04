use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaShieldHalved,
    FaEye,
};
use crate::state::security_product::{
    SecurityProduct, SecurityProductCategory, SecurityProductStatus,
};
use crate::app::{SECURITY_PRODUCTS_STATE, PROVIDERS_STATE, CLOUD_PLATFORMS_STATE, MACHINE_ROOMS_STATE};

/// 安全产品管理页面
#[component]
pub fn SecurityProductManagement() -> Element {
    let mut search_query = use_signal(|| String::new());
    let mut category_filter = use_signal(|| String::from("all"));
    let mut status_filter = use_signal(|| String::from("all"));

    // 模态框状态
    let mut show_add_modal = use_signal(|| false);
    let mut editing_product = use_signal(|| None::<SecurityProduct>);
    let mut viewing_product = use_signal(|| None::<SecurityProduct>);

    // 统计数据
    let total_count = SECURITY_PRODUCTS_STATE.read().len() as i32;
    let active_count = SECURITY_PRODUCTS_STATE.read()
        .iter()
        .filter(|p| p.status == SecurityProductStatus::Active)
        .count() as i32;

    // 过滤逻辑
    let filtered_products: Vec<SecurityProduct> = SECURITY_PRODUCTS_STATE.read()
        .iter()
        .filter(|product| {
            let matches_search = search_query.read().is_empty()
                || product.name.to_lowercase().contains(&search_query.read().to_lowercase())
                || product.vendor.to_lowercase().contains(&search_query.read().to_lowercase())
                || product.model.to_lowercase().contains(&search_query.read().to_lowercase());

            let matches_category = *category_filter.read() == "all"
                || product.category.display_name() == *category_filter.read();

            let matches_status = *status_filter.read() == "all"
                || product.status.display_name() == *status_filter.read();

            matches_search && matches_category && matches_status
        })
        .cloned()
        .collect();

    let is_empty = filtered_products.is_empty();

    rsx! {
        div { class: "p-6 space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                div { class: "flex items-center gap-3",
                    Icon { icon: FaShieldHalved, width: 28, height: 28, class: "text-blue-600" }
                    div {
                        h1 { class: "text-2xl font-bold text-gray-800", "安全产品管理" }
                        p { class: "text-sm text-gray-500 mt-1", "管理防火墙、WAF、IPS等安全设备" }
                    }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加产品" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                StatCard {
                    title: "总产品数",
                    value: "{total_count}",
                    color: "blue"
                }
                StatCard {
                    title: "运行中",
                    value: "{active_count}",
                    color: "green"
                }
                // 显示各分类数量
                {
                    let firewall_count = SECURITY_PRODUCTS_STATE.read()
                        .iter()
                        .filter(|p| p.category == SecurityProductCategory::Firewall && p.status == SecurityProductStatus::Active)
                        .count() as i32;
                    rsx! {
                        StatCard {
                            title: "防火墙",
                            value: "{firewall_count}",
                            color: "indigo"
                        }
                    }
                }
                {
                    let waf_count = SECURITY_PRODUCTS_STATE.read()
                        .iter()
                        .filter(|p| p.category == SecurityProductCategory::Waf && p.status == SecurityProductStatus::Active)
                        .count() as i32;
                    rsx! {
                        StatCard {
                            title: "WAF",
                            value: "{waf_count}",
                            color: "purple"
                        }
                    }
                }
            }

            // 搜索和筛选
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex gap-4",
                    div { class: "flex-1 relative",
                        Icon {
                            icon: FaMagnifyingGlass,
                            width: 16,
                            height: 16,
                            class: "absolute left-3 top-2.5 text-gray-400"
                        }
                        input {
                            r#type: "text",
                            class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "搜索产品名称、厂商、型号...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{category_filter}",
                        onchange: move |e| category_filter.set(e.value()),
                        option { value: "all", "全部分类" }
                        for category in SecurityProductCategory::all_categories() {
                            option { value: "{category.display_name()}", "{category.display_name()}" }
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{status_filter}",
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "all", "全部状态" }
                        for status in SecurityProductStatus::all_statuses() {
                            option { value: "{status.display_name()}", "{status.display_name()}" }
                        }
                    }
                }
            }

            // 产品列表表格
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "产品名称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "分类" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "厂商/型号" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "版本" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "管理IP" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "部署位置" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        if is_empty {
                            tr {
                                td { colspan: 8, class: "px-6 py-12 text-center text-gray-500",
                                    "暂无安全产品数据"
                                }
                            }
                        } else {
                            for product in filtered_products.iter() {
                                {
                                    let deployment_location = get_deployment_location(&product);
                                    rsx! {
                                        tr { class: "hover:bg-gray-50",
                                            td { class: "px-6 py-4 whitespace-nowrap",
                                                div { class: "flex items-center",
                                                    div { class: "flex-shrink-0 h-10 w-10 bg-indigo-100 rounded-full flex items-center justify-center",
                                                        Icon { icon: FaShieldHalved, width: 20, height: 20, class: "text-indigo-600" }
                                                    }
                                                    div { class: "ml-4",
                                                        div { class: "text-sm font-medium text-gray-900", "{product.name}" }
                                                        div { class: "text-sm text-gray-500", "{product.vendor} {product.model}" }
                                                    }
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap",
                                                span { class: "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-indigo-100 text-indigo-800",
                                                    "{product.category.display_name()}"
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                                "{product.version}"
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                                if let Some(ref ip) = product.management_ip {
                                                    "{ip}"
                                                } else {
                                                    "-"
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                                "{deployment_location}"
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap",
                                                span { class: "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full {product.status.color_class()}",
                                                    "{product.status.display_name()}"
                                                }
                                            }
                                            td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                                button {
                                                    class: "text-blue-600 hover:text-blue-900 mr-3",
                                                    onclick: {
                                                        let product = product.clone();
                                                        move |_| viewing_product.set(Some(product.clone()))
                                                    },
                                                    Icon { icon: FaEye, width: 16, height: 16 }
                                                }
                                                button {
                                                    class: "text-indigo-600 hover:text-indigo-900 mr-3",
                                                    onclick: {
                                                        let product = product.clone();
                                                        move |_| editing_product.set(Some(product.clone()))
                                                    },
                                                    Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                                }
                                                button {
                                                    class: "text-red-600 hover:text-red-900",
                                                    onclick: {
                                                        let product_id = product.id;
                                                        move |_| {
                                                            let mut products = SECURITY_PRODUCTS_STATE.write();
                                                            if let Some(pos) = products.iter().position(|p| p.id == product_id) {
                                                                products.remove(pos);
                                                            }
                                                        }
                                                    },
                                                    Icon { icon: FaTrash, width: 16, height: 16 }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 添加产品模态框
        if *show_add_modal.read() {
            AddEditProductModal {
                product: None,
                on_save: move |new_product: SecurityProduct| {
                    let mut products = SECURITY_PRODUCTS_STATE.write();
                    // 生成新ID
                    let new_id = products.iter().map(|p| p.id).max().unwrap_or(0) + 1;
                    let mut new_product = new_product;
                    new_product.id = new_id;
                    products.push(new_product);
                    show_add_modal.set(false);
                },
                on_close: move |_| show_add_modal.set(false)
            }
        }

        // 编辑产品模态框
        if let Some(product) = editing_product.read().as_ref() {
            AddEditProductModal {
                product: Some(product.clone()),
                on_save: move |updated: SecurityProduct| {
                    let mut products = SECURITY_PRODUCTS_STATE.write();
                    if let Some(idx) = products.iter().position(|p| p.id == updated.id) {
                        products[idx] = updated;
                    }
                    editing_product.set(None);
                },
                on_close: move |_| editing_product.set(None)
            }
        }

        // 查看产品模态框
        if let Some(product) = viewing_product.read().as_ref() {
            ViewProductModal {
                product: product.clone(),
                on_close: move |_| viewing_product.set(None)
            }
        }
    }
}

/// 获取部署位置
fn get_deployment_location(product: &SecurityProduct) -> String {
    if product.is_cloud_deployment() {
        CLOUD_PLATFORMS_STATE.read()
            .iter()
            .find(|p| p.id == product.cloud_platform_id.unwrap())
            .map(|p| p.platform_name.clone())
            .unwrap_or_else(|| "云平台".to_string())
    } else if product.is_physical_deployment() {
        MACHINE_ROOMS_STATE.read()
            .iter()
            .find(|r| r.id == product.machine_room_id.unwrap())
            .map(|r| r.room_name.clone())
            .unwrap_or_else(|| "机房".to_string())
    } else {
        "未指定".to_string()
    }
}

/// 统计卡片组件
#[component]
fn StatCard(
    title: String,
    value: String,
    color: String,
) -> Element {
    let bg_color = match color.as_str() {
        "blue" => "bg-blue-500",
        "green" => "bg-green-500",
        "indigo" => "bg-indigo-500",
        "purple" => "bg-purple-500",
        _ => "bg-gray-500",
    };

    rsx! {
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center",
                div { class: "p-2 rounded-full {bg_color}",
                    Icon { icon: FaShieldHalved, width: 20, height: 20, class: "text-white" }
                }
                div { class: "ml-3",
                    p { class: "text-sm text-gray-500", "{title}" }
                    p { class: "text-xl font-bold text-gray-800", "{value}" }
                }
            }
        }
    }
}

/// 添加/编辑产品模态框
#[component]
fn AddEditProductModal(
    product: Option<SecurityProduct>,
    on_save: EventHandler<SecurityProduct>,
    on_close: EventHandler<()>,
) -> Element {
    let is_edit = product.is_some();
    let modal_title = if is_edit { "编辑产品" } else { "添加产品" };
    let mut name = use_signal(|| product.as_ref().map(|p| p.name.clone()).unwrap_or_default());
    let mut category = use_signal(|| product.as_ref().map(|p| p.category).unwrap_or(SecurityProductCategory::Firewall));
    let mut vendor = use_signal(|| product.as_ref().map(|p| p.vendor.clone()).unwrap_or_default());
    let mut model_val = use_signal(|| product.as_ref().map(|p| p.model.clone()).unwrap_or_default());
    let mut version = use_signal(|| product.as_ref().map(|p| p.version.clone()).unwrap_or_default());
    let mut serial_number = use_signal(|| product.as_ref().and_then(|p| p.serial_number.clone()).unwrap_or_default());
    let mut license_type = use_signal(|| product.as_ref().map(|p| p.license_type.clone()).unwrap_or("永久".to_string()));
    let mut license_expiry = use_signal(|| product.as_ref().and_then(|p| p.license_expiry.clone()).unwrap_or_default());
    let mut management_ip = use_signal(|| product.as_ref().and_then(|p| p.management_ip.clone()).unwrap_or_default());
    let mut deployment_mode = use_signal(|| product.as_ref().map(|p| p.deployment_mode.clone()).unwrap_or_default());
    let mut status = use_signal(|| product.as_ref().map(|p| p.status).unwrap_or(SecurityProductStatus::Active));
    let mut throughput = use_signal(|| product.as_ref().and_then(|p| p.throughput.clone()).unwrap_or_default());
    let mut contact_person = use_signal(|| product.as_ref().map(|p| p.contact_person.clone()).unwrap_or_default());
    let mut contact_phone = use_signal(|| product.as_ref().map(|p| p.contact_phone.clone()).unwrap_or_default());
    let mut remarks = use_signal(|| product.as_ref().and_then(|p| p.remarks.clone()).unwrap_or_default());

    let providers = PROVIDERS_STATE.read().clone();
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read().clone();
    let machine_rooms = MACHINE_ROOMS_STATE.read().clone();

    let category_display = category.read().display_name().to_string();
    let status_display = status.read().display_name().to_string();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto",
                div { class: "flex justify-between items-center p-4 border-b sticky top-0 bg-white",
                    h3 { class: "text-lg font-semibold", "{modal_title}" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4 overflow-y-auto",
                    // 基本信息
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "产品名称 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{name}",
                                oninput: move |e| name.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "产品分类 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{category_display}",
                                onchange: move |e| category.set(SecurityProductCategory::from_str(&e.value())),
                                for cat in SecurityProductCategory::all_categories() {
                                    option { value: "{cat.display_name()}", "{cat.display_name()}" }
                                }
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "厂商 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{vendor}",
                                oninput: move |e| vendor.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "型号 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{model_val}",
                                oninput: move |e| model_val.set(e.value()),
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "版本" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{version}",
                                oninput: move |e| version.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "序列号" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{serial_number}",
                                oninput: move |e| serial_number.set(e.value()),
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "管理IP" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{management_ip}",
                                oninput: move |e| management_ip.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "部署模式" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{deployment_mode}",
                                onchange: move |e| deployment_mode.set(e.value()),
                                option { value: "路由模式", "路由模式" }
                                option { value: "桥接模式", "桥接模式" }
                                option { value: "单臂模式", "单臂模式" }
                                option { value: "旁路模式", "旁路模式" }
                                option { value: "单机部署", "单机部署" }
                                option { value: "分布式部署", "分布式部署" }
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "授权类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{license_type}",
                                onchange: move |e| license_type.set(e.value()),
                                option { value: "永久", "永久" }
                                option { value: "订阅", "订阅" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "授权到期" }
                            input {
                                r#type: "date",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{license_expiry}",
                                oninput: move |e| license_expiry.set(e.value()),
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "吞吐量" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "如：40 Gbps",
                                value: "{throughput}",
                                oninput: move |e| throughput.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{status_display}",
                                onchange: move |e| {
                                    status.set(match e.value().as_str() {
                                        "运行中" => SecurityProductStatus::Active,
                                        "已停用" => SecurityProductStatus::Inactive,
                                        "维护中" => SecurityProductStatus::Maintenance,
                                        "已下线" => SecurityProductStatus::Decommissioned,
                                        _ => SecurityProductStatus::Active,
                                    });
                                },
                                for s in SecurityProductStatus::all_statuses() {
                                    option { value: "{s.display_name()}", "{s.display_name()}" }
                                }
                            }
                        }
                    }

                    div { class: "border-t pt-4 mt-2",
                        h4 { class: "text-sm font-medium text-gray-700 mb-2", "部署位置" }
                        div { class: "grid grid-cols-3 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    option { value: "", "请选择" }
                                    for provider in providers.iter() {
                                        option { value: "{provider.id}", "{provider.short_name}" }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "云平台" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    option { value: "", "请选择" }
                                    for platform in cloud_platforms.iter() {
                                        option { value: "{platform.id}", "{platform.platform_name}" }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "机房" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    option { value: "", "请选择" }
                                    for room in machine_rooms.iter() {
                                        option { value: "{room.id}", "{room.room_name}" }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "border-t pt-4 mt-2",
                        h4 { class: "text-sm font-medium text-gray-700 mb-2", "联系信息" }
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: "{contact_person}",
                                    oninput: move |e| contact_person.set(e.value()),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "联系电话" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: "{contact_phone}",
                                    oninput: move |e| contact_phone.set(e.value()),
                                }
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            rows: 3,
                            value: "{remarks}",
                            oninput: move |e| remarks.set(e.value()),
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t sticky bottom-0 bg-white",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| {
                            let new_product = SecurityProduct {
                                id: product.as_ref().map(|p| p.id).unwrap_or(0),
                                name: name.read().clone(),
                                category: *category.read(),
                                vendor: vendor.read().clone(),
                                model: model_val.read().clone(),
                                version: version.read().clone(),
                                serial_number: if serial_number.read().is_empty() { None } else { Some(serial_number.read().clone()) },
                                license_type: license_type.read().clone(),
                                license_expiry: if license_expiry.read().is_empty() { None } else { Some(license_expiry.read().clone()) },
                                management_ip: if management_ip.read().is_empty() { None } else { Some(management_ip.read().clone()) },
                                deployment_mode: deployment_mode.read().clone(),
                                cloud_platform_id: None,
                                machine_room_id: None,
                                provider_id: None,
                                status: *status.read(),
                                features: vec![],
                                throughput: if throughput.read().is_empty() { None } else { Some(throughput.read().clone()) },
                                contact_person: contact_person.read().clone(),
                                contact_phone: contact_phone.read().clone(),
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
                            };
                            on_save.call(new_product);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 查看产品详情模态框
#[component]
fn ViewProductModal(
    product: SecurityProduct,
    on_close: EventHandler<()>,
) -> Element {
    let deployment_location = get_deployment_location(&product);
    let provider_name = match product.provider_id {
        Some(pid) => PROVIDERS_STATE.read()
            .iter()
            .find(|p| p.id == pid)
            .map(|p| p.short_name.clone())
            .unwrap_or_else(|| "未分配".to_string()),
        None => "未分配".to_string(),
    };

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-lg mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "产品详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-3",
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "产品名称" }
                        span { class: "font-medium", "{product.name}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "分类" }
                        span { class: "px-2 py-1 inline-flex text-xs font-semibold rounded-full bg-indigo-100 text-indigo-800",
                            "{product.category.display_name()}"
                        }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "厂商/型号" }
                        span { class: "font-medium", "{product.vendor} {product.model}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "版本" }
                        span { class: "font-medium", "{product.version}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "管理IP" }
                        span { class: "font-medium", "{product.management_ip.as_ref().unwrap_or(&\"-\".to_string())}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "部署位置" }
                        span { class: "font-medium", "{deployment_location}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "服务商" }
                        span { class: "font-medium", "{provider_name}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "部署模式" }
                        span { class: "font-medium", "{product.deployment_mode}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "授权类型" }
                        span { class: "font-medium", "{product.license_type}" }
                    }
                    if let Some(expiry) = &product.license_expiry {
                        div { class: "flex justify-between",
                            span { class: "text-gray-500", "授权到期" }
                            span { class: "font-medium", "{expiry}" }
                        }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "状态" }
                        span { class: "px-2 py-1 inline-flex text-xs font-semibold rounded-full {product.status.color_class()}",
                            "{product.status.display_name()}"
                        }
                    }
                    if let Some(ref throughput) = &product.throughput {
                        div { class: "flex justify-between",
                            span { class: "text-gray-500", "吞吐量" }
                            span { class: "font-medium", "{throughput}" }
                        }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "负责人" }
                        span { class: "font-medium", "{product.contact_person}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-500", "联系电话" }
                        span { class: "font-medium", "{product.contact_phone}" }
                    }
                    if let Some(ref remarks) = &product.remarks {
                        div { class: "mt-2",
                            span { class: "text-gray-500", "备注" }
                            div { class: "text-sm mt-1", "{remarks}" }
                        }
                    }
                }

                div { class: "flex justify-end p-4 border-t",
                    button {
                        class: "px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200",
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                }
            }
        }
    }
}
