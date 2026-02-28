use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaServer, FaCloud,
    FaCircleCheck, FaClock, FaCircleXmark, FaEye, FaFilter, FaDownload,
    FaBuilding, FaUser, FaNetworkWired, FaMoneyBill, FaShield, FaFileLines,
};
use shared::BusinessResource;

/// 业务资源管理页面
#[allow(non_snake_case)]
pub fn BusinessApplication() -> Element {
    let mut resources = use_signal(Vec::<BusinessResource>::new);
    let mut search_query = use_signal(String::new);
    let mut filter_status = use_signal(|| "全部".to_string());
    let mut filter_type = use_signal(|| "全部".to_string());
    let mut show_add_modal = use_signal(|| false);
    let mut show_detail_modal = use_signal(|| None::<i32>);
    let mut editing_resource = use_signal(|| None::<i32>);

    // 加载数据
    use_effect(move || {
        // TODO: 从API加载实际数据
        // 这里使用示例数据
        let sample_data = create_sample_data();
        resources.set(sample_data);
    });

    // 计算统计数据
    let total_count = resources.read().len() as i32;
    let cloud_count = resources.read().iter().filter(|r| r.resource_type == "cloud").count() as i32;
    let physical_count = resources.read().iter().filter(|r| r.resource_type == "physical").count() as i32;
    let pending_count = resources.read().iter()
        .filter(|r| r.application_status.as_deref() == Some("待审核"))
        .count() as i32;

    // 过滤数据
    let filtered_resources: Vec<BusinessResource> = resources.read()
        .iter()
        .filter(|resource| {
            let query = search_query.read().to_lowercase();
            let filter_status_val = filter_status.read().clone();
            let filter_type_val = filter_type.read().clone();
            let status_match = filter_status_val == "全部"
                || resource.application_status.as_ref() == Some(&filter_status_val);
            let type_match = filter_type_val == "全部"
                || resource.resource_type == filter_type_val;

            (query.is_empty() ||
             resource.ecs_name.to_lowercase().contains(&query) ||
             resource.customer_name.to_lowercase().contains(&query) ||
             resource.ip_address.contains(&query))
                && status_match && type_match
        })
        .cloned()
        .collect();

    let is_empty = filtered_resources.is_empty();

    rsx! {
        div { class: "space-y-6 p-6 bg-gray-50 min-h-screen",
            // 页面标题
            div { class: "flex justify-between items-center",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "业务资源申请管理" }
                    p { class: "text-sm text-gray-500 mt-1", "管理云资源和物理机申请流程" }
                }
                div { class: "flex space-x-3",
                    button {
                        class: "flex items-center px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors",
                        Icon { icon: FaDownload, width: 16, height: 16 }
                        span { class: "ml-2", "导出" }
                    }
                    button {
                        class: "flex items-center px-4 py-2 bg-gradient-to-r from-blue-600 to-blue-700 text-white rounded-lg hover:from-blue-700 hover:to-blue-800 transition-all shadow-sm",
                        onclick: move |_| show_add_modal.set(true),
                        Icon { icon: FaPlus, width: 16, height: 16 }
                        span { class: "ml-2", "新建申请" }
                    }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                StatCard {
                    title: "总申请数",
                    value: total_count.to_string(),
                    color: "blue",
                    Icon { icon: FaServer, width: 24, height: 24 }
                }
                StatCard {
                    title: "云服务器",
                    value: cloud_count.to_string(),
                    color: "cyan",
                    Icon { icon: FaCloud, width: 24, height: 24 }
                }
                StatCard {
                    title: "物理机",
                    value: physical_count.to_string(),
                    color: "purple",
                    Icon { icon: FaServer, width: 24, height: 24 }
                }
                StatCard {
                    title: "待审核",
                    value: pending_count.to_string(),
                    color: "orange",
                    Icon { icon: FaClock, width: 24, height: 24 }
                }
            }

            // 搜索和筛选栏
            div { class: "bg-white rounded-xl shadow-sm p-4",
                div { class: "flex flex-wrap gap-4 items-center",
                    // 搜索框
                    div { class: "flex-1 min-w-64",
                        div { class: "relative",
                            div { class: "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none",
                                Icon { icon: FaMagnifyingGlass, width: 16, height: 16 }
                            }
                            input {
                                r#type: "text",
                                class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "搜索名称、客户或IP地址...",
                                value: search_query,
                                oninput: move |e| search_query.set(e.value()),
                            }
                        }
                    }
                    // 状态筛选
                    div { class: "flex items-center space-x-2",
                        Icon { icon: FaFilter, width: 16, height: 16 }
                        select {
                            class: "px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: filter_status,
                            onchange: move |e| filter_status.set(e.value()),
                            option { value: "全部", "全部状态" }
                            option { value: "待审核", "待审核" }
                            option { value: "已批准", "已批准" }
                            option { value: "已拒绝", "已拒绝" }
                        }
                    }
                    // 类型筛选
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        value: filter_type,
                        onchange: move |e| filter_type.set(e.value()),
                        option { value: "全部", "全部类型" }
                        option { value: "cloud", "云服务器" }
                        option { value: "physical", "物理机" }
                    }
                }
            }

            // 数据表格
            div { class: "bg-white rounded-xl shadow-sm overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "资源名称" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "类型" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "客户/项目" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "配置" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "IP地址" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "申请状态" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "交付状态" }
                            th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for resource in filtered_resources {
                            tr { class: "hover:bg-gray-50 transition-colors",
                                // 资源名称
                                td { class: "px-6 py-4",
                                    div { class: "flex items-center",
                                        div {
                                            class: if resource.resource_type == "cloud" {
                                                "flex-shrink-0 h-10 w-10 bg-blue-100 rounded-lg flex items-center justify-center"
                                            } else {
                                                "flex-shrink-0 h-10 w-10 bg-purple-100 rounded-lg flex items-center justify-center"
                                            },
                                            if resource.resource_type == "cloud" {
                                                Icon { icon: FaCloud, width: 20, height: 20 }
                                            } else {
                                                Icon { icon: FaServer, width: 20, height: 20 }
                                            }
                                        }
                                        div { class: "ml-4",
                                            div { class: "text-sm font-medium text-gray-900", {resource.ecs_name.clone()} }
                                            div { class: "text-xs text-gray-500", {resource.cloud_region.clone()} }
                                        }
                                    }
                                }
                                // 类型
                                td { class: "px-6 py-4",
                                    span {
                                        class: if resource.resource_type == "cloud" {
                                            "px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800"
                                        } else {
                                            "px-2 py-1 text-xs font-medium rounded-full bg-purple-100 text-purple-800"
                                        },
                                        if resource.resource_type == "cloud" { "云服务器" } else { "物理机" }
                                    }
                                }
                                // 客户/项目
                                td { class: "px-6 py-4",
                                    div { class: "text-sm text-gray-900", {resource.customer_name.clone()} }
                                    div { class: "text-xs text-gray-500",
                                        {resource.project_name.clone().unwrap_or_default()}
                                    }
                                }
                                // 配置
                                td { class: "px-6 py-4",
                                    div { class: "text-sm text-gray-900",
                                        "{resource.cpu_cores}核 {resource.memory_gb}GB"
                                    }
                                    div { class: "text-xs text-gray-500",
                                        {resource.ecs_type.clone()}
                                    }
                                }
                                // IP地址
                                td { class: "px-6 py-4 text-sm text-gray-600",
                                    {resource.ip_address.clone()}
                                }
                                // 申请状态
                                td { class: "px-6 py-4",
                                    StatusBadge { status: resource.application_status.clone() }
                                }
                                // 交付状态
                                td { class: "px-6 py-4",
                                    DeliveryStatusBadge { status: resource.delivery_status.clone() }
                                }
                                // 操作
                                td { class: "px-6 py-4",
                                    div { class: "flex items-center space-x-2",
                                        button {
                                            class: "p-1.5 text-gray-500 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors",
                                            title: "查看详情",
                                            onclick: {
                                                let id = resource.id;
                                                move |_| show_detail_modal.set(id)
                                            },
                                            Icon { icon: FaEye, width: 16, height: 16 }
                                        }
                                        button {
                                            class: "p-1.5 text-gray-500 hover:text-green-600 hover:bg-green-50 rounded-lg transition-colors",
                                            title: "编辑",
                                            onclick: {
                                                let id = resource.id;
                                                move |_| editing_resource.set(id)
                                            },
                                            Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                        }
                                        button {
                                            class: "p-1.5 text-gray-500 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors",
                                            title: "删除",
                                            onclick: {
                                                let id = resource.id;
                                                move |_| {
                                                    if let Some(id) = id {
                                                        resources.write().retain(|r| r.id != Some(id));
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

                // 空状态
                if is_empty {
                    div { class: "text-center py-16",
                        div { class: "text-gray-400 mb-4",
                            Icon { icon: FaServer, width: 48, height: 48 }
                        }
                        p { class: "text-gray-500 text-lg", "没有找到匹配的业务资源" }
                        p { class: "text-gray-400 text-sm mt-2", "尝试调整搜索条件或添加新的申请" }
                    }
                }
            }
        }

        // 添加模态框
        if *show_add_modal.read() {
            AddResourceModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |resource: BusinessResource| {
                    resources.write().push(resource);
                    show_add_modal.set(false);
                }
            }
        }

        // 详情模态框
        if let Some(id) = *show_detail_modal.read() {
            if let Some(resource) = resources.read().iter().find(|r| r.id == Some(id)).cloned() {
                ResourceDetailModal {
                    resource: resource,
                    on_close: move |_| show_detail_modal.set(None),
                }
            }
        }

        // 编辑模态框
        if let Some(id) = *editing_resource.read() {
            if let Some(resource) = resources.read().iter().find(|r| r.id == Some(id)).cloned() {
                EditResourceModal {
                    resource: resource,
                    on_close: move |_| editing_resource.set(None),
                    on_save: move |updated: BusinessResource| {
                        let mut list = resources.write();
                        if let Some(r) = list.iter_mut().find(|r| r.id == updated.id) {
                            *r = updated;
                        }
                        editing_resource.set(None);
                    }
                }
            }
        }
    }
}

/// 统计卡片组件
#[component]
fn StatCard(title: String, value: String, color: String, children: Element) -> Element {
    let bg_color = match color.as_str() {
        "blue" => "bg-blue-500",
        "cyan" => "bg-cyan-500",
        "purple" => "bg-purple-500",
        "orange" => "bg-orange-500",
        _ => "bg-gray-500",
    };

    rsx! {
        div { class: "bg-white rounded-xl shadow-sm p-5 hover:shadow-md transition-shadow",
            div { class: "flex items-center",
                div { class: "p-3 rounded-lg {bg_color} text-white",
                    {children}
                }
                div { class: "ml-4",
                    p { class: "text-sm text-gray-500", {title} }
                    p { class: "text-2xl font-bold text-gray-800", {value} }
                }
            }
        }
    }
}

/// 状态徽章
#[component]
fn StatusBadge(status: Option<String>) -> Element {
    let status_text = status.unwrap_or_else(|| "未知".to_string());
    let (bg_class, text_class) = match status_text.as_str() {
        "待审核" => ("bg-yellow-100", "text-yellow-800"),
        "已批准" => ("bg-green-100", "text-green-800"),
        "已拒绝" => ("bg-red-100", "text-red-800"),
        _ => ("bg-gray-100", "text-gray-800"),
    };

    rsx! {
        span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {bg_class} {text_class}",
            if status_text == "待审核" {
                Icon { icon: FaClock, width: 12, height: 12 }
            } else if status_text == "已批准" {
                Icon { icon: FaCircleCheck, width: 12, height: 12 }
            } else if status_text == "已拒绝" {
                Icon { icon: FaCircleXmark, width: 12, height: 12 }
            } else {
                Icon { icon: FaClock, width: 12, height: 12 }
            }
            span { class: "ml-1", {status_text} }
        }
    }
}

/// 交付状态徽章
#[component]
fn DeliveryStatusBadge(status: Option<String>) -> Element {
    let status_text = status.unwrap_or_else(|| "未知".to_string());
    let bg_class = match status_text.as_str() {
        "待交付" => "bg-gray-100 text-gray-800",
        "交付中" => "bg-blue-100 text-blue-800",
        "已交付" => "bg-green-100 text-green-800",
        _ => "bg-gray-100 text-gray-800",
    };

    rsx! {
        span { class: "px-2 py-1 text-xs font-medium rounded-full {bg_class}",
            {status_text}
        }
    }
}

/// 添加资源模态框
#[component]
fn AddResourceModal(on_close: EventHandler<()>, on_save: EventHandler<BusinessResource>) -> Element {
    let mut current_step = use_signal(|| 0);
    let mut resource_type = use_signal(|| "cloud".to_string());
    let mut ecs_name = use_signal(String::new);
    let mut customer_name = use_signal(String::new);
    let mut cloud_region = use_signal(|| "华东1-杭州".to_string());
    let mut cloud_category = use_signal(|| "阿里云".to_string());
    let mut cpu_cores = use_signal(|| 4u32);
    let mut memory_gb = use_signal(|| 8u32);
    let mut ip_address = use_signal(String::new);
    // 新增字段
    let mut applicant = use_signal(String::new);
    let mut department = use_signal(String::new);
    let mut project_name = use_signal(String::new);
    let mut project_code = use_signal(String::new);
    let mut business_owner = use_signal(String::new);
    let mut tech_owner = use_signal(String::new);
    let mut contact_phone = use_signal(String::new);
    let mut bandwidth_mbps = use_signal(|| 100u32);
    let mut bandwidth_type = use_signal(|| "按量".to_string());
    let mut billing_method = use_signal(|| "包年包月".to_string());
    let mut purchase_duration = use_signal(|| 12u32);
    let mut cost_center = use_signal(String::new);
    let mut security_level = use_signal(|| "二级".to_string());
    let mut data_sensitivity = use_signal(|| "内部".to_string());
    let mut purpose = use_signal(String::new);
    let mut remarks = use_signal(String::new);

    let steps = ["基本信息", "配置信息", "业务关联", "费用与合规"];

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden",
                // 头部
                div { class: "px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-blue-600 to-blue-700",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-semibold text-white", "新建资源申请" }
                        button {
                            class: "text-white hover:text-gray-200 text-2xl",
                            onclick: move |_| on_close.call(()),
                            "×"
                        }
                    }
                    // 步骤指示器
                    div { class: "flex items-center mt-4 space-x-4",
                        for (i, step) in steps.iter().enumerate() {
                            div { class: "flex items-center",
                                div {
                                    class: if i <= *current_step.read() {
                                        "w-8 h-8 rounded-full bg-white text-blue-600 flex items-center justify-center font-semibold text-sm"
                                    } else {
                                        "w-8 h-8 rounded-full bg-blue-400 text-white flex items-center justify-center font-semibold text-sm"
                                    },
                                    {(i + 1).to_string()}
                                }
                                if i < steps.len() - 1 {
                                    div { class: "w-12 h-0.5 bg-blue-400 mx-2" }
                                }
                            }
                        }
                    }
                }

                // 内容区域
                div { class: "p-6 overflow-y-auto max-h-[60vh]",
                    // 步骤0: 基本信息
                    if *current_step.read() == 0 {
                        div { class: "space-y-6",
                            // 资源类型选择
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-3", "资源类型" }
                                div { class: "grid grid-cols-2 gap-4",
                                    button {
                                        class: if *resource_type.read() == "cloud" {
                                            "p-4 border-2 border-blue-500 rounded-xl bg-blue-50"
                                        } else {
                                            "p-4 border border-gray-300 rounded-xl hover:border-blue-300"
                                        },
                                        onclick: move |_| resource_type.set("cloud".to_string()),
                                        div { class: "flex items-center",
                                            Icon { icon: FaCloud, width: 32, height: 32 }
                                            div { class: "ml-3 text-left",
                                                div { class: "font-medium", "云服务器" }
                                                div { class: "text-sm text-gray-500", "ECS/云主机" }
                                            }
                                        }
                                    }
                                    button {
                                        class: if *resource_type.read() == "physical" {
                                            "p-4 border-2 border-blue-500 rounded-xl bg-blue-50"
                                        } else {
                                            "p-4 border border-gray-300 rounded-xl hover:border-blue-300"
                                        },
                                        onclick: move |_| resource_type.set("physical".to_string()),
                                        div { class: "flex items-center",
                                            Icon { icon: FaServer, width: 32, height: 32 }
                                            div { class: "ml-3 text-left",
                                                div { class: "font-medium", "物理机" }
                                                div { class: "text-sm text-gray-500", "独立服务器" }
                                            }
                                        }
                                    }
                                }
                            }

                            // 基本信息
                            div { class: "grid grid-cols-2 gap-4",
                                InputField {
                                    label: "资源名称".to_string(),
                                    placeholder: "输入资源名称".to_string(),
                                    value: ecs_name,
                                    required: true,
                                }
                                InputField {
                                    label: "客户名称".to_string(),
                                    placeholder: "输入客户名称".to_string(),
                                    value: customer_name,
                                    required: true,
                                }
                                InputField {
                                    label: "IP地址".to_string(),
                                    placeholder: "例如: 192.168.1.1".to_string(),
                                    value: ip_address,
                                    required: true,
                                }
                                SelectField {
                                    label: "云区域".to_string(),
                                    options: vec!["华东1-杭州", "华东2-上海", "华北2-北京", "华南1-深圳", "香港", "新加坡"],
                                    value: cloud_region,
                                }
                            }

                            // 申请信息
                            div { class: "border-t pt-4 mt-4",
                                h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                    Icon { icon: FaUser, width: 16, height: 16 }
                                    span { class: "ml-2", "申请信息" }
                                }
                                div { class: "grid grid-cols-2 gap-4",
                                    InputField {
                                        label: "申请人".to_string(),
                                        placeholder: "输入申请人姓名".to_string(),
                                        value: applicant,
                                        required: true,
                                    }
                                    InputField {
                                        label: "申请部门".to_string(),
                                        placeholder: "输入申请部门".to_string(),
                                        value: department,
                                        required: true,
                                    }
                                }
                            }
                        }
                    }

                    // 步骤1: 配置信息
                    if *current_step.read() == 1 {
                        div { class: "space-y-6",
                            h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                Icon { icon: FaServer, width: 16, height: 16 }
                                span { class: "ml-2", "资源配置" }
                            }
                            div { class: "grid grid-cols-3 gap-4",
                                NumberField {
                                    label: "CPU核数".to_string(),
                                    value: cpu_cores,
                                    min: 1,
                                    max: 128,
                                }
                                NumberField {
                                    label: "内存(GB)".to_string(),
                                    value: memory_gb,
                                    min: 1,
                                    max: 1024,
                                }
                                SelectField {
                                    label: "云服务商".to_string(),
                                    options: vec!["阿里云", "腾讯云", "华为云", "AWS", "Azure"],
                                    value: cloud_category,
                                }
                            }

                            // 网络配置
                            div { class: "border-t pt-4 mt-4",
                                h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                    Icon { icon: FaNetworkWired, width: 16, height: 16 }
                                    span { class: "ml-2", "网络配置" }
                                }
                                div { class: "grid grid-cols-3 gap-4",
                                    NumberField {
                                        label: "带宽(Mbps)".to_string(),
                                        value: bandwidth_mbps,
                                        min: 1,
                                        max: 10000,
                                    }
                                    SelectField {
                                        label: "带宽类型".to_string(),
                                        options: vec!["按量", "包月"],
                                        value: bandwidth_type,
                                    }
                                    NumberField {
                                        label: "公网IP数量".to_string(),
                                        value: use_signal(|| 1u32),
                                        min: 0,
                                        max: 100,
                                    }
                                }
                            }
                        }
                    }

                    // 步骤2: 业务关联
                    if *current_step.read() == 2 {
                        div { class: "space-y-6",
                            h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                Icon { icon: FaBuilding, width: 16, height: 16 }
                                span { class: "ml-2", "项目信息" }
                            }
                            div { class: "grid grid-cols-2 gap-4",
                                InputField {
                                    label: "项目名称".to_string(),
                                    placeholder: "输入项目名称".to_string(),
                                    value: project_name,
                                    required: false,
                                }
                                InputField {
                                    label: "项目编号".to_string(),
                                    placeholder: "输入项目编号".to_string(),
                                    value: project_code,
                                    required: false,
                                }
                            }

                            div { class: "border-t pt-4 mt-4",
                                h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                    Icon { icon: FaUser, width: 16, height: 16 }
                                    span { class: "ml-2", "负责人信息" }
                                }
                                div { class: "grid grid-cols-2 gap-4",
                                    InputField {
                                        label: "业务负责人".to_string(),
                                        placeholder: "输入业务负责人".to_string(),
                                        value: business_owner,
                                        required: false,
                                    }
                                    InputField {
                                        label: "技术负责人".to_string(),
                                        placeholder: "输入技术负责人".to_string(),
                                        value: tech_owner,
                                        required: false,
                                    }
                                    InputField {
                                        label: "联系电话".to_string(),
                                        placeholder: "输入联系电话".to_string(),
                                        value: contact_phone,
                                        required: false,
                                    }
                                }
                            }

                            div { class: "border-t pt-4 mt-4",
                                h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                    Icon { icon: FaFileLines, width: 16, height: 16 }
                                    span { class: "ml-2", "用途说明" }
                                }
                                textarea {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                    rows: 4,
                                    placeholder: "请输入资源用途说明...",
                                    value: purpose,
                                    oninput: move |e| purpose.set(e.value()),
                                }
                            }
                        }
                    }

                    // 步骤3: 费用与合规
                    if *current_step.read() == 3 {
                        div { class: "space-y-6",
                            h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                Icon { icon: FaMoneyBill, width: 16, height: 16 }
                                span { class: "ml-2", "费用信息" }
                            }
                            div { class: "grid grid-cols-3 gap-4",
                                SelectField {
                                    label: "计费方式".to_string(),
                                    options: vec!["包年包月", "按量付费"],
                                    value: billing_method,
                                }
                                NumberField {
                                    label: "购买时长(月)".to_string(),
                                    value: purchase_duration,
                                    min: 1,
                                    max: 36,
                                }
                                InputField {
                                    label: "成本中心".to_string(),
                                    placeholder: "输入成本中心".to_string(),
                                    value: cost_center,
                                    required: false,
                                }
                            }

                            div { class: "border-t pt-4 mt-4",
                                h4 { class: "text-sm font-medium text-gray-700 mb-3 flex items-center",
                                    Icon { icon: FaShield, width: 16, height: 16 }
                                    span { class: "ml-2", "合规信息" }
                                }
                                div { class: "grid grid-cols-2 gap-4",
                                    SelectField {
                                        label: "等保级别".to_string(),
                                        options: vec!["一级", "二级", "三级", "四级"],
                                        value: security_level,
                                    }
                                    SelectField {
                                        label: "数据敏感级别".to_string(),
                                        options: vec!["公开", "内部", "机密", "绝密"],
                                        value: data_sensitivity,
                                    }
                                }
                            }

                            div { class: "border-t pt-4 mt-4",
                                h4 { class: "text-sm font-medium text-gray-700 mb-3", "备注" }
                                textarea {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                    rows: 3,
                                    placeholder: "其他备注信息...",
                                    value: remarks,
                                    oninput: move |e| remarks.set(e.value()),
                                }
                            }
                        }
                    }
                }

                // 底部按钮
                div { class: "px-6 py-4 border-t border-gray-200 bg-gray-50 flex justify-between",
                    if *current_step.read() > 0 {
                        button {
                            class: "px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100",
                            onclick: {
                                let mut current_step = current_step.clone();
                                move |_| {
                                    let step = *current_step.read();
                                    current_step.set(step.saturating_sub(1));
                                }
                            },
                            "上一步"
                        }
                    } else {
                        div {}
                    }

                    div { class: "flex space-x-3",
                        button {
                            class: "px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }
                        if *current_step.read() < 3 {
                            button {
                                class: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                                onclick: {
                                    let mut current_step = current_step.clone();
                                    move |_| {
                                        let step = *current_step.read();
                                        current_step.set(step + 1);
                                    }
                                },
                                "下一步"
                            }
                        } else {
                            button {
                                class: "px-6 py-2 bg-gradient-to-r from-blue-600 to-blue-700 text-white rounded-lg hover:from-blue-700 hover:to-blue-800",
                                onclick: move |_| {
                                    let new_resource = BusinessResource {
                                        id: Some(chrono::Utc::now().timestamp() as i32),
                                        resource_type: resource_type.read().clone(),
                                        ecs_name: ecs_name.read().clone(),
                                        ecs_status: "运行中".to_string(),
                                        resource_id: String::new(),
                                        cloud_region: cloud_region.read().clone(),
                                        cloud_category: cloud_category.read().clone(),
                                        cloud_provider_config_id: None,
                                        zone_name: None,
                                        platform_name: None,
                                        county_city: None,
                                        vdc_name: None,
                                        customer_name: customer_name.read().clone(),
                                        application_name: None,
                                        contract_name: None,
                                        instance_id: String::new(),
                                        ecs_type: "ecs.g6.large".to_string(),
                                        ecs_os: "CentOS 7.9".to_string(),
                                        cpu_cores: *cpu_cores.read(),
                                        memory_gb: *memory_gb.read(),
                                        system_disk: "cloud_essd".to_string(),
                                        system_disk_size_gb: 40,
                                        data_disk: None,
                                        completion_time: None,
                                        release_time: None,
                                        has_security_product: false,
                                        ip_address: ip_address.read().clone(),
                                        ecs_login_method: None,
                                        ecs_login_username: None,
                                        ecs_initial_password: None,
                                        bastion_address: None,
                                        bastion_admin_account: None,
                                        bastion_initial_password: None,
                                        physical_machine_info: None,
                                        cloud_vm_info: None,
                                        remarks: Some(remarks.read().clone()),
                                        created_at: Some(chrono::Utc::now()),
                                        updated_at: Some(chrono::Utc::now()),
                                        created_by: Some(applicant.read().clone()),
                                        updated_by: Some(applicant.read().clone()),
                                        applicant: Some(applicant.read().clone()),
                                        department: Some(department.read().clone()),
                                        approver: None,
                                        approval_time: None,
                                        approval_remarks: None,
                                        rejection_reason: None,
                                        bandwidth_mbps: Some(*bandwidth_mbps.read()),
                                        bandwidth_type: Some(bandwidth_type.read().clone()),
                                        public_ip_count: Some(1),
                                        network_type: Some("VPC".to_string()),
                                        project_name: Some(project_name.read().clone()),
                                        project_code: Some(project_code.read().clone()),
                                        business_owner: Some(business_owner.read().clone()),
                                        tech_owner: Some(tech_owner.read().clone()),
                                        contact_phone: Some(contact_phone.read().clone()),
                                        billing_method: Some(billing_method.read().clone()),
                                        purchase_duration: Some(*purchase_duration.read()),
                                        cost_center: Some(cost_center.read().clone()),
                                        security_level: Some(security_level.read().clone()),
                                        data_sensitivity: Some(data_sensitivity.read().clone()),
                                        purpose: Some(purpose.read().clone()),
                                        expected_delivery_time: None,
                                        application_status: Some("待审核".to_string()),
                                        delivery_status: Some("待交付".to_string()),
                                        delivery_confirmed_at: None,
                                        delivery_confirmed_by: None,
                                    };
                                    on_save.call(new_resource);
                                },
                                "提交申请"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 资源详情模态框
#[component]
fn ResourceDetailModal(resource: BusinessResource, on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden",
                // 头部
                div { class: "px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-blue-600 to-blue-700",
                    div { class: "flex justify-between items-center",
                        div {
                            h3 { class: "text-xl font-semibold text-white", {resource.ecs_name.clone()} }
                            p { class: "text-blue-200 text-sm mt-1", {resource.customer_name.clone()} }
                        }
                        button {
                            class: "text-white hover:text-gray-200 text-2xl",
                            onclick: move |_| on_close.call(()),
                            "×"
                        }
                    }
                }

                // 内容区域 - 分标签页
                div { class: "p-6 overflow-y-auto max-h-[70vh]",
                    // 基本信息
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaServer, width: 16, height: 16 }
                            span { class: "ml-2", "基本信息" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "资源类型", value: if resource.resource_type == "cloud" { "云服务器" } else { "物理机" } }
                            DetailItem { label: "资源名称", value: resource.ecs_name.clone() }
                            DetailItem { label: "客户名称", value: resource.customer_name.clone() }
                            DetailItem { label: "IP地址", value: resource.ip_address.clone() }
                            DetailItem { label: "云区域", value: resource.cloud_region.clone() }
                            DetailItem { label: "云服务商", value: resource.cloud_category.clone() }
                            DetailItem { label: "实例类型", value: resource.ecs_type.clone() }
                            DetailItem { label: "操作系统", value: resource.ecs_os.clone() }
                        }
                    }

                    // 配置信息
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaServer, width: 16, height: 16 }
                            span { class: "ml-2", "配置信息" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "CPU核数", value: format!("{}核", resource.cpu_cores) }
                            DetailItem { label: "内存", value: format!("{}GB", resource.memory_gb) }
                            DetailItem { label: "系统盘", value: resource.system_disk.clone() }
                            DetailItem { label: "系统盘大小", value: format!("{}GB", resource.system_disk_size_gb) }
                        }
                    }

                    // 申请流程
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaUser, width: 16, height: 16 }
                            span { class: "ml-2", "申请流程" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "申请人", value: resource.applicant.clone().unwrap_or_default() }
                            DetailItem { label: "申请部门", value: resource.department.clone().unwrap_or_default() }
                            DetailItem { label: "审批人", value: resource.approver.clone().unwrap_or_default() }
                            DetailItem { label: "申请状态", value: resource.application_status.clone().unwrap_or_default() }
                            DetailItem { label: "审批备注", value: resource.approval_remarks.clone().unwrap_or_default() }
                            DetailItem { label: "拒绝原因", value: resource.rejection_reason.clone().unwrap_or_default() }
                        }
                    }

                    // 网络配置
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaNetworkWired, width: 16, height: 16 }
                            span { class: "ml-2", "网络配置" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "带宽", value: format!("{}Mbps", resource.bandwidth_mbps.unwrap_or(0)) }
                            DetailItem { label: "带宽类型", value: resource.bandwidth_type.clone().unwrap_or_default() }
                            DetailItem { label: "公网IP数量", value: resource.public_ip_count.unwrap_or(0).to_string() }
                            DetailItem { label: "网络类型", value: resource.network_type.clone().unwrap_or_default() }
                        }
                    }

                    // 业务关联
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaBuilding, width: 16, height: 16 }
                            span { class: "ml-2", "业务关联" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "项目名称", value: resource.project_name.clone().unwrap_or_default() }
                            DetailItem { label: "项目编号", value: resource.project_code.clone().unwrap_or_default() }
                            DetailItem { label: "业务负责人", value: resource.business_owner.clone().unwrap_or_default() }
                            DetailItem { label: "技术负责人", value: resource.tech_owner.clone().unwrap_or_default() }
                            DetailItem { label: "联系电话", value: resource.contact_phone.clone().unwrap_or_default() }
                        }
                    }

                    // 费用信息
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaMoneyBill, width: 16, height: 16 }
                            span { class: "ml-2", "费用信息" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "计费方式", value: resource.billing_method.clone().unwrap_or_default() }
                            DetailItem { label: "购买时长", value: format!("{}月", resource.purchase_duration.unwrap_or(0)) }
                            DetailItem { label: "成本中心", value: resource.cost_center.clone().unwrap_or_default() }
                        }
                    }

                    // 合规信息
                    div { class: "mb-6",
                        h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                            Icon { icon: FaShield, width: 16, height: 16 }
                            span { class: "ml-2", "合规信息" }
                        }
                        div { class: "grid grid-cols-4 gap-4",
                            DetailItem { label: "等保级别", value: resource.security_level.clone().unwrap_or_default() }
                            DetailItem { label: "数据敏感级别", value: resource.data_sensitivity.clone().unwrap_or_default() }
                        }
                    }

                    // 用途说明
                    if let Some(purpose) = &resource.purpose {
                        if !purpose.is_empty() {
                            div { class: "mb-6",
                                h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b flex items-center",
                                    Icon { icon: FaFileLines, width: 16, height: 16 }
                                    span { class: "ml-2", "用途说明" }
                                }
                                p { class: "text-gray-600 text-sm", {purpose.clone()} }
                            }
                        }
                    }

                    // 备注
                    if let Some(remarks) = &resource.remarks {
                        if !remarks.is_empty() {
                            div { class: "mb-6",
                                h4 { class: "text-sm font-semibold text-gray-800 mb-3 pb-2 border-b", "备注" }
                                p { class: "text-gray-600 text-sm", {remarks.clone()} }
                            }
                        }
                    }
                }

                // 底部
                div { class: "px-6 py-4 border-t border-gray-200 bg-gray-50 flex justify-end",
                    button {
                        class: "px-6 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200",
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                }
            }
        }
    }
}

