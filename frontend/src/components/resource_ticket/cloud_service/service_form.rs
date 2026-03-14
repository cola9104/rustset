use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaShieldHalved;
use crate::components::common::{Modal, ModalFooter, ErrorMessage, FormMode};
use crate::components::security_product::security_product_selector::SecurityProductSelector;
use crate::app::{PROVIDERS_STATE, CLOUD_PLATFORMS_STATE, SECURITY_PRODUCTS_STATE};
use crate::components::security_product::security_product_selector::SelectedSecurityProducts;
use super::cloud_service_request::{CloudServiceRequest, CloudServiceStatus};

/// 云服务申请表单数据结构
#[derive(Clone, Debug, Default)]
pub struct CloudServiceFormData {
    pub title: String,
    pub applicant: String,
    pub department: String,
    pub provider_id: Option<i32>,
    pub cloud_platform: String,
    pub instance_type: String,
    pub instance_count: i32,
    pub duration: String,
    pub purpose: String,
    pub security_products: SelectedSecurityProducts,
}

impl From<&CloudServiceRequest> for CloudServiceFormData {
    fn from(req: &CloudServiceRequest) -> Self {
        Self {
            title: req.title.clone(),
            applicant: req.applicant.clone(),
            department: req.department.clone(),
            provider_id: req.provider_id,
            cloud_platform: req.cloud_platform.clone(),
            instance_type: req.instance_type.clone(),
            instance_count: req.instance_count,
            duration: req.duration.clone(),
            purpose: req.purpose.clone(),
            security_products: req.security_products.clone(),
        }
    }
}

impl CloudServiceFormData {
    /// 转换为 CloudServiceRequest
    pub fn to_request(&self, id: i32, status: CloudServiceStatus, created_at: String) -> CloudServiceRequest {
        CloudServiceRequest {
            id,
            title: self.title.clone(),
            applicant: self.applicant.clone(),
            department: self.department.clone(),
            provider_id: self.provider_id,
            cloud_platform: self.cloud_platform.clone(),
            instance_type: self.instance_type.clone(),
            instance_count: self.instance_count,
            duration: self.duration.clone(),
            purpose: self.purpose.clone(),
            security_products: self.security_products.clone(),
            status,
            created_at,
        }
    }

    /// 验证表单数据
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("申请标题不能为空".to_string());
        }
        if self.applicant.trim().is_empty() {
            return Err("申请人不能为空".to_string());
        }
        if self.department.trim().is_empty() {
            return Err("申请部门不能为空".to_string());
        }
        if self.cloud_platform.trim().is_empty() {
            return Err("请选择云平台".to_string());
        }
        if self.instance_count < 1 {
            return Err("实例数量至少为1".to_string());
        }
        if self.purpose.trim().is_empty() {
            return Err("用途说明不能为空".to_string());
        }
        Ok(())
    }
}

/// 云服务申请表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct CloudServiceFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始数据
    #[props(default = None)]
    request: Option<CloudServiceRequest>,
    /// 保存事件
    on_save: EventHandler<CloudServiceRequest>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 云服务申请表单组件
