use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaMagnifyingGlass, FaCloud, FaCheck, FaClock, FaXmark,
    FaCircleCheck, FaCircleXmark, FaPen, FaEye, FaShieldHalved
};
use crate::app::CLOUD_PLATFORMS_STATE;
use crate::app::PROVIDERS_STATE;
use crate::app::SECURITY_PRODUCTS_STATE;
use crate::components::security_product::security_product_selector::SelectedSecurityProducts;
use crate::state::security_product::{SecurityProductCategory, SecurityProductStatus};

/// 云服务申请状态
#[derive(Clone, Debug, PartialEq)]
pub enum CloudServiceStatus {
    Pending,    // 待审批
    Approved,   // 已批准
    Rejected,   // 已拒绝
    Processing, // 处理中
    Completed,  // 已完成
}

impl CloudServiceStatus {
    fn display_name(&self) -> &'static str {
        match self {
            CloudServiceStatus::Pending => "待审批",
            CloudServiceStatus::Approved => "已批准",
            CloudServiceStatus::Rejected => "已拒绝",
            CloudServiceStatus::Processing => "处理中",
            CloudServiceStatus::Completed => "已完成",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            CloudServiceStatus::Pending => "bg-yellow-100 text-yellow-800",
            CloudServiceStatus::Approved => "bg-green-100 text-green-800",
            CloudServiceStatus::Rejected => "bg-red-100 text-red-800",
            CloudServiceStatus::Processing => "bg-blue-100 text-blue-800",
            CloudServiceStatus::Completed => "bg-gray-100 text-gray-800",
        }
    }
}

/// 云服务申请数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct CloudServiceRequest {
    pub id: i32,
    pub title: String,
    pub applicant: String,
    pub department: String,
    pub provider_id: Option<i32>,
    pub cloud_platform: String,
    pub instance_type: String,
    pub instance_count: i32,
    pub duration: String,
    pub purpose: String,
    pub security_products: SelectedSecurityProducts, // 选中的安全产品
    pub status: CloudServiceStatus,
    pub created_at: String,
}

impl CloudServiceRequest {
    pub fn provider_name(&self) -> String {
        if let Some(pid) = self.provider_id {
            PROVIDERS_STATE.read()
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.short_name.clone())
                .unwrap_or_else(|| format!("服务商{}", pid))
        } else {
            "未分配".to_string()
        }
    }

    pub fn provider_color(&self) -> &'static str {
        if let Some(pid) = self.provider_id {
            match pid {
                1 => "bg-blue-100 text-blue-800",
                2 => "bg-orange-100 text-orange-800",
                3 => "bg-green-100 text-green-800",
                4 => "bg-purple-100 text-purple-800",
                _ => "bg-gray-100 text-gray-800",
            }
        } else {
            "bg-gray-100 text-gray-600"
        }
    }
}