/// 详情项组件
#[component]
fn DetailItem(label: String, value: String) -> Element {
    rsx! {
        div { class: "mb-3",
            p { class: "text-xs text-gray-500 mb-1", {label} }
            p { class: "text-sm text-gray-800 font-medium", {value} }
        }
    }
}

/// 编辑资源模态框
#[component]
fn EditResourceModal(resource: BusinessResource, on_close: EventHandler<()>, on_save: EventHandler<BusinessResource>) -> Element {
    let ecs_name = use_signal(|| resource.ecs_name.clone());
    let customer_name = use_signal(|| resource.customer_name.clone());
    let ip_address = use_signal(|| resource.ip_address.clone());
    let cpu_cores = use_signal(|| resource.cpu_cores);
    let memory_gb = use_signal(|| resource.memory_gb);
    let application_status = use_signal(|| resource.application_status.clone().unwrap_or_default());
    let delivery_status = use_signal(|| resource.delivery_status.clone().unwrap_or_default());
    let approver = use_signal(|| resource.approver.clone().unwrap_or_default());
    let mut approval_remarks = use_signal(|| resource.approval_remarks.clone().unwrap_or_default());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden",
                // 头部
                div { class: "px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-green-600 to-green-700",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-semibold text-white", "编辑资源" }
                        button {
                            class: "text-white hover:text-gray-200 text-2xl",
                            onclick: move |_| on_close.call(()),
                            "×"
                        }
                    }
                }

                // 内容
                div { class: "p-6 overflow-y-auto max-h-[60vh] space-y-4",
                    // 基本信息
                    div { class: "grid grid-cols-2 gap-4",
                        InputField {
                            label: "资源名称".to_string(),
                            placeholder: "输入资源名称".to_string(),
                            value: ecs_name,
                            required: true,
                        }
                        InputField {
                            label: "客户名称".to_string(),
                            placeholder: "输入客户名称".to_string(),
                            value: customer_name,
                            required: true,
                        }
                        InputField {
                            label: "IP地址".to_string(),
                            placeholder: "输入IP地址".to_string(),
                            value: ip_address,
                            required: true,
                        }
                    }

                    // 配置
                    div { class: "grid grid-cols-2 gap-4",
                        NumberField {
                            label: "CPU核数".to_string(),
                            value: cpu_cores,
                            min: 1,
                            max: 128,
                        }
                        NumberField {
                            label: "内存(GB)".to_string(),
                            value: memory_gb,
                            min: 1,
                            max: 1024,
                        }
                    }

                    // 审批状态
                    div { class: "border-t pt-4",
                        h4 { class: "text-sm font-medium text-gray-700 mb-3", "审批管理" }
                        div { class: "grid grid-cols-2 gap-4",
                            SelectFieldWithCustom {
                                label: "申请状态".to_string(),
                                options: vec!["待审核", "已批准", "已拒绝"],
                                value: application_status,
                            }
                            SelectFieldWithCustom {
                                label: "交付状态".to_string(),
                                options: vec!["待交付", "交付中", "已交付"],
                                value: delivery_status,
                            }
                            InputField {
                                label: "审批人".to_string(),
                                placeholder: "输入审批人".to_string(),
                                value: approver,
                                required: false,
                            }
                        }
                        div { class: "mt-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "审批备注" }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                rows: 3,
                                value: approval_remarks,
                                oninput: move |e| approval_remarks.set(e.value()),
                            }
                        }
                    }
                }

                // 底部
                div { class: "px-6 py-4 border-t border-gray-200 bg-gray-50 flex justify-end space-x-3",
                    button {
                        class: "px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-6 py-2 bg-gradient-to-r from-green-600 to-green-700 text-white rounded-lg hover:from-green-700 hover:to-green-800",
                        onclick: move |_| {
                            let mut updated = resource.clone();
                            updated.ecs_name = ecs_name.read().clone();
                            updated.customer_name = customer_name.read().clone();
                            updated.ip_address = ip_address.read().clone();
                            updated.cpu_cores = *cpu_cores.read();
                            updated.memory_gb = *memory_gb.read();
                            updated.application_status = Some(application_status.read().clone());
                            updated.delivery_status = Some(delivery_status.read().clone());
                            updated.approver = Some(approver.read().clone());
                            updated.approval_remarks = Some(approval_remarks.read().clone());
                            updated.updated_at = Some(chrono::Utc::now());
                            on_save.call(updated);
                        },
                        "保存更改"
                    }
                }
            }
        }
    }
}

