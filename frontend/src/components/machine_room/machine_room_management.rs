use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaPlus, FaPenToSquare, FaEye, FaTrash, FaMagnifyingGlass, FaBuilding};
use crate::state::machine_room::MachineRoomConfig;
use crate::app::MACHINE_ROOMS_STATE;
use crate::app::PROVIDERS_STATE;

/// 机房管理页面
#[component]
pub fn MachineRoomManagement() -> Element {
    let mut show_add_modal = use_signal(|| false);
    let mut show_edit_modal = use_signal(|| false);
    let mut show_view_modal = use_signal(|| false);
    let mut selected_room = use_signal(|| None::<MachineRoomConfig>);
    let mut search_query = use_signal(|| String::new());
    let mut provider_filter = use_signal(|| String::from("all"));
    let mut room_type_filter = use_signal(|| String::from("all"));

    // 获取服务商名称的辅助函数
    let get_provider_name = |provider_id: i32| -> String {
        PROVIDERS_STATE.read()
            .iter()
            .find(|p| p.id == provider_id)
            .map(|p| p.short_name.clone())
            .unwrap_or_else(|| "未知服务商".to_string())
    };

    // 过滤机房
    let filtered_rooms = MACHINE_ROOMS_STATE.read().iter().filter(|room| {
        let matches_search = search_query.read().is_empty()
            || room.room_name.contains(search_query.read().as_str())
            || room.facility_type.contains(search_query.read().as_str())
            || room.room_code.contains(search_query.read().as_str());

        let matches_provider = provider_filter.read().as_str() == "all"
            || provider_filter.read().parse::<i32>().ok() == Some(room.provider_id);

        let matches_type = room_type_filter.read().as_str() == "all"
            || room_type_filter.read().as_str() == room.room_type;

        matches_search && matches_provider && matches_type
    }).cloned().collect::<Vec<_>>();

    // 统计数据
    let total_count = MACHINE_ROOMS_STATE.read().len() as i32;
    let active_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.status == "active").count() as i32;
    let core_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.room_type == "核心机房").count() as i32;
    let dmz_public_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.room_type == "DMZ机房（公有云）").count() as i32;
    let dmz_gov_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.room_type == "DMZ机房（政务云）").count() as i32;

    // 按服务商统计
    let telecom_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.provider_id == 1).count() as i32;
    let unicom_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.provider_id == 2).count() as i32;
    let mobile_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.provider_id == 3).count() as i32;
    let broadcasting_count = MACHINE_ROOMS_STATE.read().iter().filter(|r| r.provider_id == 4).count() as i32;

    // 预计算服务商名称
    let rooms_with_provider_names: Vec<(MachineRoomConfig, String)> = filtered_rooms
        .iter()
        .map(|room| {
            let provider_name = get_provider_name(room.provider_id);
            (room.clone(), provider_name)
        })
        .collect();

    rsx! {
        div { class: "p-6 bg-white min-h-screen",
            // 页面标题
            div { class: "flex justify-between items-center mb-6",
                div { class: "flex items-center gap-3",
                    Icon { icon: FaBuilding, width: 28, height: 28, class: "text-blue-600" }
                    h1 { class: "text-2xl font-bold text-gray-800", "机房管理" }
                }
                button {
                    class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors",
                    onclick: move |_| {
                        show_add_modal.set(true);
                    },
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { "添加机房" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-7 gap-4 mb-6",
                div { class: "bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "机房总数" }
                    div { class: "text-3xl font-bold mt-1", "{total_count}" }
                }
                div { class: "bg-gradient-to-br from-green-500 to-green-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "运行中" }
                    div { class: "text-3xl font-bold mt-1", "{active_count}" }
                }
                div { class: "bg-gradient-to-br from-purple-500 to-purple-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "核心机房" }
                    div { class: "text-3xl font-bold mt-1", "{core_count}" }
                }
                div { class: "bg-gradient-to-br from-orange-500 to-orange-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "DMZ公有云" }
                    div { class: "text-3xl font-bold mt-1", "{dmz_public_count}" }
                }
                div { class: "bg-gradient-to-br from-pink-500 to-pink-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "DMZ政务云" }
                    div { class: "text-3xl font-bold mt-1", "{dmz_gov_count}" }
                }
                div { class: "bg-gradient-to-br from-sky-500 to-sky-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "电信" }
                    div { class: "text-3xl font-bold mt-1", "{telecom_count}" }
                }
                div { class: "bg-gradient-to-br from-yellow-500 to-yellow-600 rounded-lg p-4 text-white",
                    div { class: "text-sm opacity-80", "联通" }
                    div { class: "text-3xl font-bold mt-1", "{unicom_count}" }
                }
            }

            // 搜索和筛选
            div { class: "bg-gray-50 rounded-lg p-4 mb-6",
                div { class: "flex flex-wrap gap-4",
                    div { class: "flex-1 min-w-64",
                        div { class: "relative",
                            Icon { icon: FaMagnifyingGlass, width: 16, height: 16, class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" }
                            input {
                                r#type: "text",
                                class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "搜索机房名称、位置、编码...",
                                value: "{search_query}",
                                oninput: move |e| {
                                    search_query.set(e.value());
                                }
                            }
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        value: "{provider_filter}",
                        onchange: move |e| {
                            provider_filter.set(e.value());
                        },
                        option { value: "all", "全部服务商" }
                        for provider in PROVIDERS_STATE.read().iter() {
                            option { value: "{provider.id}", "{provider.short_name}" }
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        value: "{room_type_filter}",
                        onchange: move |e| {
                            room_type_filter.set(e.value());
                        },
                        option { value: "all", "全部类型" }
                        option { value: "核心机房", "核心机房" }
                        option { value: "DMZ机房（公有云）", "DMZ机房（公有云）" }
                        option { value: "DMZ机房（政务云）", "DMZ机房（政务云）" }
                    }
                }
            }

            // 机房列表表格
            div { class: "bg-white rounded-lg border border-gray-200 overflow-hidden",
                div { class: "overflow-x-auto",
                    table { class: "w-full",
                        thead {
                            tr { class: "bg-gray-50 border-b border-gray-200",
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "机房名称" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "机房编码" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "设施类型" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "机房类型" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "负责人" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "机柜数" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                            }
                        }
                        tbody {
                            if filtered_rooms.is_empty() {
                                tr {
                                    td { class: "px-6 py-8 text-center text-gray-500", colspan: "9",
                                        "暂无数据"
                                    }
                                }
                            } else {
                                for (room, provider_name) in rooms_with_provider_names.iter() {
                                    tr { class: "hover:bg-gray-50 border-b border-gray-100 transition-colors",
                                        td { class: "px-6 py-4",
                                            div { class: "font-medium text-gray-900", "{room.room_name}" }
                                            div { class: "text-sm text-gray-500", "{room.address}" }
                                        }
                                        td { class: "px-6 py-4 text-sm text-gray-600 font-mono", "{room.room_code}" }
                                        td { class: "px-6 py-4",
                                            div { class: "text-sm text-gray-900", "{room.facility_type}" }
                                            if let Some(floor) = &room.floor {
                                                div { class: "text-xs text-gray-500", "{floor}" }
                                            }
                                        }
                                        td { class: "px-6 py-4 text-sm",
                                            span {
                                                class: match room.provider_id {
                                                    1 => "px-2 py-1 rounded-full text-xs font-medium bg-sky-100 text-sky-800",
                                                    2 => "px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800",
                                                    3 => "px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800",
                                                    _ => "px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800",
                                                },
                                                "{provider_name}"
                                            }
                                        }
                                        td { class: "px-6 py-4 text-sm text-gray-600", "{room.room_type}" }
                                        td { class: "px-6 py-4",
                                            div { class: "text-sm text-gray-900", "{room.contact_person}" }
                                            div { class: "text-xs text-gray-500", "{room.contact_phone}" }
                                        }
                                        td { class: "px-6 py-4 text-sm text-gray-600",
                                            if let Some(count) = room.cabinet_count {
                                                "{count} 个"
                                            } else {
                                                "-"
                                            }
                                        }
                                        td { class: "px-6 py-4 text-sm",
                                            if room.status == "active" {
                                                span { class: "px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800", "运行中" }
                                            } else {
                                                span { class: "px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800", "停用" }
                                            }
                                        }
                                        td { class: "px-6 py-4 text-sm",
                                            div { class: "flex gap-2",
                                                button {
                                                    class: "text-blue-600 hover:text-blue-800 transition-colors",
                                                    title: "查看",
                                                    onclick: {
                                                        let room = room.clone();
                                                        move |_| {
                                                            selected_room.set(Some(room.clone()));
                                                            show_view_modal.set(true);
                                                        }
                                                    },
                                                    Icon { icon: FaEye, width: 16, height: 16 }
                                                }
                                                button {
                                                    class: "text-yellow-600 hover:text-yellow-800 transition-colors",
                                                    title: "编辑",
                                                    onclick: {
                                                        let room = room.clone();
                                                        move |_| {
                                                            selected_room.set(Some(room.clone()));
                                                            show_edit_modal.set(true);
                                                        }
                                                    },
                                                    Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                                }
                                                button {
                                                    class: "text-red-600 hover:text-red-800 transition-colors",
                                                    title: "删除",
                                                    onclick: {
                                                        let room_id = room.id;
                                                        move |_| {
                                                            MACHINE_ROOMS_STATE.write().retain(|r| r.id != room_id);
                                                        }
                                                    },
                                                    Icon { icon: FaTrash, width: 16, height: 16 }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 添加机房模态框
        if *show_add_modal.read() {
            AddRoomModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |room| {
                    MACHINE_ROOMS_STATE.write().push(room);
                    show_add_modal.set(false);
                }
            }
        }

        // 编辑机房模态框
        if *show_edit_modal.read() {
            if let Some(room) = &*selected_room.read() {
                EditRoomModal {
                    room: room.clone(),
                    on_close: move |_| show_edit_modal.set(false),
                    on_save: move |room: MachineRoomConfig| {
                        let id = room.id;
                        let idx = MACHINE_ROOMS_STATE.read().iter().position(|r| r.id == id);
                        if let Some(idx) = idx {
                            MACHINE_ROOMS_STATE.write()[idx] = room;
                        }
                        show_edit_modal.set(false);
                    }
                }
            }
        }

        // 查看机房模态框
        if *show_view_modal.read() {
            if let Some(room) = &*selected_room.read() {
                ViewRoomModal {
                    room: room.clone(),
                    on_close: move |_| show_view_modal.set(false)
                }
            }
        }
    }
}

/// 添加机房模态框
#[component]
fn AddRoomModal(
    on_close: EventHandler,
    on_save: EventHandler<MachineRoomConfig>
) -> Element {
    let mut room_name = use_signal(|| String::new());
    let mut room_code = use_signal(|| String::new());
    let mut facility_type = use_signal(|| String::from("服务商机房"));
    let mut address = use_signal(|| String::new());
    let mut provider_id = use_signal(|| 1);
    let mut room_type = use_signal(|| String::from("DMZ机房（政务云）"));
    let mut contact_person = use_signal(|| String::new());
    let mut contact_phone = use_signal(|| String::new());
    let mut floor = use_signal(|| String::new());
    let mut cabinet_count = use_signal(|| String::new());
    let mut area_size = use_signal(|| String::new());
    let mut remarks = use_signal(|| String::new());
    let mut status = use_signal(|| String::from("active"));

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-bold text-gray-800", "添加机房" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房名称 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{room_name}",
                                oninput: move |e| room_name.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房编码" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{room_code}",
                                oninput: move |e| room_code.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "所属服务商 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{provider_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        provider_id.set(id);
                                    }
                                },
                                for provider in PROVIDERS_STATE.read().iter() {
                                    option { value: "{provider.id}", "{provider.short_name}" }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "设施类型 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{facility_type}",
                                onchange: move |e| {
                                    let new_facility_type = e.value();
                                    facility_type.set(new_facility_type.clone());
                                    // 设施类型改变时重置机房类型
                                    // 数据中心只能选核心机房
                                    // 服务商机房只能选DMZ机房（公有云）或DMZ机房（政务云）
                                    if new_facility_type == "数据中心" {
                                        room_type.set("核心机房".to_string());
                                    } else if *room_type.read() == "核心机房" {
                                        room_type.set("DMZ机房（政务云）".to_string());
                                    }
                                },
                                option { value: "数据中心", "数据中心" }
                                option { value: "服务商机房", "服务商机房" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房类型 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{room_type}",
                                onchange: move |e| room_type.set(e.value()),
                                // 根据设施类型动态显示机房类型选项
                                if *facility_type.read() == "数据中心" {
                                    // 数据中心只能选核心机房
                                    option { value: "核心机房", "核心机房" }
                                } else {
                                    // 服务商机房只能选DMZ机房（公有云）或DMZ机房（政务云）
                                    option { value: "DMZ机房（公有云）", "DMZ机房（公有云）" }
                                    option { value: "DMZ机房（政务云）", "DMZ机房（政务云）" }
                                }
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "详细地址 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{address}",
                                oninput: move |e| address.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{contact_person}",
                                oninput: move |e| contact_person.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系电话 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{contact_phone}",
                                oninput: move |e| contact_phone.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "楼层" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{floor}",
                                oninput: move |e| floor.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机柜数量" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{cabinet_count}",
                                oninput: move |e| cabinet_count.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "面积 (㎡)" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{area_size}",
                                oninput: move |e| area_size.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                rows: "3",
                                value: "{remarks}",
                                oninput: move |e| remarks.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{status}",
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "运行中" }
                                option { value: "inactive", "停用" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-3 p-6 border-t bg-gray-50",
                    button {
                        class: "px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |_| {
                            let new_id = MACHINE_ROOMS_STATE.read().iter().map(|r| r.id).max().unwrap_or(0) + 1;
                            let room = MachineRoomConfig {
                                id: new_id,
                                room_name: room_name.read().clone(),
                                room_code: room_code.read().clone(),
                                facility_type: facility_type.read().clone(),
                                address: address.read().clone(),
                                provider_id: *provider_id.read(),
                                room_type: room_type.read().clone(),
                                contact_person: contact_person.read().clone(),
                                contact_phone: contact_phone.read().clone(),
                                floor: if floor.read().is_empty() { None } else { Some(floor.read().clone()) },
                                cabinet_count: cabinet_count.read().parse().ok(),
                                area_size: if area_size.read().is_empty() { None } else { Some(area_size.read().clone()) },
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                status: status.read().clone(),
                                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                updated_at: None,
                            };
                            on_save.call(room);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 编辑机房模态框
#[component]
fn EditRoomModal(
    room: MachineRoomConfig,
    on_close: EventHandler,
    on_save: EventHandler<MachineRoomConfig>
) -> Element {
    let mut room_name = use_signal(|| room.room_name.clone());
    let mut room_code = use_signal(|| room.room_code.clone());
    let mut facility_type = use_signal(|| room.facility_type.clone());
    let mut address = use_signal(|| room.address.clone());
    let mut provider_id = use_signal(|| room.provider_id);
    let mut room_type = use_signal(|| room.room_type.clone());
    let mut contact_person = use_signal(|| room.contact_person.clone());
    let mut contact_phone = use_signal(|| room.contact_phone.clone());
    let mut floor = use_signal(|| room.floor.clone().unwrap_or_default());
    let mut cabinet_count = use_signal(|| room.cabinet_count.map(|v| v.to_string()).unwrap_or_default());
    let mut area_size = use_signal(|| room.area_size.clone().unwrap_or_default());
    let mut remarks = use_signal(|| room.remarks.clone().unwrap_or_default());
    let mut status = use_signal(|| room.status.clone());

    // 当 room prop 变化时更新所有信号
    use_effect(move || {
        room_name.set(room.room_name.clone());
        room_code.set(room.room_code.clone());
        facility_type.set(room.facility_type.clone());
        address.set(room.address.clone());
        provider_id.set(room.provider_id);
        room_type.set(room.room_type.clone());
        contact_person.set(room.contact_person.clone());
        contact_phone.set(room.contact_phone.clone());
        floor.set(room.floor.clone().unwrap_or_default());
        cabinet_count.set(room.cabinet_count.map(|v| v.to_string()).unwrap_or_default());
        area_size.set(room.area_size.clone().unwrap_or_default());
        remarks.set(room.remarks.clone().unwrap_or_default());
        status.set(room.status.clone());
    });

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-bold text-gray-800", "编辑机房" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房名称 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{room_name}",
                                oninput: move |e| room_name.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房编码" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{room_code}",
                                oninput: move |e| room_code.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "所属服务商 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{provider_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        provider_id.set(id);
                                    }
                                },
                                for provider in PROVIDERS_STATE.read().iter() {
                                    option { value: "{provider.id}", "{provider.short_name}" }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "设施类型 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{facility_type}",
                                onchange: move |e| {
                                    let new_facility_type = e.value();
                                    facility_type.set(new_facility_type.clone());
                                    // 设施类型改变时重置机房类型
                                    // 数据中心只能选核心机房
                                    // 服务商机房只能选DMZ机房（公有云）或DMZ机房（政务云）
                                    if new_facility_type == "数据中心" {
                                        room_type.set("核心机房".to_string());
                                    } else if *room_type.read() == "核心机房" {
                                        room_type.set("DMZ机房（政务云）".to_string());
                                    }
                                },
                                option { value: "数据中心", "数据中心" }
                                option { value: "服务商机房", "服务商机房" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房类型 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{room_type}",
                                onchange: move |e| room_type.set(e.value()),
                                // 根据设施类型动态显示机房类型选项
                                if *facility_type.read() == "数据中心" {
                                    // 数据中心只能选核心机房
                                    option { value: "核心机房", "核心机房" }
                                } else {
                                    // 服务商机房只能选DMZ机房（公有云）或DMZ机房（政务云）
                                    option { value: "DMZ机房（公有云）", "DMZ机房（公有云）" }
                                    option { value: "DMZ机房（政务云）", "DMZ机房（政务云）" }
                                }
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "详细地址 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{address}",
                                oninput: move |e| address.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{contact_person}",
                                oninput: move |e| contact_person.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系电话 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{contact_phone}",
                                oninput: move |e| contact_phone.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "楼层" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{floor}",
                                oninput: move |e| floor.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机柜数量" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{cabinet_count}",
                                oninput: move |e| cabinet_count.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "面积 (㎡)" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{area_size}",
                                oninput: move |e| area_size.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                rows: "3",
                                value: "{remarks}",
                                oninput: move |e| remarks.set(e.value())
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                value: "{status}",
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "运行中" }
                                option { value: "inactive", "停用" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-3 p-6 border-t bg-gray-50",
                    button {
                        class: "px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |_| {
                            let updated_room = MachineRoomConfig {
                                id: room.id,
                                room_name: room_name.read().clone(),
                                room_code: room_code.read().clone(),
                                facility_type: facility_type.read().clone(),
                                address: address.read().clone(),
                                provider_id: *provider_id.read(),
                                room_type: room_type.read().clone(),
                                contact_person: contact_person.read().clone(),
                                contact_phone: contact_phone.read().clone(),
                                floor: if floor.read().is_empty() { None } else { Some(floor.read().clone()) },
                                cabinet_count: cabinet_count.read().parse().ok(),
                                area_size: if area_size.read().is_empty() { None } else { Some(area_size.read().clone()) },
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                status: status.read().clone(),
                                created_at: room.created_at.clone(),
                                updated_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string()),
                            };
                            on_save.call(updated_room);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 查看机房模态框
#[component]
fn ViewRoomModal(
    room: MachineRoomConfig,
    on_close: EventHandler
) -> Element {
    // 获取服务商名称
    let provider_name = PROVIDERS_STATE.read()
        .iter()
        .find(|p| p.id == room.provider_id)
        .map(|p| p.short_name.clone())
        .unwrap_or_else(|| "未知服务商".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-bold text-gray-800", "机房详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "space-y-4",
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "col-span-2",
                                label { class: "block text-sm font-medium text-gray-500", "机房名称" }
                                p { class: "mt-1 text-lg font-semibold text-gray-900", "{room.room_name}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "机房编码" }
                                p { class: "mt-1 text-gray-900", "{room.room_code}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "所属服务商" }
                                p { class: "mt-1 text-gray-900", "{provider_name}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "设施类型" }
                                p { class: "mt-1 text-gray-900", "{room.facility_type}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "机房类型" }
                                p { class: "mt-1 text-gray-900", "{room.room_type}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "状态" }
                                p { class: "mt-1",
                                    if room.status == "active" {
                                        span { class: "px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800", "运行中" }
                                    } else {
                                        span { class: "px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800", "停用" }
                                    }
                                }
                            }
                            div { class: "col-span-2",
                                label { class: "block text-sm font-medium text-gray-500", "详细地址" }
                                p { class: "mt-1 text-gray-900", "{room.address}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "负责人" }
                                p { class: "mt-1 text-gray-900", "{room.contact_person}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "联系电话" }
                                p { class: "mt-1 text-gray-900", "{room.contact_phone}" }
                            }
                            if let Some(floor) = &room.floor {
                                div {
                                    label { class: "block text-sm font-medium text-gray-500", "楼层" }
                                    p { class: "mt-1 text-gray-900", "{floor}" }
                                }
                            }
                            if let Some(count) = room.cabinet_count {
                                div {
                                    label { class: "block text-sm font-medium text-gray-500", "机柜数量" }
                                    p { class: "mt-1 text-gray-900", "{count} 个" }
                                }
                            }
                            if let Some(size) = &room.area_size {
                                div {
                                    label { class: "block text-sm font-medium text-gray-500", "面积" }
                                    p { class: "mt-1 text-gray-900", "{size} ㎡" }
                                }
                            }
                            if let Some(remarks) = &room.remarks {
                                div { class: "col-span-2",
                                    label { class: "block text-sm font-medium text-gray-500", "备注" }
                                    p { class: "mt-1 text-gray-900", "{remarks}" }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-500", "创建时间" }
                                p { class: "mt-1 text-gray-900", "{room.created_at}" }
                            }
                            if let Some(updated) = &room.updated_at {
                                div {
                                    label { class: "block text-sm font-medium text-gray-500", "更新时间" }
                                    p { class: "mt-1 text-gray-900", "{updated}" }
                                }
                            }
                        }
                    }
                }
                div { class: "flex justify-end p-6 border-t bg-gray-50",
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                }
            }
        }
    }
}
