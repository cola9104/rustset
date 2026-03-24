use crate::app::{CLOUD_PLATFORMS_STATE, MACHINE_ROOMS_STATE, PROVIDERS_STATE};
use crate::components::common::{ErrorMessage, Modal, ModalFooter};
use crate::state::security_product::{
    SecurityProduct, SecurityProductCategory, SecurityProductStatus,
};
use dioxus::prelude::*;

/// 表单模式
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    /// 新增模式 - 可以修改所有字段
    New,
    /// 编辑模式 - 保持UI不变，只允许修改部分字段
    Edit,
}

impl FormMode {
    pub fn title(&self) -> &'static str {
        match self {
            FormMode::New => "添加产品",
            FormMode::Edit => "编辑产品",
        }
    }

    pub fn save_text(&self) -> &'static str {
        match self {
            FormMode::New => "添加",
            FormMode::Edit => "保存",
        }
    }
}

/// 表单数据结构 - 用于在组件间传递表单状态
#[derive(Clone, Debug)]
pub struct ProductFormData {
    pub name: String,
    pub category: SecurityProductCategory,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: String,
    pub license_type: String,
    pub license_expiry: String,
    pub management_ip: String,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: SecurityProductStatus,
    pub throughput: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: String,
}

impl Default for ProductFormData {
    fn default() -> Self {
        Self {
            name: String::new(),
            category: SecurityProductCategory::Firewall,
            vendor: String::new(),
            model: String::new(),
            version: String::new(),
            serial_number: String::new(),
            license_type: "永久".to_string(),
            license_expiry: String::new(),
            management_ip: String::new(),
            deployment_mode: String::new(),
            cloud_platform_id: None,
            machine_room_id: None,
            provider_id: None,
            status: SecurityProductStatus::Active,
            throughput: String::new(),
            contact_person: String::new(),
            contact_phone: String::new(),
            remarks: String::new(),
        }
    }
}

impl From<&SecurityProduct> for ProductFormData {
    fn from(product: &SecurityProduct) -> Self {
        Self {
            name: product.name.clone(),
            category: product.category,
            vendor: product.vendor.clone(),
            model: product.model.clone(),
            version: product.version.clone(),
            serial_number: product.serial_number.clone().unwrap_or_default(),
            license_type: product.license_type.clone(),
            license_expiry: product.license_expiry.clone().unwrap_or_default(),
            management_ip: product.management_ip.clone().unwrap_or_default(),
            deployment_mode: product.deployment_mode.clone(),
            cloud_platform_id: product.cloud_platform_id,
            machine_room_id: product.machine_room_id,
            provider_id: product.provider_id,
            status: product.status,
            throughput: product.throughput.clone().unwrap_or_default(),
            contact_person: product.contact_person.clone(),
            contact_phone: product.contact_phone.clone(),
            remarks: product.remarks.clone().unwrap_or_default(),
        }
    }
}

impl ProductFormData {
    /// 转换为 SecurityProduct
    pub fn to_product(&self, id: i32, created_at: String) -> SecurityProduct {
        SecurityProduct {
            id,
            name: self.name.clone(),
            category: self.category,
            vendor: self.vendor.clone(),
            model: self.model.clone(),
            version: self.version.clone(),
            serial_number: if self.serial_number.is_empty() {
                None
            } else {
                Some(self.serial_number.clone())
            },
            license_type: self.license_type.clone(),
            license_expiry: if self.license_expiry.is_empty() {
                None
            } else {
                Some(self.license_expiry.clone())
            },
            management_ip: if self.management_ip.is_empty() {
                None
            } else {
                Some(self.management_ip.clone())
            },
            deployment_mode: self.deployment_mode.clone(),
            cloud_platform_id: self.cloud_platform_id,
            machine_room_id: self.machine_room_id,
            provider_id: self.provider_id,
            status: self.status,
            features: vec![],
            throughput: if self.throughput.is_empty() {
                None
            } else {
                Some(self.throughput.clone())
            },
            contact_person: self.contact_person.clone(),
            contact_phone: self.contact_phone.clone(),
            remarks: if self.remarks.is_empty() {
                None
            } else {
                Some(self.remarks.clone())
            },
            created_at,
        }
    }

    /// 验证表单数据
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("产品名称不能为空".to_string());
        }
        if self.vendor.trim().is_empty() {
            return Err("厂商不能为空".to_string());
        }
        if self.model.trim().is_empty() {
            return Err("型号不能为空".to_string());
        }
        Ok(())
    }
}

/// 产品表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct ProductFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始产品数据
    #[props(default = None)]
    product: Option<SecurityProduct>,
    /// 保存事件
    on_save: EventHandler<SecurityProduct>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 产品表单组件 - 处理新增和编辑的通用组件