/// 输入框组件
#[component]
fn InputField(label: String, placeholder: String, value: Signal<String>, required: bool) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-medium text-gray-700 mb-1",
                {label.clone()}
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            input {
                r#type: "text",
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                placeholder: placeholder,
                value: value,
                oninput: move |e| value.set(e.value()),
            }
        }
    }
}

/// 数字输入框组件
#[component]
fn NumberField(label: String, value: Signal<u32>, min: u32, max: u32) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-medium text-gray-700 mb-1", {label.clone()} }
            input {
                r#type: "number",
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                min: min,
                max: max,
                value: value,
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<u32>() {
                        value.set(v);
                    }
                },
            }
        }
    }
}

/// 下拉选择框组件
#[component]
fn SelectField(label: String, options: Vec<&'static str>, value: Signal<String>) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-medium text-gray-700 mb-1", {label.clone()} }
            select {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                value: value,
                onchange: move |e| value.set(e.value()),
                for option in options {
                    option { value: option, {option} }
                }
            }
        }
    }
}

/// 下拉选择框组件 (带自定义值支持)
#[component]
fn SelectFieldWithCustom(label: String, options: Vec<&'static str>, value: Signal<String>) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-medium text-gray-700 mb-1", {label.clone()} }
            select {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                value: value,
                onchange: move |e| value.set(e.value()),
                for option in options {
                    option { value: option, {option} }
                }
            }
        }
    }
}

