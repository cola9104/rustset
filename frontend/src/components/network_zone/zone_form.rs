use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaPlus, FaXmark};
use crate::state::network_zone::NetworkZone;
use crate::app::{NETWORK_ZONES_STATE, CLOUD_PLATFORMS_STATE, MACHINE_ROOMS_STATE};
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
            FormMode::New => "添加网络区域",
            FormMode::Edit => "编辑网络区域",
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
pub struct ZoneFormData {
    pub name: String,
    pub description: String,
    pub zone_type: String, // "cloud" 或 "physical"
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub cidr_blocks: Vec<String>,
    pub ip_ranges: Vec<String>,
    pub is_active: bool,
}

impl From<&NetworkZone> for ZoneFormData {
    fn from(zone: &NetworkZone) -> Self {
        Self {
            name: zone.name.clone(),
            description: zone.description.clone(),
            zone_type: if zone.is_cloud_zone() { "cloud".to_string() } else { "physical".to_string() },
            cloud_platform_id: zone.cloud_platform_id,
            machine_room_id: zone.machine_room_id,
            cidr_blocks: zone.cidr_blocks.clone(),
            ip_ranges: zone.ip_ranges.clone(),
            is_active: zone.is_active,
        }
    }
}

impl ZoneFormData {
    /// 转换为 NetworkZone
    pub fn to_zone(&self, id: i32) -> NetworkZone {
        NetworkZone {
            id,
            name: self.name.clone(),
            cloud_platform_id: if self.zone_type == "cloud" { self.cloud_platform_id } else { None },
            machine_room_id: if self.zone_type == "physical" { self.machine_room_id } else { None },
            cidr_blocks: self.cidr_blocks.clone(),
            ip_ranges: self.ip_ranges.clone(),
            description: self.description.clone(),
            is_active: self.is_active,
        }
    }

    /// 验证表单数据
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("区域名称不能为空".to_string());
        }
        if self.zone_type == "cloud" && self.cloud_platform_id.is_none() {
            return Err("请选择云平台".to_string());
        }
        if self.zone_type == "physical" && self.machine_room_id.is_none() {
            return Err("请选择机房".to_string());
        }
        Ok(())
    }
}

/// 网络区域表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct ZoneFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始区域数据
    #[props(default = None)]
    zone: Option<NetworkZone>,
    /// 保存事件
    on_save: EventHandler<NetworkZone>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 网络区域表单组件