#[component]
pub fn ProductForm(props: ProductFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props
        .product
        .as_ref()
        .map(ProductFormData::from)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);

    // 获取关联数据
    let providers = PROVIDERS_STATE.read().clone();
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read().clone();
    let machine_rooms = MACHINE_ROOMS_STATE.read().clone();

    // 当前选中的显示值
    let category_display = form_data.read().category.display_name().to_string();
    let status_display = form_data.read().status.display_name().to_string();

    // 部署位置选择
    let provider_id_str = form_data
        .read()
        .provider_id
        .map(|id| id.to_string())
        .unwrap_or_default();
    let cloud_platform_id_str = form_data
        .read()
        .cloud_platform_id
        .map(|id| id.to_string())
        .unwrap_or_default();
    let machine_room_id_str = form_data
        .read()
        .machine_room_id
        .map(|id| id.to_string())
        .unwrap_or_default();

    rsx! {
        Modal {
            show: true,
            title: props.mode.title().to_string(),
            size: "2xl".to_string(),
            on_close: move |_| props.on_close.call(()),
            footer: rsx! {
                ModalFooter {
                    save_text: props.mode.save_text().to_string(),
                    cancel_text: "取消".to_string(),
                    save_disabled: false,
                    on_save: move |_| {
                        // 验证表单
                        let data = form_data.read();
                        if let Err(e) = data.validate() {
                            error_msg.set(e);
                            return;
                        }
                        error_msg.set(String::new());

                        // 构建产品对象
                        let product_id = props.product.as_ref().map(|p| p.id).unwrap_or(0);
                        let created_at = props.product.as_ref()
                            .map(|p| p.created_at.clone())
                            .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

                        let product = data.to_product(product_id, created_at);
                        props.on_save.call(product);
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
                div { class: "grid grid-cols-2 gap-4",
                    // 产品名称
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "产品名称 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().name}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.name = e.value();
                            },
                        }
                    }

                    // 产品分类
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "产品分类 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{category_display}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.category = SecurityProductCategory::from_str(&e.value());
                            },
                            for cat in SecurityProductCategory::all_categories() {
                                option { value: "{cat.display_name()}", "{cat.display_name()}" }
                            }
                        }
                    }
                }

                div { class: "grid grid-cols-2 gap-4",
                    // 厂商
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "厂商 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().vendor}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.vendor = e.value();
                            },
                        }
                    }

                    // 型号
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "型号 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().model}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.model = e.value();
                            },
                        }
                    }
                }

                div { class: "grid grid-cols-2 gap-4",
                    // 版本
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "版本" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().version}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.version = e.value();
                            },
                        }
                    }

                    // 序列号
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "序列号" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().serial_number}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.serial_number = e.value();
                            },
                        }
                    }
                }

                div { class: "grid grid-cols-2 gap-4",
                    // 管理IP
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "管理IP" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().management_ip}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.management_ip = e.value();
                            },
                        }
                    }

                    // 部署模式
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "部署模式" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().deployment_mode}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.deployment_mode = e.value();
                            },
                            option { value: "", "请选择" }
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
                    // 授权类型
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "授权类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().license_type}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.license_type = e.value();
                            },
                            option { value: "永久", "永久" }
                            option { value: "订阅", "订阅" }
                        }
                    }

                    // 授权到期
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "授权到期" }
                        input {
                            r#type: "date",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().license_expiry}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.license_expiry = e.value();
                            },
                        }
                    }
                }

                div { class: "grid grid-cols-2 gap-4",
                    // 吞吐量
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "吞吐量" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "如：40 Gbps",
                            value: "{form_data.read().throughput}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.throughput = e.value();
                            },
                        }
                    }

                    // 状态
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{status_display}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.status = match e.value().as_str() {
                                    "运行中" => SecurityProductStatus::Active,
                                    "已停用" => SecurityProductStatus::Inactive,
                                    "维护中" => SecurityProductStatus::Maintenance,
                                    "已下线" => SecurityProductStatus::Decommissioned,
                                    _ => SecurityProductStatus::Active,
                                };
                            },
                            for s in SecurityProductStatus::all_statuses() {
                                option { value: "{s.display_name()}", "{s.display_name()}" }
                            }
                        }
                    }
                }

                // 部署位置
                div { class: "border-t pt-4 mt-2",
                    h4 { class: "text-sm font-medium text-gray-700 mb-2", "部署位置" }
                    div { class: "grid grid-cols-3 gap-4",
                        // 服务商
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{provider_id_str}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.provider_id = e.value().parse().ok();
                                },
                                option { value: "", "请选择" }
                                for provider in providers.iter() {
                                    option { value: "{provider.id}", "{provider.short_name}" }
                                }
                            }
                        }

                        // 云平台
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "云平台" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{cloud_platform_id_str}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.cloud_platform_id = e.value().parse().ok();
                                },
                                option { value: "", "请选择" }
                                for platform in cloud_platforms.iter() {
                                    option { value: "{platform.id}", "{platform.platform_name}" }
                                }
                            }
                        }

                        // 机房
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{machine_room_id_str}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.machine_room_id = e.value().parse().ok();
                                },
                                option { value: "", "请选择" }
                                for room in machine_rooms.iter() {
                                    option { value: "{room.id}", "{room.room_name}" }
                                }
                            }
                        }
                    }
                }

                // 联系信息
                div { class: "border-t pt-4 mt-2",
                    h4 { class: "text-sm font-medium text-gray-700 mb-2", "联系信息" }
                    div { class: "grid grid-cols-2 gap-4",
                        // 负责人
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{form_data.read().contact_person}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.contact_person = e.value();
                                },
                            }
                        }

                        // 联系电话
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系电话" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{form_data.read().contact_phone}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.contact_phone = e.value();
                                },
                            }
                        }
                    }
                }

                // 备注
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        rows: 3,
                        value: "{form_data.read().remarks}",
                        oninput: move |e| {
                            let mut data = form_data.write();
                            data.remarks = e.value();
                        },
                    }
                }
            },
        }
    }
}