#[component]
pub fn CloudServiceForm(props: CloudServiceFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props.request.as_ref()
        .map(CloudServiceFormData::from)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);
    let mut show_security_selector = use_signal(|| false);
    let security_products_signal = use_signal(|| form_data.read().security_products.clone());

    // 获取服务商和云平台列表
    let providers = PROVIDERS_STATE.read().clone();
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read().clone();

    // 编辑时保存原始ID
    let editing_id = props.request.as_ref().map(|r| r.id);

    rsx! {
        Modal {
            show: true,
            title: props.mode.title("云服务申请"),
            size: "2xl".to_string(),
            on_close: move |_| props.on_close.call(()),
            footer: rsx! {
                ModalFooter {
                    save_text: props.mode.save_text().to_string(),
                    cancel_text: "取消".to_string(),
                    save_disabled: false,
                    on_save: move |_| {
                        // 验证表单
                        let data = form_data.read().clone();
                        if let Err(e) = data.validate() {
                            error_msg.set(e);
                            return;
                        }
                        error_msg.set(String::new());

                        // 更新安全产品数据
                        let security_products = security_products_signal.read().clone();
                        let mut final_data = form_data.write();
                        final_data.security_products = security_products;
                        drop(final_data);

                        // 根据模式构建请求对象
                        let request = match props.mode {
                            FormMode::New => {
                                data.to_request(
                                    0,
                                    CloudServiceStatus::Pending,
                                    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                )
                            }
                            FormMode::Edit => {
                                if let Some(id) = editing_id {
                                    data.to_request(id, CloudServiceStatus::Pending, String::new())
                                } else {
                                    return;
                                }
                            }
                        };

                        props.on_save.call(request);
                    },
                    on_cancel: move |_| props.on_close.call(()),
                }
            },

            div { class: "space-y-4",
                // 错误提示
                if !error_msg.read().is_empty() {
                    ErrorMessage { message: error_msg.read().clone() }
                }

                // 基本信息
                div { class: "border-b border-gray-200 pb-4",
                    h4 { class: "text-sm font-semibold text-gray-800 mb-3", "基本信息" }
                    div { class: "grid grid-cols-2 gap-4",
                        // 申请标题
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "申请标题 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "如：OA系统云服务器扩容",
                                value: "{form_data.read().title}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.title = e.value();
                                },
                            }
                        }
                        // 申请人
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "申请人 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "申请人姓名",
                                value: "{form_data.read().applicant}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.applicant = e.value();
                                },
                            }
                        }
                        // 申请部门
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "申请部门 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "如：信息部",
                                value: "{form_data.read().department}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.department = e.value();
                                },
                            }
                        }
                        // 服务商
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                value: "{form_data.read().provider_id.unwrap_or(-1)}",
                                onchange: move |e| {
                                    let val: i32 = e.value().parse().unwrap_or(-1);
                                    let mut data = form_data.write();
                                    data.provider_id = if val > 0 { Some(val) } else { None };
                                },
                                option { value: "-1", "请选择服务商" }
                                for provider in providers.iter() {
                                    option {
                                        value: "{provider.id}",
                                        selected: form_data.read().provider_id == Some(provider.id),
                                        "{provider.short_name}"
                                    }
                                }
                            }
                        }
                    }
                }

                // 云资源配置
                div { class: "border-b border-gray-200 pb-4",
                    h4 { class: "text-sm font-semibold text-gray-800 mb-3", "云资源配置" }
                    div { class: "grid grid-cols-2 gap-4",
                        // 云平台
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "云平台 "
                                span { class: "text-red-500", "*" }
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                value: "{form_data.read().cloud_platform}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.cloud_platform = e.value();
                                },
                                option { value: "", "请选择云平台" }
                                for platform in cloud_platforms.iter() {
                                    option {
                                        value: "{platform.platform_name}",
                                        selected: form_data.read().cloud_platform == platform.platform_name,
                                        "{platform.platform_name}"
                                    }
                                }
                            }
                        }
                        // 实例类型
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "实例类型" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "如：ecs.g6.xlarge",
                                value: "{form_data.read().instance_type}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.instance_type = e.value();
                                },
                            }
                        }
                        // 实例数量
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "实例数量" }
                            input {
                                r#type: "number",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                min: "1",
                                value: "{form_data.read().instance_count}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.instance_count = e.value().parse::<i32>().unwrap_or(1);
                                },
                            }
                        }
                        // 使用时长
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "使用时长" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                value: "{form_data.read().duration}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.duration = e.value();
                                },
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
                    label { class: "block text-sm font-medium text-gray-700 mb-1",
                        "用途说明 "
                        span { class: "text-red-500", "*" }
                    }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        rows: 3,
                        placeholder: "请详细说明申请用途...",
                        value: "{form_data.read().purpose}",
                        oninput: move |e| {
                            let mut data = form_data.write();
                            data.purpose = e.value();
                        },
                    }
                }

                // 安全产品选择
                div { class: "border-t border-gray-200 pt-4",
                    div { class: "flex items-center justify-between mb-2",
                        h4 { class: "text-sm font-semibold text-gray-800", "安全产品选择" }
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
                        SecurityProductSelector {
                            selected: security_products_signal,
                            active_only: true,
                        }
                    } else {
                        // 显示已选择的安全产品摘要
                        div { class: "bg-gray-50 rounded-lg p-3",
                            if security_products_signal.read().is_empty() {
                                p { class: "text-sm text-gray-400", "尚未选择安全产品" }
                            } else {
                                div { class: "space-y-1",
                                    for (category, product_id) in security_products_signal.read().products.iter() {
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
            },
        }
    }
}