/// 创建示例数据
fn create_sample_data() -> Vec<BusinessResource> {
    vec![
        BusinessResource {
            id: Some(1),
            resource_type: "cloud".to_string(),
            ecs_name: "生产环境Web服务器-01".to_string(),
            ecs_status: "运行中".to_string(),
            resource_id: "i-bp1234567890".to_string(),
            cloud_region: "华东1-杭州".to_string(),
            cloud_category: "阿里云".to_string(),
            cloud_provider_config_id: None,
            zone_name: Some("华东区".to_string()),
            platform_name: Some("公众云".to_string()),
            county_city: Some("杭州市".to_string()),
            vdc_name: Some("VDC-001".to_string()),
            customer_name: "科技有限公司".to_string(),
            application_name: Some("电商平台".to_string()),
            contract_name: Some("2024年度云服务合同".to_string()),
            instance_id: "i-bp1234567890".to_string(),
            ecs_type: "ecs.g6.xlarge".to_string(),
            ecs_os: "CentOS 7.9".to_string(),
            cpu_cores: 4,
            memory_gb: 16,
            system_disk: "cloud_essd".to_string(),
            system_disk_size_gb: 100,
            data_disk: Some("500GB ESSD".to_string()),
            completion_time: None,
            release_time: None,
            has_security_product: true,
            ip_address: "192.168.1.100".to_string(),
            ecs_login_method: Some("SSH".to_string()),
            ecs_login_username: Some("root".to_string()),
            ecs_initial_password: None,
            bastion_address: Some("bastion.example.com".to_string()),
            bastion_admin_account: Some("admin".to_string()),
            bastion_initial_password: None,
            physical_machine_info: None,
            cloud_vm_info: None,
            remarks: Some("生产环境核心服务器".to_string()),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            created_by: Some("张三".to_string()),
            updated_by: Some("张三".to_string()),
            applicant: Some("张三".to_string()),
            department: Some("技术部".to_string()),
            approver: Some("李四".to_string()),
            approval_time: Some(chrono::Utc::now()),
            approval_remarks: Some("批准创建".to_string()),
            rejection_reason: None,
            bandwidth_mbps: Some(100),
            bandwidth_type: Some("包月".to_string()),
            public_ip_count: Some(1),
            network_type: Some("VPC".to_string()),
            project_name: Some("电商平台升级".to_string()),
            project_code: Some("PRJ-2024-001".to_string()),
            business_owner: Some("王五".to_string()),
            tech_owner: Some("赵六".to_string()),
            contact_phone: Some("13800138000".to_string()),
            billing_method: Some("包年包月".to_string()),
            purchase_duration: Some(12),
            cost_center: Some("CC-001".to_string()),
            security_level: Some("三级".to_string()),
            data_sensitivity: Some("机密".to_string()),
            purpose: Some("电商平台生产环境Web服务器，承载核心交易系统".to_string()),
            expected_delivery_time: None,
            application_status: Some("已批准".to_string()),
            delivery_status: Some("已交付".to_string()),
            delivery_confirmed_at: Some(chrono::Utc::now()),
            delivery_confirmed_by: Some("王五".to_string()),
        },
        BusinessResource {
            id: Some(2),
            resource_type: "physical".to_string(),
            ecs_name: "数据库物理机-DB01".to_string(),
            ecs_status: "运行中".to_string(),
            resource_id: "".to_string(),
            cloud_region: "北京机房A区".to_string(),
            cloud_category: "自建机房".to_string(),
            cloud_provider_config_id: None,
            zone_name: Some("华北区".to_string()),
            platform_name: None,
            county_city: Some("北京市".to_string()),
            vdc_name: None,
            customer_name: "科技有限公司".to_string(),
            application_name: Some("核心数据库".to_string()),
            contract_name: None,
            instance_id: "SN2024PH001".to_string(),
            ecs_type: "Dell PowerEdge R740".to_string(),
            ecs_os: "CentOS 8.4".to_string(),
            cpu_cores: 32,
            memory_gb: 128,
            system_disk: "SSD RAID1".to_string(),
            system_disk_size_gb: 480,
            data_disk: Some("4TB SSD RAID10".to_string()),
            completion_time: None,
            release_time: None,
            has_security_product: true,
            ip_address: "10.0.0.50".to_string(),
            ecs_login_method: Some("堡垒机".to_string()),
            ecs_login_username: Some("admin".to_string()),
            ecs_initial_password: None,
            bastion_address: None,
            bastion_admin_account: None,
            bastion_initial_password: None,
            physical_machine_info: None,
            cloud_vm_info: None,
            remarks: Some("核心数据库服务器".to_string()),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            created_by: Some("张三".to_string()),
            updated_by: Some("张三".to_string()),
            applicant: Some("张三".to_string()),
            department: Some("技术部".to_string()),
            approver: None,
            approval_time: None,
            approval_remarks: None,
            rejection_reason: None,
            bandwidth_mbps: Some(1000),
            bandwidth_type: Some("专线".to_string()),
            public_ip_count: Some(0),
            network_type: Some("内网".to_string()),
            project_name: Some("数据库扩容".to_string()),
            project_code: Some("PRJ-2024-002".to_string()),
            business_owner: Some("王五".to_string()),
            tech_owner: Some("赵六".to_string()),
            contact_phone: Some("13800138001".to_string()),
            billing_method: None,
            purchase_duration: None,
            cost_center: Some("CC-002".to_string()),
            security_level: Some("三级".to_string()),
            data_sensitivity: Some("绝密".to_string()),
            purpose: Some("承载核心交易数据库，需要高性能存储".to_string()),
            expected_delivery_time: None,
            application_status: Some("待审核".to_string()),
            delivery_status: Some("待交付".to_string()),
            delivery_confirmed_at: None,
            delivery_confirmed_by: None,
        },
        BusinessResource {
            id: Some(3),
            resource_type: "cloud".to_string(),
            ecs_name: "测试环境应用服务器".to_string(),
            ecs_status: "运行中".to_string(),
            resource_id: "i-bp0987654321".to_string(),
            cloud_region: "华南1-深圳".to_string(),
            cloud_category: "腾讯云".to_string(),
            cloud_provider_config_id: None,
            zone_name: Some("华南区".to_string()),
            platform_name: None,
            county_city: Some("深圳市".to_string()),
            vdc_name: None,
            customer_name: "科技有限公司".to_string(),
            application_name: Some("测试平台".to_string()),
            contract_name: None,
            instance_id: "i-bp0987654321".to_string(),
            ecs_type: "ecs.g6.large".to_string(),
            ecs_os: "Ubuntu 20.04".to_string(),
            cpu_cores: 2,
            memory_gb: 4,
            system_disk: "cloud_ssd".to_string(),
            system_disk_size_gb: 50,
            data_disk: None,
            completion_time: None,
            release_time: None,
            has_security_product: false,
            ip_address: "192.168.2.200".to_string(),
            ecs_login_method: Some("SSH".to_string()),
            ecs_login_username: Some("ubuntu".to_string()),
            ecs_initial_password: None,
            bastion_address: None,
            bastion_admin_account: None,
            bastion_initial_password: None,
            physical_machine_info: None,
            cloud_vm_info: None,
            remarks: None,
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            created_by: Some("李四".to_string()),
            updated_by: Some("李四".to_string()),
            applicant: Some("李四".to_string()),
            department: Some("研发部".to_string()),
            approver: Some("张三".to_string()),
            approval_time: Some(chrono::Utc::now()),
            approval_remarks: Some("测试环境使用".to_string()),
            rejection_reason: None,
            bandwidth_mbps: Some(10),
            bandwidth_type: Some("按量".to_string()),
            public_ip_count: Some(1),
            network_type: Some("VPC".to_string()),
            project_name: Some("新产品测试".to_string()),
            project_code: Some("PRJ-2024-003".to_string()),
            business_owner: Some("李四".to_string()),
            tech_owner: Some("李四".to_string()),
            contact_phone: Some("13900139000".to_string()),
            billing_method: Some("按量付费".to_string()),
            purchase_duration: None,
            cost_center: Some("CC-003".to_string()),
            security_level: Some("二级".to_string()),
            data_sensitivity: Some("内部".to_string()),
            purpose: Some("新产品功能测试环境".to_string()),
            expected_delivery_time: None,
            application_status: Some("已批准".to_string()),
            delivery_status: Some("交付中".to_string()),
            delivery_confirmed_at: None,
            delivery_confirmed_by: None,
        },
    ]
}
