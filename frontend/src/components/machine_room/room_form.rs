use dioxus::prelude::*;
use crate::state::machine_room::MachineRoomConfig;
use crate::app::PROVIDERS_STATE;
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
            FormMode::New => "添加机房",
            FormMode::Edit => "编辑机房",
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
pub struct RoomFormData {
    pub room_name: String,
    pub room_code: String,
    pub facility_type: String,
    pub address: String,
    pub provider_id: i32,
    pub room_type: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub floor: String,
    pub cabinet_count: String,
    pub area_size: String,
    pub remarks: String,
    pub status: String,
}

impl Default for RoomFormData {
    fn default() -> Self {
        Self {
            room_name: String::new(),
            room_code: String::new(),
            facility_type: "服务商机房".to_string(),
            address: String::new(),
            provider_id: 1,
            room_type: "DMZ机房（政务云）".to_string(),
            contact_person: String::new(),
            contact_phone: String::new(),
            floor: String::new(),
            cabinet_count: String::new(),
            area_size: String::new(),
            remarks: String::new(),
            status: "active".to_string(),
        }
    }
}

impl From<&MachineRoomConfig> for RoomFormData {
    fn from(room: &MachineRoomConfig) -> Self {
        Self {
            room_name: room.room_name.clone(),
            room_code: room.room_code.clone(),
            facility_type: room.facility_type.clone(),
            address: room.address.clone(),
            provider_id: room.provider_id,
            room_type: room.room_type.clone(),
            contact_person: room.contact_person.clone(),
            contact_phone: room.contact_phone.clone(),
            floor: room.floor.clone().unwrap_or_default(),
            cabinet_count: room.cabinet_count.map(|v| v.to_string()).unwrap_or_default(),
            area_size: room.area_size.clone().unwrap_or_default(),
            remarks: room.remarks.clone().unwrap_or_default(),
            status: room.status.clone(),
        }
    }
}

impl RoomFormData {
    /// 转换为 MachineRoomConfig
    pub fn to_config(&self, id: i32, created_at: String, updated_at: Option<String>) -> MachineRoomConfig {
        MachineRoomConfig {
            id,
            room_name: self.room_name.clone(),
            room_code: self.room_code.clone(),
            facility_type: self.facility_type.clone(),
            address: self.address.clone(),
            provider_id: self.provider_id,
            room_type: self.room_type.clone(),
            contact_person: self.contact_person.clone(),
            contact_phone: self.contact_phone.clone(),
            floor: if self.floor.is_empty() { None } else { Some(self.floor.clone()) },
            cabinet_count: self.cabinet_count.parse().ok(),
            area_size: if self.area_size.is_empty() { None } else { Some(self.area_size.clone()) },
            remarks: if self.remarks.is_empty() { None } else { Some(self.remarks.clone()) },
            status: self.status.clone(),
            created_at,
            updated_at,
        }
    }

    /// 验证表单数据
    pub fn validate(&self) -> Result<(), String> {
        if self.room_name.trim().is_empty() {
            return Err("机房名称不能为空".to_string());
        }
        if self.address.trim().is_empty() {
            return Err("详细地址不能为空".to_string());
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

/// 机房表单组件属性
#[derive(Props, Clone, PartialEq)]
pub struct RoomFormProps {
    /// 表单模式
    mode: FormMode,
    /// 编辑时的原始机房数据
    #[props(default = None)]
    room: Option<MachineRoomConfig>,
    /// 保存事件
    on_save: EventHandler<MachineRoomConfig>,
    /// 关闭事件
    on_close: EventHandler<()>,
}

/// 机房表单组件 - 处理新增和编辑的通用组件
#[component]
pub fn RoomForm(props: RoomFormProps) -> Element {
    // 初始化表单数据
    let initial_data = props.room.as_ref()
        .map(RoomFormData::from)
        .unwrap_or_default();

    let mut form_data = use_signal(|| initial_data);
    let mut error_msg = use_signal(String::new);

    // 获取服务商列表
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
                        if let Err(e) = data.validate() {
                            error_msg.set(e);
                            return;
                        }
                        error_msg.set(String::new());

                        // 根据模式构建机房配置对象
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
                                if let Some(original) = props.room.as_ref() {
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

                // 基本信息
                div { class: "grid grid-cols-2 gap-4",
                    // 机房名称
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "机房名称 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().room_name}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.room_name = e.value();
                            },
                        }
                    }

                    // 机房编码
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "机房编码" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().room_code}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.room_code = e.value();
                            },
                        }
                    }

                    // 所属服务商
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "所属服务商 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
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

                    // 设施类型
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "设施类型 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().facility_type}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                let new_facility_type = e.value();
                                data.facility_type = new_facility_type.clone();
                                // 设施类型改变时重置机房类型
                                if new_facility_type == "数据中心" {
                                    data.room_type = "核心机房".to_string();
                                } else if data.room_type == "核心机房" {
                                    data.room_type = "DMZ机房（政务云）".to_string();
                                }
                            },
                            option { value: "数据中心", "数据中心" }
                            option { value: "服务商机房", "服务商机房" }
                        }
                    }

                    // 机房类型
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "机房类型 "
                            span { class: "text-red-500", "*" }
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().room_type}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.room_type = e.value();
                            },
                            // 根据设施类型动态显示机房类型选项
                            if form_data.read().facility_type == "数据中心" {
                                option { value: "核心机房", "核心机房" }
                            } else {
                                option { value: "DMZ机房（公有云）", "DMZ机房（公有云）" }
                                option { value: "DMZ机房（政务云）", "DMZ机房（政务云）" }
                            }
                        }
                    }

                    // 详细地址
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "详细地址 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().address}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.address = e.value();
                            },
                        }
                    }

                    // 负责人
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "负责人 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().contact_person}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.contact_person = e.value();
                            },
                        }
                    }

                    // 联系电话
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "联系电话 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().contact_phone}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.contact_phone = e.value();
                            },
                        }
                    }

                    // 楼层
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "楼层" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().floor}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.floor = e.value();
                            },
                        }
                    }

                    // 机柜数量
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "机柜数量" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "请输入机柜数量",
                            value: "{form_data.read().cabinet_count}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.cabinet_count = e.value();
                            },
                        }
                    }

                    // 面积
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "面积 (㎡)" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            placeholder: "请输入面积",
                            value: "{form_data.read().area_size}",
                            oninput: move |e| {
                                let mut data = form_data.write();
                                data.area_size = e.value();
                            },
                        }
                    }

                    // 备注
                    div { class: "col-span-2",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            rows: 3,
                            placeholder: "请输入备注信息",
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
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{form_data.read().status}",
                            onchange: move |e| {
                                let mut data = form_data.write();
                                data.status = e.value();
                            },
                            option { value: "active", "运行中" }
                            option { value: "inactive", "停用" }
                        }
                    }
                }
            },
        }
    }
}
