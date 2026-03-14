use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaMagnifyingGlass, FaCloud, FaCheck, FaClock, FaXmark,
    FaCircleCheck, FaPen, FaEye
};
use crate::app::PROVIDERS_STATE;
use crate::components::security_product::security_product_selector::SelectedSecurityProducts;

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
    let mut show_edit_form = use_signal(|| None::<i32>);

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
                                tr {
                                    key: "{req.id}",
                                    class: "hover:bg-gray-50",
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
                                        {
                                            let edit_id = req.id;
                                            rsx! {
                                                button {
                                                    class: "text-green-600 hover:text-green-900 mr-3",
                                                    title: "编辑",
                                                    onclick: move |_| show_edit_form.set(Some(edit_id)),
                                                    Icon { icon: FaPen, width: 16, height: 16 }
                                                }
                                            }
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

            // 新建申请表单
            if *show_new_form.read() {
                super::service_form::CloudServiceForm {
                    mode: crate::components::common::FormMode::New,
                    request: None,
                    on_save: move |new_request: CloudServiceRequest| {
                        let mut reqs = requests.write();
                        let new_id = reqs.iter().map(|r| r.id).max().unwrap_or(0) + 1;
                        let mut new_request = new_request;
                        new_request.id = new_id;
                        reqs.push(new_request);
                        show_new_form.set(false);
                    },
                    on_close: move |_| show_new_form.set(false),
                }
            }

            // 编辑申请表单
            {
                let edit_id = *show_edit_form.read();
                edit_id.and_then(|id| {
                    let all_requests = requests.read().clone();
                    let editing_request = all_requests.iter().find(|r| r.id == id).cloned();
                    editing_request.map(|req| rsx! {
                        super::service_form::CloudServiceForm {
                            mode: crate::components::common::FormMode::Edit,
                            request: Some(req),
                            on_save: move |updated_request: CloudServiceRequest| {
                                let mut reqs = requests.write();
                                if let Some(r) = reqs.iter_mut().find(|r| r.id == id) {
                                    *r = updated_request;
                                }
                                show_edit_form.set(None);
                            },
                            on_close: move |_| show_edit_form.set(None),
                        }
                    })
                })
            }
        }
    }
}