use super::network_policy_request::{
    AccessDirection, NetworkPolicyRequest, NetworkPolicyStatus, PolicyProtocol,
};
use crate::app::AUTH_STATE;
use crate::components::common::{ErrorMessage, FormMode, Modal, ModalFooter};
use crate::services::ip_zone_api::fetch_ip_zones;
use dioxus::prelude::*;

/// 网络策略申请表单数据结构
#[derive(Clone, Debug)]
pub struct NetworkPolicyFormData {
    pub title: String,
    pub organization: String,
    pub applicant: String,
    pub department: String,
    pub source_zone: String,
    pub destination_zone: String,
    pub direction: AccessDirection,
    pub protocol: PolicyProtocol,
    pub port_range: String,
    pub description: String,
    pub valid_until: String,
}

impl Default for NetworkPolicyFormData {
    fn default() -> Self {
        Self {
            title: String::new(),
            organization: String::new(),
            applicant: String::new(),
            department: String::new(),
            source_zone: String::new(),
            destination_zone: String::new(),
            direction: AccessDirection::Outbound,
            protocol: PolicyProtocol::Tcp,
            port_range: String::new(),
            description: String::new(),
            valid_until: chrono::Local::now().format("%Y-%m-%d").to_string(),
        }
    }
}

impl From<&NetworkPolicyRequest> for NetworkPolicyFormData {
    fn from(req: &NetworkPolicyRequest) -> Self {
        Self {
            title: req.title.clone(),
            organization: req.organization.clone(),
            applicant: req.applicant.clone(),
            department: req.department.clone(),
            source_zone: req.source_zone.clone(),
            destination_zone: req.destination_zone.clone(),
            direction: req.direction.clone(),
            protocol: req.protocol.clone(),
            port_range: req.port_range.clone(),
            description: req.description.clone(),
            valid_until: req.valid_until.clone(),
        }
    }
}

impl NetworkPolicyFormData {
    /// 转换为 NetworkPolicyRequest
    pub fn to_request(
        &self,
        id: i32,
        status: NetworkPolicyStatus,
        created_at: String,
    ) -> NetworkPolicyRequest {
        NetworkPolicyRequest {
            id,
            title: self.title.clone(),
            organization: self.organization.clone(),
            applicant: self.applicant.clone(),
            department: self.department.clone(),
            source_zone: self.source_zone.clone(),
            destination_zone: self.destination_zone.clone(),
            direction: self.direction.clone(),
            protocol: self.protocol.clone(),
            port_range: self.port_range.clone(),
            description: self.description.clone(),
            valid_until: self.valid_until.clone(),
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
        if self.source_zone.trim().is_empty() {
            return Err("请选择源网络区域".to_string());
        }
        if self.destination_zone.trim().is_empty() {
            return Err("请选择目标网络区域".to_string());
        }
        if self.port_range.trim().is_empty() {
            return Err("端口范围不能为空".to_string());
        }
        if self.description.trim().is_empty() {
            return Err("描述不能为空".to_string());
        }
        Ok(())
    }
}

/// 网络策略申请表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct NetworkPolicyFormProps {
    /// 表单模式
    pub mode: FormMode,
    /// 编辑时的原始数据
    #[props(default = None)]
    pub request: Option<NetworkPolicyRequest>,
    /// 保存事件
    pub on_save: EventHandler<NetworkPolicyRequest>,
    /// 关闭事件
    pub on_close: EventHandler<()>,
}

