use dioxus::prelude::*;
use crate::state::service_provider::ServiceProviderConfig;
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
            FormMode::New => "添加服务商",
            FormMode::Edit => "编辑服务商",
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
#[derive(Clone, Debug)]
pub struct ProviderFormData {
    pub provider_name: String,
    pub provider_code: String,
    pub short_name: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub headquarters: String,
    pub service_area: String,
    pub business_license: String,
    pub remarks: String,
    pub status: String,
}

impl Default for ProviderFormData {
    fn default() -> Self {
        Self {
            provider_name: String::new(),
            provider_code: String::new(),
            short_name: String::new(),
            contact_person: String::new(),
            contact_phone: String::new(),
            contact_email: String::new(),
            headquarters: String::new(),
            service_area: "全国".to_string(),
            business_license: String::new(),
            remarks: String::new(),
            status: "active".to_string(),
        }
    }
}

impl From<&ServiceProviderConfig> for ProviderFormData {
    fn from(provider: &ServiceProviderConfig) -> Self {
        Self {
            provider_name: provider.provider_name.clone(),
            provider_code: provider.provider_code.clone(),
            short_name: provider.short_name.clone(),
            contact_person: provider.contact_person.clone(),
            contact_phone: provider.contact_phone.clone(),
            contact_email: provider.contact_email.clone(),
            headquarters: provider.headquarters.clone(),
            service_area: provider.service_area.clone(),
            business_license: provider.business_license.clone(),
            remarks: provider.remarks.clone().unwrap_or_default(),
            status: provider.status.clone(),
        }
    }
}

impl ProviderFormData {
    /// 转换为 ServiceProviderConfig
    pub fn to_config(&self, id: i32, created_at: String, updated_at: Option<String>) -> ServiceProviderConfig {
        ServiceProviderConfig {
            id,
            provider_name: self.provider_name.clone(),
            provider_code: self.provider_code.clone(),
            short_name: self.short_name.clone(),
            logo_url: None,
            contact_person: self.contact_person.clone(),
            contact_phone: self.contact_phone.clone(),
            contact_email: self.contact_email.clone(),
            headquarters: self.headquarters.clone(),
            service_area: self.service_area.clone(),
            business_license: self.business_license.clone(),
            remarks: if self.remarks.is_empty() { None } else { Some(self.remarks.clone()) },
            status: self.status.clone(),
            created_at,
            updated_at,
        }
    }

    /// 验证表单数据
    pub fn validate(&self) -> Result<(), String> {
        if self.provider_name.trim().is_empty() {
            return Err("服务商名称不能为空".to_string());
        }
        if self.short_name.trim().is_empty() {
            return Err("简称不能为空".to_string());
        }
        if self.contact_person.trim().is_empty() {
            return Err("负责人不能为空".to_string());
        }
        if self.contact_phone.trim().is_empty() {
            return Err("联系电话不能为空".to_string());
        }
        Ok(())
    }
}

/// 服务商表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct ProviderFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始服务商数据
    #[props(default = None)]
    provider: Option<ServiceProviderConfig>,
    /// 保存事件
    on_save: EventHandler<ServiceProviderConfig>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 服务商表单组件 - 处理新增和编辑的通用组件
#[component]
pub fn ProviderForm(props: ProviderFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props.provider.as_ref()
        .map(ProviderFormData::from)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);

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
                        if let Err(e) = data.validate() {
                            error_msg.set(e);
                            return;
                        }
                        error_msg.set(String::new());

                        // 根据模式构建配置对象
                        let config = match props.mode {
                            FormMode::New => {
                                // 新建时ID由后端生成，前端传0
                                data.to_config(
                                    0,
                                    chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                    None,
                                )
                            }
                            FormMode::Edit => {
                                if let Some(original) = props.provider.as_ref() {
                                    data.to_config(
                                        original.id,
                                        original.created_at.clone(),
                                        Some(chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string()),
                                    )
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

                // 表单字段
                div { class: "grid grid-cols-2 gap-4",
                    // 服务商名称
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "服务商名称 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "中国电信",
                            value: "{form_data.read().provider_name}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.provider_name = e.value();
                            },
                        }
                    }

                    // 服务商编码
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "服务商编码 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "CHINA_TELECOM",
                            value: "{form_data.read().provider_code}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.provider_code = e.value();
                            },
                        }
                    }

                    // 简称
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "简称 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "电信",
                            value: "{form_data.read().short_name}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.short_name = e.value();
                            },
                        }
                    }

                    // 负责人
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "负责人 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "张经理",
                            value: "{form_data.read().contact_person}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.contact_person = e.value();
                            },
                        }
                    }

                    // 联系电话
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "联系电话 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "10000",
                            value: "{form_data.read().contact_phone}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.contact_phone = e.value();
                            },
                        }
                    }

                    // 联系邮箱
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "联系邮箱" }
                        input {
                            r#type: "email",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "telecom@example.com",
                            value: "{form_data.read().contact_email}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.contact_email = e.value();
                            },
                        }
                    }

                    // 总部地址
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "总部地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "北京市西城区金融大街35号",
                            value: "{form_data.read().headquarters}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.headquarters = e.value();
                            },
                        }
                    }

                    // 服务区域
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "服务区域" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "全国",
                            value: "{form_data.read().service_area}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.service_area = e.value();
                            },
                        }
                    }

                    // 营业执照号
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "营业执照号" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "91110000XXXXXXXX",
                            value: "{form_data.read().business_license}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.business_license = e.value();
                            },
                        }
                    }

                    // 备注
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            rows: 2,
                            placeholder: "服务商备注信息",
                            value: "{form_data.read().remarks}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.remarks = e.value();
                            },
                        }
                    }

                    // 状态
                    div { class: "col-span-2 sm:col-span-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().status}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.status = e.value();
                            },
                            option { value: "active", "活跃" }
                            option { value: "inactive", "停用" }
                        }
                    }
                }
            },
        }
    }
}