#[component]
pub fn ZoneForm(props: ZoneFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props.zone.as_ref()
        .map(ZoneFormData::from)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(|| String::new());
    let mut new_cidr = use_signal(|| String::new());

    // 获取云平台和机房列表
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read().clone();
    let machine_rooms = MACHINE_ROOMS_STATE.read().clone();

    // 获取当前编辑的区域ID
    let editing_zone_id = props.zone.as_ref().map(|z| z.id);

    // 计算已被分配的云平台ID（编辑时排除当前区域）
    let assigned_cloud_ids: std::collections::HashSet<i32> = NETWORK_ZONES_STATE.read()
        .iter()
        .filter(|z| Some(z.id) != editing_zone_id)
        .filter_map(|z| z.cloud_platform_id)
        .collect();

    // 计算已被分配的机房ID（编辑时排除当前区域）
    let assigned_room_ids: std::collections::HashSet<i32> = NETWORK_ZONES_STATE.read()
        .iter()
        .filter(|z| Some(z.id) != editing_zone_id)
        .filter_map(|z| z.machine_room_id)
        .collect();

    // 删除 CIDR 网段
    let mut remove_cidr = move |index: usize| {
        let mut data = form_data.write();
        data.cidr_blocks.remove(index);
    };

    // 删除 IP 范围
    let mut remove_ip_range = move |index: usize| {
        let mut data = form_data.write();
        data.ip_ranges.remove(index);
    };

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

                        // 检查名称是否重复
                        let name_exists = NETWORK_ZONES_STATE.read()
                            .iter()
                            .filter(|z| Some(z.id) != props.zone.as_ref().map(|z| z.id))
                            .any(|z| z.name == data.name);
                        if name_exists {
                            error_msg.set("网络区域名称已存在".to_string());
                            return;
                        }

                        // 根据模式构建区域对象
                        let zone = match props.mode {
                            FormMode::New => {
                                let max_id = NETWORK_ZONES_STATE.read()
                                    .iter()
                                    .map(|z| z.id)
                                    .max()
                                    .unwrap_or(0) + 1;
                                data.to_zone(max_id)
                            }
                            FormMode::Edit => {
                                if let Some(ref original) = props.zone.as_ref() {
                                    data.to_zone(original.id)
                                } else {
                                    return;
                                }
                            }
                        };

                        props.on_save.call(zone);
                    },
                    on_cancel: move |_| props.on_close.call(()),
                }
            },

            div { class: "space-y-4",
                // 错误提示
                if !error_msg.read().is_empty() {
                    ErrorMessage { message: error_msg.read().clone() }
                }

                // 区域类型选择
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "区域类型 *" }
                    div { class: "flex gap-4",
                        label { class: "flex items-center gap-2 cursor-pointer",
                            input {
                                r#type: "radio",
                                class: "w-4 h-4 text-blue-600 focus:ring-blue-500",
                                name: "zone_type",
                                checked: "{form_data.read().zone_type == \"cloud\"}",
                                oninput: move |_| {
                                    let mut data = form_data.write();
                                    data.zone_type = "cloud".to_string();
                                    data.machine_room_id = None;
                                }
                            }
                            span { class: "text-sm text-gray-700", "云平台区域" }
                        }
                        label { class: "flex items-center gap-2 cursor-pointer",
                            input {
                                r#type: "radio",
                                class: "w-4 h-4 text-blue-600 focus:ring-blue-500",
                                name: "zone_type",
                                checked: "{form_data.read().zone_type == \"physical\"}",
                                oninput: move |_| {
                                    let mut data = form_data.write();
                                    data.zone_type = "physical".to_string();
                                    data.cloud_platform_id = None;
                                }
                            }
                            span { class: "text-sm text-gray-700", "物理机房区域" }
                        }
                    }
                }

                // 关联云平台（仅当选择云平台区域时显示）
                if form_data.read().zone_type == "cloud" {
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "关联云平台 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            value: "{form_data.read().cloud_platform_id.unwrap_or(-1)}",
                            onchange: move |e| {
                                let val: i32 = e.value().parse().unwrap_or(-1);
                                let mut data = form_data.write();
                                data.cloud_platform_id = if val > 0 { Some(val) } else { None };
                            },
                            option { value: "-1", "请选择云平台" }
                            for platform in cloud_platforms.iter() {
                                if !assigned_cloud_ids.contains(&platform.id) {
                                    option {
                                        value: "{platform.id}",
                                        selected: form_data.read().cloud_platform_id == Some(platform.id),
                                        "{platform.foundation} ({platform.platform_name})"
                                    }
                                }
                            }
                        }
                        p { class: "mt-1 text-xs text-gray-500", "云网络区域绑定到云平台配置" }
                    }
                }

                // 关联机房（仅当选择物理机房区域时显示）
                if form_data.read().zone_type == "physical" {
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "关联机房 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            value: "{form_data.read().machine_room_id.unwrap_or(-1)}",
                            onchange: move |e| {
                                let val: i32 = e.value().parse().unwrap_or(-1);
                                let mut data = form_data.write();
                                data.machine_room_id = if val > 0 { Some(val) } else { None };
                            },
                            option { value: "-1", "请选择机房" }
                            for room in machine_rooms.iter() {
                                if !assigned_room_ids.contains(&room.id) {
                                    option {
                                        value: "{room.id}",
                                        selected: form_data.read().machine_room_id == Some(room.id),
                                        "{room.room_name} - {room.facility_type}"
                                    }
                                }
                            }
                        }
                        p { class: "mt-1 text-xs text-gray-500", "物理网络区域绑定到机房配置" }
                    }
                }

                // 区域名称
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1",
                        "区域名称 "
                        span { class: "text-red-500", "*" }
                    }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        placeholder: "例如: 阿里云, 市政务云机房A内网",
                        value: "{form_data.read().name}",
                        oninput: move |e| {
                            let mut data = form_data.write();
                            data.name = e.value();
                        },
                    }
                }

                // 网段配置部分
                div { class: "border-t border-gray-200 pt-4 mt-4",
                    h4 { class: "text-sm font-semibold text-gray-800 mb-3", "网段配置" }
                    p { class: "text-xs text-gray-500 mb-3", "支持 CIDR (192.168.1.0/24)、IP 范围 (192.168.1.1-192.168.1.100) 或单个 IP (10.0.0.1)" }

                    div { class: "flex gap-2 mb-3",
                        input {
                            r#type: "text",
                            class: "flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "输入网段/IP，按回车快速添加",
                            value: "{new_cidr}",
                            oninput: move |e| new_cidr.set(e.value()),
                            onkeydown: move |e| {
                                if e.key().to_string() == "Enter" {
                                    let cidr = new_cidr.read().trim().to_string();
                                    if !cidr.is_empty() {
                                        let is_cidr = cidr.contains('/');
                                        let mut data = form_data.write();
                                        if is_cidr {
                                            if !data.cidr_blocks.contains(&cidr) {
                                                data.cidr_blocks.push(cidr);
                                            }
                                        } else {
                                            if !data.ip_ranges.contains(&cidr) {
                                                data.ip_ranges.push(cidr);
                                            }
                                        }
                                    }
                                    new_cidr.set(String::new());
                                }
                            },
                        }
                        button {
                            class: "px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2",
                            onclick: move |_| {
                                let cidr = new_cidr.read().trim().to_string();
                                if !cidr.is_empty() {
                                    let is_cidr = cidr.contains('/');
                                    let mut data = form_data.write();
                                    if is_cidr {
                                        if !data.cidr_blocks.contains(&cidr) {
                                            data.cidr_blocks.push(cidr);
                                        }
                                    } else {
                                        if !data.ip_ranges.contains(&cidr) {
                                            data.ip_ranges.push(cidr);
                                        }
                                    }
                                    new_cidr.set(String::new());
                                }
                            },
                            Icon { icon: FaPlus, width: 16, height: 16 }
                            "添加"
                        }
                    }

                    // 合并显示所有网段/IP
                    div { class: "space-y-1",
                        // CIDR 网段
                        for (index, cidr) in form_data.read().cidr_blocks.iter().enumerate() {
                            div { class: "flex items-center justify-between p-2 bg-blue-50 rounded-lg border border-blue-100",
                                div { class: "flex items-center gap-2",
                                    span { class: "inline-flex items-center px-2 py-0.5 text-xs font-medium rounded bg-blue-100 text-blue-800", "CIDR" }
                                    span { class: "text-sm text-gray-700 font-mono", "{cidr}" }
                                }
                                button {
                                    class: "text-red-600 hover:text-red-800 p-1 hover:bg-red-50 rounded",
                                    onclick: move |_| remove_cidr(index),
                                    Icon { icon: FaXmark, width: 16, height: 16 }
                                }
                            }
                        }
                        // IP 范围
                        for (index, range) in form_data.read().ip_ranges.iter().enumerate() {
                            div { class: "flex items-center justify-between p-2 bg-green-50 rounded-lg border border-green-100",
                                div { class: "flex items-center gap-2",
                                    span { class: "inline-flex items-center px-2 py-0.5 text-xs font-medium rounded bg-green-100 text-green-800", "IP" }
                                    span { class: "text-sm text-gray-700 font-mono", "{range}" }
                                }
                                button {
                                    class: "text-red-600 hover:text-red-800 p-1 hover:bg-red-50 rounded",
                                    onclick: move |_| remove_ip_range(index),
                                    Icon { icon: FaXmark, width: 16, height: 16 }
                                }
                            }
                        }
                        // 空状态提示
                        if form_data.read().cidr_blocks.is_empty() && form_data.read().ip_ranges.is_empty() {
                            div { class: "text-center py-4 text-sm text-gray-400 italic", "暂未配置网段" }
                        }
                    }
                }

                // 描述
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "描述" }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        rows: 2,
                        placeholder: "描述该区域的用途",
                        value: "{form_data.read().description}",
                        oninput: move |e| {
                            let mut data = form_data.write();
                            data.description = e.value();
                        },
                    }
                }

                // 激活状态
                div {
                    label { class: "flex items-center gap-2 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500",
                            checked: "{form_data.read().is_active}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.is_active = e.checked();
                            },
                        }
                        span { class: "text-sm text-gray-700", "启用该区域" }
                    }
                }
            },
        }
    }
}
