use super::physical_server_request::{PhysicalServerRequest, PhysicalServerStatus};
use crate::app::{MACHINE_ROOMS_STATE, PROVIDERS_STATE, SECURITY_PRODUCTS_STATE};
use crate::components::common::{ErrorMessage, FormMode, Modal, ModalFooter};
use crate::components::security_product::security_product_selector::SecurityProductSelector;
use crate::components::security_product::security_product_selector::SelectedSecurityProducts;
use crate::services::{
    machine_room_api::fetch_machine_rooms, security_product_api::fetch_security_products,
    service_provider_api::fetch_service_providers,
};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaShieldHalved;
use dioxus_free_icons::Icon;

/// 物理机申请表单数据结构
#[derive(Clone, Debug, Default)]
pub struct PhysicalServerFormData {
    pub title: String,
    pub applicant: String,
    pub department: String,
    pub provider_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub server_type: String,
    pub cpu_cores: String,
    pub memory: String,
    pub storage: String,
    pub server_count: i32,
    pub purpose: String,
    pub security_products: SelectedSecurityProducts,
}

impl From<&PhysicalServerRequest> for PhysicalServerFormData {
    fn from(req: &PhysicalServerRequest) -> Self {
        Self {
            title: req.title.clone(),
            applicant: req.applicant.clone(),
            department: req.department.clone(),
            provider_id: req.provider_id,
            machine_room_id: req.machine_room_id,
            server_type: req.server_type.clone(),
            cpu_cores: req.cpu_cores.clone(),
            memory: req.memory.clone(),
            storage: req.storage.clone(),
            server_count: req.server_count,
            purpose: req.purpose.clone(),
            security_products: req.security_products.clone(),
        }
    }
}

impl PhysicalServerFormData {
    /// 转换为 PhysicalServerRequest
    pub fn to_request(
        &self,
        id: i32,
        status: PhysicalServerStatus,
        created_at: String,
    ) -> PhysicalServerRequest {
        PhysicalServerRequest {
            id,
            title: self.title.clone(),
            applicant: self.applicant.clone(),
            department: self.department.clone(),
            provider_id: self.provider_id,
            machine_room_id: self.machine_room_id,
            server_type: self.server_type.clone(),
            cpu_cores: self.cpu_cores.clone(),
            memory: self.memory.clone(),
            storage: self.storage.clone(),
            server_count: self.server_count,
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
        if self.server_count < 1 {
            return Err("服务器数量至少为1".to_string());
        }
        if self.purpose.trim().is_empty() {
            return Err("用途说明不能为空".to_string());
        }
        Ok(())
    }
}

/// 物理机申请表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct PhysicalServerFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始数据
    #[props(default = None)]
    request: Option<PhysicalServerRequest>,
    /// 保存事件
    on_save: EventHandler<PhysicalServerRequest>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 物理机申请表单组件
#[component]
pub fn PhysicalServerForm(props: PhysicalServerFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props
        .request
        .as_ref()
        .map(PhysicalServerFormData::from)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);
    let mut show_security_selector = use_signal(|| false);
    let security_products_signal = use_signal(|| form_data.read().security_products.clone());

    // 如果全局状态为空，从API获取数据
    use_effect(move || {
        spawn(async move {
            // 获取服务商
            if PROVIDERS_STATE.read().is_empty() {
                match fetch_service_providers().await {
                    Ok(providers) => {
                        *PROVIDERS_STATE.write() = providers;
                    }
                    Err(e) => {
                        tracing::error!("加载服务商数据失败: {}", e);
                    }
                }
            }

            // 获取机房
            if MACHINE_ROOMS_STATE.read().is_empty() {
                match fetch_machine_rooms().await {
                    Ok(rooms) => {
                        *MACHINE_ROOMS_STATE.write() = rooms;
                    }
                    Err(e) => {
                        tracing::error!("加载机房数据失败: {}", e);
                    }
                }
            }

            // 获取安全产品
            if SECURITY_PRODUCTS_STATE.read().is_empty() {
                match fetch_security_products().await {
                    Ok(products) => {
                        *SECURITY_PRODUCTS_STATE.write() = products;
                    }
                    Err(e) => {
                        tracing::error!("加载安全产品数据失败: {}", e);
                    }
                }
            }
        });
    });

