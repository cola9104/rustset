use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaShieldHalved,
    FaEye,
};
use crate::state::security_product::{
    SecurityProduct, SecurityProductCategory, SecurityProductStatus,
};
use crate::app::{PROVIDERS_STATE, CLOUD_PLATFORMS_STATE, MACHINE_ROOMS_STATE};
use crate::services::{
    fetch_security_products, create_security_product, update_security_product, delete_security_product,
    fetch_service_providers, fetch_machine_rooms, fetch_cloud_platform_configs,
};
use super::product_form::{ProductForm, FormMode};

/// 安全产品管理页面
#[component]
pub fn SecurityProductManagement() -> Element {
    let mut search_query = use_signal(String::new);
    let mut category_filter = use_signal(|| String::from("all"));
    let mut status_filter = use_signal(|| String::from("all"));

    // 模态框状态
    let mut show_add_modal = use_signal(|| false);
    let mut editing_product = use_signal(|| None::<SecurityProduct>);
    let mut viewing_product = use_signal(|| None::<SecurityProduct>);

    // 数据和加载状态
    let products = use_signal(Vec::<SecurityProduct>::new);
    let is_loading = use_signal(|| true);

    // 加载数据 - 包括安全产品、服务商、机房和云平台
    {
        let mut products_clone = products;
        let mut is_loading_clone = is_loading;
        use_effect(move || {
            spawn(async move {
                // 加载服务商数据
                match fetch_service_providers().await {
                    Ok(providers) => {
                        *PROVIDERS_STATE.write() = providers;
                    }
                    Err(e) => {
                        tracing::error!("加载服务商数据失败: {}", e);
                    }
                }
                // 加载机房数据
                match fetch_machine_rooms().await {
                    Ok(rooms) => {
                        *MACHINE_ROOMS_STATE.write() = rooms;
                    }
                    Err(e) => {
                        tracing::error!("加载机房数据失败: {}", e);
                    }
                }
                // 加载云平台数据
                match fetch_cloud_platform_configs().await {
                    Ok(platforms) => {
                        *CLOUD_PLATFORMS_STATE.write() = platforms;
                    }
                    Err(e) => {
                        tracing::error!("加载云平台数据失败: {}", e);
                    }
                }
                // 加载安全产品数据
                match fetch_security_products().await {
                    Ok(data) => {
                        products_clone.set(data);
                        is_loading_clone.set(false);
                    }
                    Err(e) => {
                        tracing::error!("加载安全产品数据失败: {}", e);
                        is_loading_clone.set(false);
                    }
                }
            });
        });
    }

    // 刷新数据的函数
    let refresh_data = {
        let products = products;
        move || {
            let mut products = products;
            spawn(async move {
                match fetch_security_products().await {
                    Ok(data) => {
                        products.set(data);
                    }
                    Err(e) => {
                        tracing::error!("刷新安全产品数据失败: {}", e);
                    }
                }
            });
        }
    };

    // 统计数据
    let total_count = products.read().len() as i32;
    let active_count = products.read()
        .iter()
        .filter(|p| p.status == SecurityProductStatus::Active)
        .count() as i32;

    // 过滤逻辑
    let filtered_products: Vec<SecurityProduct> = products.read()
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
                    let firewall_count = products.read()
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
                    let waf_count = products.read()
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

            // 加载状态
            if *is_loading.read() {
                div { class: "bg-white rounded-lg shadow p-12 text-center",
                    div { class: "text-gray-500", "加载数据中..." }
                }
            } else {
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
                                        let deployment_location = get_deployment_location(product);
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
                                                            let refresh_data = refresh_data;
                                                            move |_| {
                                                                let refresh_data = refresh_data;
                                                                spawn(async move {
                                                                    match delete_security_product(product_id).await {
                                                                        Ok(()) => {
                                                                            refresh_data();
                                                                        }
                                                                        Err(e) => {
                                                                            tracing::error!("删除安全产品失败: {}", e);
                                                                        }
                                                                    }
                                                                });
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
        }

        // 添加产品模态框
        if *show_add_modal.read() {
            ProductForm {
                mode: FormMode::New,
                product: None,
                on_save: move |new_product: SecurityProduct| {
                    let refresh_data = refresh_data;
                    spawn(async move {
                        match create_security_product(&new_product).await {
                            Ok(_) => {
                                refresh_data();
                            }
                            Err(e) => {
                                tracing::error!("创建安全产品失败: {}", e);
                            }
                        }
                    });
                    show_add_modal.set(false);
                },
                on_close: move |_| show_add_modal.set(false)
            }
        }

        // 编辑产品模态框
        if let Some(product) = editing_product.read().as_ref() {
            ProductForm {
                mode: FormMode::Edit,
                product: Some(product.clone()),
                on_save: move |updated: SecurityProduct| {
                    let refresh_data = refresh_data;
                    spawn(async move {
                        match update_security_product(updated.id, &updated).await {
                            Ok(_) => {
                                refresh_data();
                            }
                            Err(e) => {
                                tracing::error!("更新安全产品失败: {}", e);
                            }
                        }
                    });
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
