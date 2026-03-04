use dioxus::prelude::*;
use crate::state::cloud_platform::CloudPlatformConfig;
use crate::app::{PROVIDERS_STATE, CLOUD_PLATFORMS_STATE};
use crate::components::common::{Modal, ModalFooter, ErrorMessage};

/// 表单模式
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    /// 新增模式
    New,
    /// 编辑模式
    Edit,
}

impl FormMode {
    pub fn title(&self) -> &'static str {
        match self {
            FormMode::New => "添加云平台",
            FormMode::Edit => "编辑云平台",
        }
    }

    pub fn save_text(&self) -> &'static str {
        match self {
                FormMode::New => "添加",
                FormMode::Edit => "保存",
        }
    }
}

/// 表单数据结构
#[derive(Clone, Debug, Default)]
pub struct PlatformFormData {
    pub platform_name: String,
    pub provider_id: i32,
    pub cloud_type: String,
    pub foundation: String,
    pub region_id: String,
    pub machine_room_id: i32,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: String,
    pub status: String,
}

impl PlatformFormData {
    pub fn from_config(config: &CloudPlatformConfig) -> Self {
        Self {
            platform_name: config.platform_name.clone(),
            provider_id: config.provider_id,
            cloud_type: config.cloud_type.clone(),
            foundation: config.foundation.clone(),
            region_id: config.region_id.clone(),
            machine_room_id: config.machine_room_id,
            access_key_id: config.access_key_id.clone(),
            access_key_secret: config.access_key_secret.clone(),
            remarks: config.remarks.clone().unwrap_or_default(),
            status: config.status.clone(),
        }
    }
}

/// 平台表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct PlatformFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始配置
    config: Option<CloudPlatformConfig>,
    /// 保存事件
    on_save: EventHandler<CloudPlatformConfig>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 平台表单组件
#[component]
pub fn PlatformForm(props: PlatformFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props.config.as_ref()
        .map(PlatformFormData::from_config)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);

    // 获取关联数据
    let providers = PROVIDERS_STATE.read().clone();

    rsx! {
        Modal {
            show: true,
            title: props.mode.title().to_string(),
            size: "xl".to_string(),
            on_close: move |_| props.on_close.call(()),
            footer: rsx! {
                ModalFooter {
                    save_text: props.mode.save_text().to_string(),
                    cancel_text: "取消".to_string(),
                    save_disabled: false,
                    on_save: move |_| {
                        // 验证表单
                        let data = form_data.read();
                        if data.platform_name.trim().is_empty() {
                            error_msg.set("平台名称不能为空".to_string());
                            return;
                        }
                        if data.access_key_id.trim().is_empty() {
                            error_msg.set("AccessKey ID 不能为空".to_string());
                            return;
                        }
                        if data.access_key_secret.trim().is_empty() {
                            error_msg.set("AccessKey Secret 不能为空".to_string());
                            return;
                        }
                        error_msg.set(String::new());

                        // 根据模式构建配置对象
                        let config = match props.mode {
                            FormMode::New => {
                                let max_id = CLOUD_PLATFORMS_STATE.read()
                                    .iter()
                                    .map(|c| c.id)
                                    .max()
                                    .unwrap_or(0);
                                CloudPlatformConfig {
                                    id: max_id + 1,
                                    platform_name: data.platform_name.clone(),
                                    provider_id: data.provider_id,
                                    cloud_type: data.cloud_type.clone(),
                                    foundation: data.foundation.clone(),
                                    region_id: data.region_id.clone(),
                                    machine_room_id: data.machine_room_id,
                                    access_key_id: data.access_key_id.clone(),
                                    access_key_secret: data.access_key_secret.clone(),
                                    remarks: if data.remarks.is_empty() { None } else { Some(data.remarks.clone()) },
                                    status: data.status.clone(),
                                    last_test_time: None,
                                    last_test_result: None,
                                    created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                    updated_at: None,
                                }
                            }
                            FormMode::Edit => {
                                if let Some(ref original) = props.config.as_ref() {
                                    CloudPlatformConfig {
                                        id: original.id,
                                        platform_name: data.platform_name.clone(),
                                        provider_id: data.provider_id,
                                        cloud_type: data.cloud_type.clone(),
                                        foundation: data.foundation.clone(),
                                        region_id: data.region_id.clone(),
                                        machine_room_id: data.machine_room_id,
                                        access_key_id: data.access_key_id.clone(),
                                        access_key_secret: data.access_key_secret.clone(),
                                        remarks: if data.remarks.is_empty() { None } else { Some(data.remarks.clone()) },
                                        status: data.status.clone(),
                                        last_test_time: original.last_test_time.clone(),
                                        last_test_result: original.last_test_result.clone(),
                                        created_at: original.created_at.clone(),
                                        updated_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string()),
                                    }
                                } else {
                                    return;
                                }
                            }
                        };

                        props.on_save.call(config);
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
                    // 平台名称
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "平台名称 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: 电信-公有云",
                            value: "{form_data.read().platform_name}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.platform_name = e.value();
                            },
                        }
                    }

                    // 服务商
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "服务商 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().provider_id}",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<i32>() {
                                    let mut data = form_data.write();
                                    data.provider_id = id;
                                }
                            },
                            for provider in providers.iter() {
                                option { value: "{provider.id}", "{provider.short_name}" }
                            }
                        }
                    }

                    // 云类型
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "云类型 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().cloud_type}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.cloud_type = e.value();
                            },
                            option { value: "公有云", "公有云" }
                            option { value: "政务云", "政务云" }
                        }
                    }

                    // 云底座
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "云底座 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().foundation}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.foundation = e.value();
                            },
                            option { value: "阿里云", "阿里云" }
                            option { value: "华为云", "华为云" }
                            option { value: "腾讯云", "腾讯云" }
                        }
                    }

                    // Region ID
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Region ID" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: cn-hangzhou",
                            value: "{form_data.read().region_id}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.region_id = e.value();
                            },
                        }
                    }

                    // 部署机房ID
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "部署机房ID "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "number",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().machine_room_id}",
                            oninput: move |e| {
                                if let Ok(id) = e.value().parse::<i32>() {
                                    let mut data = form_data.write();
                                    data.machine_room_id = id;
                                }
                            },
                        }
                    }

                    // AccessKey ID
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "AccessKey ID "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                            placeholder: "LTAI5t...",
                            value: "{form_data.read().access_key_id}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.access_key_id = e.value();
                            },
                        }
                    }

                    // AccessKey Secret
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "AccessKey Secret "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "password",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                            placeholder: "••••••••••••••••",
                            value: "{form_data.read().access_key_secret}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.access_key_secret = e.value();
                            },
                        }
                    }

                    // 备注
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            rows: 3,
                            placeholder: "填写备注信息...",
                            value: "{form_data.read().remarks}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.remarks = e.value();
                            },
                        }
                    }

                    // 状态
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().status}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.status = e.value();
                            },
                            option { value: "active", "运行中" }
                            option { value: "inactive", "已停用" }
                        }
                    }
                }
            }
        }
    }
}