/// 初始化示例云服务申请数据
pub fn init_cloud_service_requests() -> Vec<CloudServiceRequest> {
    vec![
        CloudServiceRequest {
            id: 1,
            title: "OA系统云服务器扩容".to_string(),
            applicant: "张三".to_string(),
            department: "信息部".to_string(),
            provider_id: Some(1),
            cloud_platform: "电信-公有云(阿里云)".to_string(),
            instance_type: "ecs.g6.xlarge".to_string(),
            instance_count: 2,
            duration: "1年".to_string(),
            purpose: "OA系统用户增加，需要扩容服务器".to_string(),
            security_products: SelectedSecurityProducts::new(),
            status: CloudServiceStatus::Pending,
            created_at: "2024-03-01 10:30".to_string(),
        },
        CloudServiceRequest {
            id: 2,
            title: "ERP系统数据库云主机".to_string(),
            applicant: "李四".to_string(),
            department: "财务部".to_string(),
            provider_id: Some(2),
            cloud_platform: "联通-公有云(华为云)".to_string(),
            instance_type: "ecs.g6.2xlarge".to_string(),
            instance_count: 1,
            duration: "2年".to_string(),
            purpose: "ERP系统迁移上云".to_string(),
            security_products: SelectedSecurityProducts::new(),
            status: CloudServiceStatus::Approved,
            created_at: "2024-02-28 14:20".to_string(),
        },
        CloudServiceRequest {
            id: 3,
            title: "测试环境云主机".to_string(),
            applicant: "王五".to_string(),
            department: "研发部".to_string(),
            provider_id: Some(3),
            cloud_platform: "移动-公有云(阿里云)".to_string(),
            instance_type: "ecs.g6.large".to_string(),
            instance_count: 3,
            duration: "6个月".to_string(),
            purpose: "新项目测试环境".to_string(),
            security_products: SelectedSecurityProducts::new(),
            status: CloudServiceStatus::Processing,
            created_at: "2024-03-02 09:15".to_string(),
        },
        CloudServiceRequest {
            id: 4,
            title: "备份存储空间申请".to_string(),
            applicant: "赵六".to_string(),
            department: "运维部".to_string(),
            provider_id: Some(1),
            cloud_platform: "电信-政务云(阿里云)".to_string(),
            instance_type: "oss.standard".to_string(),
            instance_count: 1,
            duration: "3年".to_string(),
            purpose: "数据备份存储".to_string(),
            security_products: SelectedSecurityProducts::new(),
            status: CloudServiceStatus::Completed,
            created_at: "2024-02-25 16:45".to_string(),
        },
    ]
}

