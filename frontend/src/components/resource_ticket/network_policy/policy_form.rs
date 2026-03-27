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
    pub applicant_account: String,
    pub department: String,
    pub source_zone: String,
    pub source_address: String,
    pub source_port: String,
    pub destination_zone: String,
    pub destination_address: String,
    pub destination_port: String,
    pub direction: AccessDirection,
    pub protocol: PolicyProtocol,
    pub description: String,
    pub valid_until: String,
}

impl Default for NetworkPolicyFormData {
    fn default() -> Self {
        Self {
            title: String::new(),
            organization: String::new(),
            applicant: String::new(),
            applicant_account: String::new(),
            department: String::new(),
            source_zone: "any".to_string(),
            source_address: String::new(),
            source_port: "any".to_string(),
            destination_zone: "any".to_string(),
            destination_address: String::new(),
            destination_port: "any".to_string(),
            direction: AccessDirection::Outbound,
            protocol: PolicyProtocol::Tcp,
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
            applicant_account: req.applicant_account.clone(),
            department: req.department.clone(),
            source_zone: req.source_zone.clone(),
            source_address: req.source_address.clone(),
            source_port: req.source_port.clone(),
            destination_zone: req.destination_zone.clone(),
            destination_address: req.destination_address.clone(),
            destination_port: req.destination_port.clone(),
            direction: req.direction.clone(),
            protocol: req.protocol.clone(),
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
            applicant_account: self.applicant_account.clone(),
            department: self.department.clone(),
            source_zone: self.source_zone.clone(),
            source_address: self.source_address.clone(),
            source_port: self.source_port.clone(),
            destination_zone: self.destination_zone.clone(),
            destination_address: self.destination_address.clone(),
            destination_port: self.destination_port.clone(),
            direction: self.direction.clone(),
            protocol: self.protocol.clone(),
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
        if self.organization.trim().is_empty() {
            return Err("申请单位不能为空，请先完善当前账号资料".to_string());
        }
        if self.applicant.trim().is_empty() {
            return Err("申请人不能为空，请先完善当前账号资料".to_string());
        }
        if self.department.trim().is_empty() {
            return Err("申请部门不能为空，请先完善当前账号资料".to_string());
        }
        if self.source_zone.trim().is_empty() {
            return Err("源网络区域不能为空".to_string());
        }
        if self.destination_zone.trim().is_empty() {
            return Err("目的网络区域不能为空".to_string());
        }
        if self.source_port.trim().is_empty() {
            return Err("源端口不能为空".to_string());
        }
        if self.destination_port.trim().is_empty() {
            return Err("目的端口不能为空".to_string());
        }
        if self.valid_until.trim().is_empty() {
            return Err("到期时间不能为空".to_string());
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
    let profile_warning = current_user
        .as_ref()
        .and_then(|user| user.ticket_profile_warning());
    let save_blocked = profile_warning.is_some();

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
                .map(|user| user.requester_name())
                .unwrap_or_default(),
            applicant_account: current_user
                .as_ref()
                .map(|user| user.username.clone())
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
                    save_disabled: save_blocked,
                    on_save: move |_| {
                        if save_blocked {
                            return;
                        }
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

                if let Some(message) = profile_warning.clone() {
                    div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                        "{message}"
                    }
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
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "申请账户 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-200 bg-gray-50 text-gray-500 rounded-lg cursor-not-allowed",
                                value: "{form_data.read().applicant_account}",
                                readonly: true,
                                disabled: true,
                            }
                        }
                    }
                }

                // 网络配置
                div { class: "border-b border-gray-200 pb-4",
                    h4 { class: "text-sm font-semibold text-gray-800 mb-3", "网络配置" }
                    div { class: "grid grid-cols-2 gap-4",
                        // 有效期
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "到期时间 "
                                span { class: "text-red-500", "*" }
                            }
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
                        // 源网络区域
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "源网络区域"
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: "{form_data.read().source_zone}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.source_zone = e.value();
                                },
                                option { value: "any", "any" }
                                for zone_name in network_zones.read().iter() {
                                    option {
                                        value: "{zone_name}",
                                        selected: form_data.read().source_zone == *zone_name,
                                        "{zone_name}"
                                    }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "源IP" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                placeholder: "如：10.0.0.1/32，留空可填 any",
                                value: "{form_data.read().source_address}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.source_address = e.value();
                                },
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "源端口 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                placeholder: "如：any、443、1024-65535",
                                value: "{form_data.read().source_port}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.source_port = e.value();
                                },
                            }
                        }
                        // 目标网络区域
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "目的网络区域"
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                value: "{form_data.read().destination_zone}",
                                onchange: move |e| {
                                    let mut data = form_data.write();
                                    data.destination_zone = e.value();
                                },
                                option { value: "any", "any" }
                                for zone_name in network_zones.read().iter() {
                                    option {
                                        value: "{zone_name}",
                                        selected: form_data.read().destination_zone == *zone_name,
                                        "{zone_name}"
                                    }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "目的IP" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                placeholder: "如：172.16.0.10/32，留空可填 any",
                                value: "{form_data.read().destination_address}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.destination_address = e.value();
                                },
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "目的端口 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                placeholder: "如：any、80、8080-8090",
                                value: "{form_data.read().destination_port}",
                                oninput: move |e| {
                                    let mut data = form_data.write();
                                    data.destination_port = e.value();
                                },
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