    // 获取服务商和机房列表
    let providers = PROVIDERS_STATE.read().clone();
    let machine_rooms = MACHINE_ROOMS_STATE.read().clone();
    let selected_provider_id = form_data.read().provider_id;
    let filtered_machine_rooms = machine_rooms
        .iter()
        .filter(|room| {
            selected_provider_id
                .map(|provider_id| room.provider_id == provider_id)
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    // 编辑时保存原始ID
    let editing_id = props.request.as_ref().map(|r| r.id);

    rsx! {
        Modal {
            show: true,
            title: props.mode.title("物理机申请"),
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
                                    PhysicalServerStatus::Pending,
                                    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                )
                            }
                            FormMode::Edit => {
                                if let Some(id) = editing_id {
                                    data.to_request(id, PhysicalServerStatus::Pending, String::new())
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
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                placeholder: "如：核心数据库服务器",
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
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
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
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                placeholder: "如：信息部",
                                value: "{form_data.read().department}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.department = e.value();
                                },
                            }
                        }
                        // 服务器数量
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务器数量" }
                            input {
                                r#type: "number",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                min: "1",
                                value: "{form_data.read().server_count}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.server_count = e.value().parse::<i32>().unwrap_or(1);
                                },
                            }
                        }
                    }
                }

                // 服务器配置
                div { class: "border-b border-gray-200 pb-4",
                    h4 { class: "text-sm font-semibold text-gray-800 mb-3", "服务器配置" }
                    div { class: "grid grid-cols-2 gap-4",
                        // 服务商
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                value: "{form_data.read().provider_id.unwrap_or(-1)}",
                                onchange: move |e| {
                                    let val: i32 = e.value().parse().unwrap_or(-1);
                                    let mut data = form_data.write();
                                    data.provider_id = if val > 0 { Some(val) } else { None };
                                    if let Some(machine_room_id) = data.machine_room_id {
                                        let still_exists = MACHINE_ROOMS_STATE
                                            .read()
                                            .iter()
                                            .any(|room| {
                                                room.id == machine_room_id
                                                    && data
                                                        .provider_id
                                                        .map(|provider_id| room.provider_id == provider_id)
                                                        .unwrap_or(true)
                                            });
                                        if !still_exists {
                                            data.machine_room_id = None;
                                        }
                                    }
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
                        // 机房
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                value: "{form_data.read().machine_room_id.unwrap_or(-1)}",
                                onchange: move |e| {
                                    let val: i32 = e.value().parse().unwrap_or(-1);
                                    let mut data = form_data.write();
                                    data.machine_room_id = if val > 0 { Some(val) } else { None };
                                },
                                option { value: "-1", "请选择机房" }
                                for room in filtered_machine_rooms.iter() {
                                    option {
                                        value: "{room.id}",
                                        selected: form_data.read().machine_room_id == Some(room.id),
                                        "{room.room_name}"
                                    }
                                }
                            }
                        }
                        // 服务器类型
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务器类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                value: "{form_data.read().server_type}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.server_type = e.value();
                                },
                                option { value: "", "请选择服务器类型" }
                                option { value: "机架式服务器", "机架式服务器" }
                                option { value: "刀片服务器", "刀片服务器" }
                                option { value: "塔式服务器", "塔式服务器" }
                                option { value: "高密度服务器", "高密度服务器" }
                            }
                        }
                        // CPU核数
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "CPU核数" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                value: "{form_data.read().cpu_cores}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.cpu_cores = e.value();
                                },
                                option { value: "", "请选择CPU核数" }
                                option { value: "8核", "8核" }
                                option { value: "16核", "16核" }
                                option { value: "24核", "24核" }
                                option { value: "32核", "32核" }
                                option { value: "48核", "48核" }
                                option { value: "64核", "64核" }
                            }
                        }
                        // 内存
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "内存" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                value: "{form_data.read().memory}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.memory = e.value();
                                },
                                option { value: "", "请选择内存大小" }
                                option { value: "32GB", "32GB" }
                                option { value: "64GB", "64GB" }
                                option { value: "96GB", "96GB" }
                                option { value: "128GB", "128GB" }
                                option { value: "192GB", "192GB" }
                                option { value: "256GB", "256GB" }
                                option { value: "512GB", "512GB" }
                            }
                        }
                        // 存储
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "存储" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                                value: "{form_data.read().storage}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.storage = e.value();
                                },
                                option { value: "", "请选择存储配置" }
                                option { value: "512GB SSD", "512GB SSD" }
                                option { value: "1TB SSD", "1TB SSD" }
                                option { value: "2TB SSD", "2TB SSD" }
                                option { value: "4TB SSD", "4TB SSD" }
                                option { value: "4TB HDD", "4TB HDD" }
                                option { value: "8TB HDD", "8TB HDD" }
                                option { value: "16TB HDD", "16TB HDD" }
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
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent",
                        rows: 3,
                        placeholder: "请详细说明服务器用途...",
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