/// 网络策略申请表单组件
#[component]
pub fn NetworkPolicyForm(props: NetworkPolicyFormProps) -> Element {
    let current_user = AUTH_STATE.read().clone();

    // 初始化表单数据
    let initial_data = props
        .request
        .as_ref()
        .map(NetworkPolicyFormData::from)
        .unwrap_or_else(|| NetworkPolicyFormData {
            organization: current_user
                .as_ref()
                .map(|user| user.organization_name.clone())
                .unwrap_or_default(),
            applicant: current_user
                .as_ref()
                .map(|user| user.display_name.clone())
                .unwrap_or_default(),
            department: current_user
                .as_ref()
                .map(|user| user.department_name.clone())
                .unwrap_or_default(),
            ..Default::default()
        });

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);
    let network_zones = use_signal(Vec::<String>::new);

    {
        let mut network_zones = network_zones;
        use_effect(move || {
            spawn(async move {
                match fetch_ip_zones().await {
                    Ok(zones) => {
                        let mut zone_names = zones
                            .into_iter()
                            .map(|zone| zone.name)
                            .filter(|name| !name.trim().is_empty())
                            .collect::<Vec<_>>();
                        zone_names.sort();
                        zone_names.dedup();
                        network_zones.set(zone_names);
                    }
                    Err(e) => {
                        tracing::error!("加载网络区域失败: {}", e);
                        network_zones.set(Vec::new());
                    }
                }
            });
        });
    }

    // 编辑时保存原始ID
    let editing_id = props.request.as_ref().map(|r| r.id);

    rsx! {
        Modal {
            show: true,
            title: props.mode.title("网络策略申请"),
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

                        // 根据模式构建请求对象
                        let request = match props.mode {
                            FormMode::New => {
                                data.to_request(
                                    0,
                                    NetworkPolicyStatus::Pending,
                                    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                )
                            }
                            FormMode::Edit => {
                                if let Some(id) = editing_id {
                                    data.to_request(id, NetworkPolicyStatus::Pending, String::new())
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
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                placeholder: "如：OA系统访问互联网策略",
                                value: "{form_data.read().title}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.title = e.value();
                                },
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "申请单位" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-200 bg-gray-50 text-gray-500 rounded-lg cursor-not-allowed",
                                value: "{form_data.read().organization}",
                                readonly: true,
                                disabled: true,
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
                                class: "w-full px-3 py-2 border border-gray-200 bg-gray-50 text-gray-500 rounded-lg cursor-not-allowed",
                                value: "{form_data.read().applicant}",
                                readonly: true,
                                disabled: true,
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
                                class: "w-full px-3 py-2 border border-gray-200 bg-gray-50 text-gray-500 rounded-lg cursor-not-allowed",
                                value: "{form_data.read().department}",
                                readonly: true,
                                disabled: true,
                            }
                        }
                        // 有效期
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "有效期至" }
                            input {
                                r#type: "date",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: "{form_data.read().valid_until}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.valid_until = e.value();
                                },
                            }
                        }
                    }
                }

                // 网络配置
                div { class: "border-b border-gray-200 pb-4",
                    h4 { class: "text-sm font-semibold text-gray-800 mb-3", "网络配置" }
                    div { class: "grid grid-cols-2 gap-4",
                        // 源网络区域
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "源网络区域 "
                                span { class: "text-red-500", "*" }
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: "{form_data.read().source_zone}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.source_zone = e.value();
                                },
                                option { value: "", "请选择源网络区域" }
                                for zone_name in network_zones.read().iter() {
                                    option {
                                        value: "{zone_name}",
                                        selected: form_data.read().source_zone == *zone_name,
                                        "{zone_name}"
                                    }
                                }
                            }
                        }
                        // 目标网络区域
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "目标网络区域 "
                                span { class: "text-red-500", "*" }
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: "{form_data.read().destination_zone}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.destination_zone = e.value();
                                },
                                option { value: "", "请选择目标网络区域" }
                                for zone_name in network_zones.read().iter() {
                                    option {
                                        value: "{zone_name}",
                                        selected: form_data.read().destination_zone == *zone_name,
                                        "{zone_name}"
                                    }
                                }
                            }
                        }
                        // 访问方向
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "访问方向" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: match form_data.read().direction {
                                    AccessDirection::Inbound => "inbound",
                                    AccessDirection::Outbound => "outbound",
                                    AccessDirection::Bidirectional => "bidirectional",
                                },
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.direction = match e.value().as_str() {
                                        "inbound" => AccessDirection::Inbound,
                                        "bidirectional" => AccessDirection::Bidirectional,
                                        _ => AccessDirection::Outbound,
                                    };
                                },
                                option { value: "outbound", "出站" }
                                option { value: "inbound", "入站" }
                                option { value: "bidirectional", "双向" }
                            }
                        }
                        // 协议类型
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "协议类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: match form_data.read().protocol {
                                    PolicyProtocol::Tcp => "tcp",
                                    PolicyProtocol::Udp => "udp",
                                    PolicyProtocol::Icmp => "icmp",
                                    PolicyProtocol::Any => "any",
                                },
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.protocol = match e.value().as_str() {
                                        "udp" => PolicyProtocol::Udp,
                                        "icmp" => PolicyProtocol::Icmp,
                                        "any" => PolicyProtocol::Any,
                                        _ => PolicyProtocol::Tcp,
                                    };
                                },
                                option { value: "tcp", "TCP" }
                                option { value: "udp", "UDP" }
                                option { value: "icmp", "ICMP" }
                                option { value: "any", "ANY" }
                            }
                        }
                        // 端口范围
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "端口范围 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                placeholder: "如：443, 80 或 8000-9000",
                                value: "{form_data.read().port_range}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.port_range = e.value();
                                },
                            }
                        }
                    }
                }

                // 描述
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1",
                        "描述 "
                        span { class: "text-red-500", "*" }
                    }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                        rows: 3,
                        placeholder: "请详细说明策略用途...",
                        value: "{form_data.read().description}",
                        oninput: move |e| {
                            let mut data = form_data.write();
                            data.description = e.value();
                        },
                    }
                }
            },
        }
    }
}