/// 云服务申请页面
#[allow(non_snake_case)]
pub fn CloudServiceRequest() -> Element {
    let mut requests = use_signal(init_cloud_service_requests);
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "全部".to_string());
    let mut show_new_form = use_signal(|| false);

    // 统计数据
    let total_count = requests.read().len() as i32;
    let pending_count = requests.read().iter().filter(|r| r.status == CloudServiceStatus::Pending).count() as i32;
    let processing_count = requests.read().iter().filter(|r| r.status == CloudServiceStatus::Processing).count() as i32;
    let completed_count = requests.read().iter().filter(|r| r.status == CloudServiceStatus::Completed).count() as i32;

    // 过滤请求
    let filtered_requests: Vec<CloudServiceRequest> = requests.read()
        .iter()
        .filter(|req| {
            let query = search_query.read().to_lowercase();
            let matches_search = query.is_empty()
                || req.title.to_lowercase().contains(&query)
                || req.applicant.to_lowercase().contains(&query)
                || req.department.to_lowercase().contains(&query);

            let matches_status = status_filter.read().as_str() == "全部"
                || match status_filter.read().as_str() {
                    "待审批" => req.status == CloudServiceStatus::Pending,
                    "已批准" => req.status == CloudServiceStatus::Approved,
                    "已拒绝" => req.status == CloudServiceStatus::Rejected,
                    "处理中" => req.status == CloudServiceStatus::Processing,
                    "已完成" => req.status == CloudServiceStatus::Completed,
                    _ => true,
                };

            matches_search && matches_status
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                div {
                    div { class: "flex items-center gap-3",
                        Icon { icon: FaCloud, class: "text-blue-600 text-2xl" }
                        div {
                            h1 { class: "text-2xl font-bold text-gray-800", "云服务申请" }
                            p { class: "text-sm text-gray-500 mt-1", "申请云服务器、云数据库等云服务资源" }
                        }
                    }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_new_form.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16, class: "mr-2" }
                    "申请新资源"
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaCloud, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总申请" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-yellow-500",
                            Icon { icon: FaClock, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "待审批" }
                            p { class: "text-xl font-bold text-gray-800", {pending_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-600",
                            Icon { icon: FaCheck, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "处理中" }
                            p { class: "text-xl font-bold text-gray-800", {processing_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaCircleCheck, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已完成" }
                            p { class: "text-xl font-bold text-gray-800", {completed_count.to_string()} }
                        }
                    }
                }
            }

            // 搜索和筛选
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex flex-wrap gap-4",
                    div { class: "relative flex-1 min-w-[200px]",
                        Icon { icon: FaMagnifyingGlass,
                            width: 16,
                            height: 16,
                            class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
                        }
                        input {
                            r#type: "text",
                            class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "搜索申请标题、申请人或部门...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value())
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: status_filter,
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "全部", "全部状态" }
                        option { value: "待审批", "待审批" }
                        option { value: "已批准", "已批准" }
                        option { value: "处理中", "处理中" }
                        option { value: "已完成", "已完成" }
                        option { value: "已拒绝", "已拒绝" }
                    }
                }
            }

            // 申请列表表格
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "申请标题" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "申请人" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "云平台" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "实例类型" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "数量" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "申请时间" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        if filtered_requests.is_empty() {
                            tr {
                                td { colspan: "9", class: "px-6 py-12 text-center text-gray-500",
                                    "没有找到匹配的申请"
                                }
                            }
                        } else {
                            for req in filtered_requests.iter() {
                                tr { class: "hover:bg-gray-50",
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm font-medium text-gray-900", {req.title.clone()} }
                                        div { class: "text-sm text-gray-500", {req.department.clone()} }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {req.applicant.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span {
                                            class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {req.provider_color()}",
                                            {req.provider_name()}
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {req.cloud_platform.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {req.instance_type.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {req.instance_count.to_string()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span {
                                            class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {req.status.color_class()}",
                                            {req.status.display_name()}
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {req.created_at.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                        button {
                                            class: "text-blue-600 hover:text-blue-900 mr-3",
                                            title: "查看详情",
                                            Icon { icon: FaEye, width: 16, height: 16 }
                                        }
                                        button {
                                            class: "text-green-600 hover:text-green-900 mr-3",
                                            title: "编辑",
                                            Icon { icon: FaPen, width: 16, height: 16 }
                                        }
                                        button {
                                            class: "text-red-600 hover:text-red-900",
                                            title: "删除",
                                            Icon { icon: FaXmark, width: 16, height: 16 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 新建申请表单模态框
            if *show_new_form.read() {
                NewCloudServiceForm {
                    on_close: move |_| show_new_form.set(false),
                    on_submit: move |new_request: CloudServiceRequest| {
                        let mut reqs = requests.write();
                        let new_id = reqs.iter().map(|r| r.id).max().unwrap_or(0) + 1;
                        let mut new_request = new_request;
                        new_request.id = new_id;
                        reqs.push(new_request);
                        show_new_form.set(false);
                    },
                }
            }
        }
    }
}

/// 新建云服务申请表单
#[component]
fn NewCloudServiceForm(
    on_close: EventHandler<()>,
    on_submit: EventHandler<CloudServiceRequest>,
) -> Element {
    let mut title = use_signal(String::new);
    let mut applicant = use_signal(String::new);
    let mut department = use_signal(String::new);
    let mut provider_id = use_signal(|| None::<i32>);
    let mut cloud_platform = use_signal(String::new);
    let mut instance_type = use_signal(String::new);
    let mut instance_count = use_signal(|| 1i32);
    let mut duration = use_signal(|| "1年".to_string());
    let mut purpose = use_signal(String::new);
    let mut security_products = use_signal(SelectedSecurityProducts::new);
    let mut show_security_selector = use_signal(|| false);

    let providers = PROVIDERS_STATE.read();
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-3xl mx-4 max-h-[90vh] overflow-y-auto",
                div { class: "flex items-center justify-between p-6 border-b border-gray-200 sticky top-0 bg-white",
                    h3 { class: "text-lg font-bold text-gray-800", "申请新的云服务资源" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-6 space-y-4",
                    // 基本信息
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "申请标题 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "如：OA系统云服务器扩容",
                                value: "{title}",
                                oninput: move |e| title.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "申请人 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "申请人姓名",
                                value: "{applicant}",
                                oninput: move |e| applicant.set(e.value()),
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "申请部门 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "如：信息部",
                                value: "{department}",
                                oninput: move |e| department.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                onchange: move |e| provider_id.set(e.value().parse::<i32>().ok()),
                                option { value: "", "请选择服务商" }
                                for provider in providers.iter() {
                                    option { value: "{provider.id}", "{provider.short_name}" }
                                }
                            }
                        }
                    }

                    // 云资源配置
                    div { class: "border-t pt-4 mt-2",
                        h4 { class: "text-sm font-medium text-gray-700 mb-2", "云资源配置" }
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "云平台" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: "{cloud_platform}",
                                    onchange: move |e| cloud_platform.set(e.value()),
                                    option { value: "", "请选择云平台" }
                                    for platform in cloud_platforms.iter() {
                                        option { value: "{platform.platform_name}", "{platform.platform_name}" }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "实例类型" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "如：ecs.g6.xlarge",
                                    value: "{instance_type}",
                                    oninput: move |e| instance_type.set(e.value()),
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4 mt-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "实例数量" }
                                input {
                                    r#type: "number",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    min: "1",
                                    value: "{instance_count}",
                                    oninput: move |e| instance_count.set(e.value().parse::<i32>().unwrap_or(1)),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "使用时长" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: "{duration}",
                                    onchange: move |e| duration.set(e.value()),
                                    option { value: "1个月", "1个月" }
                                    option { value: "3个月", "3个月" }
                                    option { value: "6个月", "6个月" }
                                    option { value: "1年", "1年" }
                                    option { value: "2年", "2年" }
                                    option { value: "3年", "3年" }
                                }
                            }
                        }
                    }

                    // 用途说明
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "用途说明 *" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            rows: 3,
                            placeholder: "请详细说明申请用途...",
                            value: "{purpose}",
                            oninput: move |e| purpose.set(e.value()),
                        }
                    }

                    // 安全产品选择
                    div { class: "border-t pt-4 mt-2",
                        div { class: "flex items-center justify-between mb-2",
                            h4 { class: "text-sm font-medium text-gray-700", "安全产品选择" }
                            {
                                let is_expanded = *show_security_selector.read();
                                rsx! {
                                    button {
                                        class: "text-sm text-blue-600 hover:text-blue-800",
                                        onclick: move |_| show_security_selector.set(!is_expanded),
                                        if is_expanded {
                                            "收起选择器"
                                        } else {
                                            "展开选择器"
                                        }
                                    }
                                }
                            }
                        }
                        p { class: "text-xs text-gray-500 mb-3", "选择需要绑定的安全产品，每个分类只能选择一个" }

                        if *show_security_selector.read() {
                            crate::components::security_product::security_product_selector::SecurityProductSelector {
                                selected: security_products,
                                active_only: true,
                            }
                        } else {
                            // 显示已选择的安全产品摘要
                            div { class: "bg-gray-50 rounded-lg p-3",
                                if security_products.read().is_empty() {
                                    p { class: "text-sm text-gray-400", "尚未选择安全产品" }
                                } else {
                                    div { class: "space-y-1",
                                        for (category, product_id) in security_products.read().products.iter() {
                                            {
                                                let products = SECURITY_PRODUCTS_STATE.read();
                                                let product_name = products.iter()
                                                    .find(|p| p.id == *product_id)
                                                    .map(|p| p.name.clone())
                                                    .unwrap_or_else(|| format!("产品{}", product_id));
                                                rsx! {
                                                    div { class: "flex items-center text-sm",
                                                        Icon { icon: FaShieldHalved, width: 14, height: 14, class: "text-indigo-500 mr-2" }
                                                        span { class: "text-gray-600", "{category.display_name()}: " }
                                                        span { class: "font-medium text-gray-800", "{product_name}" }
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

                div { class: "flex justify-end gap-3 p-6 border-t sticky bottom-0 bg-white",
                    button {
                        class: "px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |_| {
                            let new_request = CloudServiceRequest {
                                id: 0,
                                title: title.read().clone(),
                                applicant: applicant.read().clone(),
                                department: department.read().clone(),
                                provider_id: *provider_id.read(),
                                cloud_platform: cloud_platform.read().clone(),
                                instance_type: instance_type.read().clone(),
                                instance_count: *instance_count.read(),
                                duration: duration.read().clone(),
                                purpose: purpose.read().clone(),
                                security_products: security_products.read().clone(),
                                status: CloudServiceStatus::Pending,
                                created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                            };
                            on_submit.call(new_request);
                        },
                        "提交申请"
                    }
                }
            }
        }
    }
}
