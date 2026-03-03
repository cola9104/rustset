use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaBuilding, FaPhone,
    FaEnvelope, FaCircleCheck, FaCircleXmark,
};
use crate::state::service_provider::ServiceProviderConfig;
use crate::app::PROVIDERS_STATE;

/// 服务商管理页面
#[component]
pub fn ServiceProviderManagement() -> Element {
    let mut search_query = use_signal(|| String::new());
    let mut status_filter = use_signal(|| String::from("all"));

    // 模态框状态
    let mut show_add_modal = use_signal(|| false);
    let mut show_edit_modal = use_signal(|| false);
    let mut show_view_modal = use_signal(|| false);
    let mut selected_provider = use_signal(|| None);

    // 统计数据
    let total_count = PROVIDERS_STATE.read().len() as i32;
    let active_count = PROVIDERS_STATE.read().iter().filter(|p| p.status == "active").count() as i32;
    let inactive_count = PROVIDERS_STATE.read().iter().filter(|p| p.status == "inactive").count() as i32;

    // 筛选逻辑
    let filtered_providers = PROVIDERS_STATE.read().iter().filter(|provider| {
        let matches_search = search_query.read().is_empty()
            || provider.provider_name.contains(search_query.read().as_str())
            || provider.short_name.contains(search_query.read().as_str())
            || provider.provider_code.contains(search_query.read().as_str());

        let matches_status = status_filter.read().as_str() == "all"
            || status_filter.read().as_str() == provider.status;

        matches_search && matches_status
    }).cloned().collect::<Vec<_>>();

    rsx! {
        div { class: "p-6",
            // 页面标题
            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold text-gray-800", "服务商管理" }
                button {
                    class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg flex items-center gap-2",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { "添加服务商" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "text-gray-500 text-sm", "总服务商" }
                    div { class: "text-2xl font-bold text-gray-800", "{total_count}" }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "text-gray-500 text-sm", "活跃服务商" }
                    div { class: "text-2xl font-bold text-green-600", "{active_count}" }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "text-gray-500 text-sm", "停用服务商" }
                    div { class: "text-2xl font-bold text-gray-400", "{inactive_count}" }
                }
            }

            // 搜索和筛选
            div { class: "bg-white rounded-lg shadow p-4 mb-4",
                div { class: "flex gap-4",
                    div { class: "flex-1 relative",
                        Icon { icon: FaMagnifyingGlass, width: 16, height: 16, class: "absolute left-3 top-3 text-gray-400" }
                        input {
                            class: "w-full pl-10 pr-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "搜索服务商名称、简称或编码...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value())
                        }
                    }
                    select {
                        class: "px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{status_filter}",
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "all", "全部状态" }
                        option { value: "active", "活跃" }
                        option { value: "inactive", "停用" }
                    }
                }
            }

            // 服务商列表
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "w-full",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "ID" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商名称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "简称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "负责人" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "联系电话" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务区域" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody {
                        if filtered_providers.is_empty() {
                            tr {
                                td { colspan: "8", class: "px-6 py-12 text-center text-gray-500",
                                    "暂无服务商数据"
                                }
                            }
                        } else {
                            for provider in filtered_providers.iter() {
                                tr { class: "hover:bg-gray-50 border-b",
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{provider.id}" }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        div { class: "flex items-center",
                                            div { class: "flex-shrink-0 h-10 w-10 bg-blue-100 rounded-full flex items-center justify-center",
                                                span { class: "text-blue-600 font-semibold", "{provider.short_name.chars().next().unwrap()}" }
                                            }
                                            div { class: "ml-4",
                                                div { class: "text-sm font-medium text-gray-900", "{provider.provider_name}" }
                                                div { class: "text-sm text-gray-500", "{provider.provider_code}" }
                                            }
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{provider.short_name}" }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{provider.contact_person}" }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{provider.contact_phone}" }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{provider.service_area}" }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        if provider.status == "active" {
                                            span { class: "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800",
                                                "活跃"
                                            }
                                        } else {
                                            span { class: "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800",
                                                "停用"
                                            }
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                        button {
                                            class: "text-blue-600 hover:text-blue-900 mr-3",
                                            onclick: {
                                                let provider = provider.clone();
                                                move |_| {
                                                    selected_provider.set(Some(provider.clone()));
                                                    show_view_modal.set(true);
                                                }
                                            },
                                            "查看"
                                        }
                                        button {
                                            class: "text-indigo-600 hover:text-indigo-900 mr-3",
                                            onclick: {
                                                let provider = provider.clone();
                                                move |_| {
                                                    selected_provider.set(Some(provider.clone()));
                                                    show_edit_modal.set(true);
                                                }
                                            },
                                            "编辑"
                                        }
                                        button {
                                            class: "text-red-600 hover:text-red-900",
                                            onclick: {
                                                let provider_id = provider.id;
                                                move |_| {
                                                    let mut providers = PROVIDERS_STATE.write();
                                                    if let Some(pos) = providers.iter().position(|p| p.id == provider_id) {
                                                        providers.remove(pos);
                                                    }
                                                }
                                            },
                                            "删除"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 添加服务商模态框
        if *show_add_modal.read() {
            AddProviderModal {
                show: show_add_modal,
                on_save: move |provider: ServiceProviderConfig| {
                    PROVIDERS_STATE.write().push(provider);
                    show_add_modal.set(false);
                },
                on_cancel: move |_| show_add_modal.set(false)
            }
        }

        // 编辑服务商模态框
        if *show_edit_modal.read() {
            if let Some(provider) = selected_provider.read().as_ref().clone() {
                EditProviderModal {
                    show: show_edit_modal,
                    provider: provider.clone(),
                    on_save: move |provider: ServiceProviderConfig| {
                        let mut providers = PROVIDERS_STATE.write();
                        if let Some(idx) = providers.iter().position(|p| p.id == provider.id) {
                            providers[idx] = provider;
                        }
                        show_edit_modal.set(false);
                    },
                    on_cancel: move |_| show_edit_modal.set(false)
                }
            }
        }

        // 查看服务商模态框
        if *show_view_modal.read() {
            if let Some(provider) = selected_provider.read().as_ref().clone() {
                ViewProviderModal {
                    show: show_view_modal,
                    provider: provider.clone(),
                    on_close: move |_| show_view_modal.set(false)
                }
            }
        }
    }
}

/// 添加服务商模态框
#[component]
fn AddProviderModal(show: Signal<bool>, on_save: EventHandler<ServiceProviderConfig>, on_cancel: EventHandler<MouseEvent>) -> Element {
    let mut provider_name = use_signal(|| String::new());
    let mut provider_code = use_signal(|| String::new());
    let mut short_name = use_signal(|| String::new());
    let mut contact_person = use_signal(|| String::new());
    let mut contact_phone = use_signal(|| String::new());
    let mut contact_email = use_signal(|| String::new());
    let mut headquarters = use_signal(|| String::new());
    let mut service_area = use_signal(|| String::from("全国"));
    let mut business_license = use_signal(|| String::new());
    let mut remarks = use_signal(|| String::new());
    let mut status = use_signal(|| String::from("active"));

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-screen overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-semibold text-gray-800", "添加服务商" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |e| on_cancel.call(e),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商名称" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "中国电信",
                                value: "{provider_name}",
                                oninput: move |e| provider_name.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商编码" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "CHINA_TELECOM",
                                value: "{provider_code}",
                                oninput: move |e| provider_code.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "简称" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "电信",
                                value: "{short_name}",
                                oninput: move |e| short_name.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "张经理",
                                value: "{contact_person}",
                                oninput: move |e| contact_person.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系电话" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "10000",
                                value: "{contact_phone}",
                                oninput: move |e| contact_phone.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系邮箱" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "telecom@example.com",
                                value: "{contact_email}",
                                oninput: move |e| contact_email.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "总部地址" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "北京市西城区金融大街35号",
                                value: "{headquarters}",
                                oninput: move |e| headquarters.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务区域" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{service_area}",
                                oninput: move |e| service_area.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "营业执照号" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "91110000XXXXXXXX",
                                value: "{business_license}",
                                oninput: move |e| business_license.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                            textarea {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                rows: "2",
                                placeholder: "服务商备注信息",
                                value: "{remarks}",
                                oninput: move |e| remarks.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{status}",
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "活跃" }
                                option { value: "inactive", "停用" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-3 p-6 border-t bg-gray-50",
                    button {
                        class: "px-4 py-2 border rounded-lg hover:bg-gray-100",
                        onclick: move |e| on_cancel.call(e),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |_| {
                            let new_provider = ServiceProviderConfig {
                                id: 0, // 会在后端生成
                                provider_name: provider_name.read().clone(),
                                provider_code: provider_code.read().clone(),
                                short_name: short_name.read().clone(),
                                logo_url: None,
                                contact_person: contact_person.read().clone(),
                                contact_phone: contact_phone.read().clone(),
                                contact_email: contact_email.read().clone(),
                                headquarters: headquarters.read().clone(),
                                service_area: service_area.read().clone(),
                                business_license: business_license.read().clone(),
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                status: status.read().clone(),
                                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                updated_at: None,
                            };
                            on_save.call(new_provider);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 编辑服务商模态框
#[component]
fn EditProviderModal(show: Signal<bool>, provider: ServiceProviderConfig, on_save: EventHandler<ServiceProviderConfig>, on_cancel: EventHandler<MouseEvent>) -> Element {
    let mut provider_name = use_signal(|| provider.provider_name.clone());
    let mut provider_code = use_signal(|| provider.provider_code.clone());
    let mut short_name = use_signal(|| provider.short_name.clone());
    let mut contact_person = use_signal(|| provider.contact_person.clone());
    let mut contact_phone = use_signal(|| provider.contact_phone.clone());
    let mut contact_email = use_signal(|| provider.contact_email.clone());
    let mut headquarters = use_signal(|| provider.headquarters.clone());
    let mut service_area = use_signal(|| provider.service_area.clone());
    let mut business_license = use_signal(|| provider.business_license.clone());
    let mut remarks = use_signal(|| provider.remarks.clone().unwrap_or_default());
    let mut status = use_signal(|| provider.status.clone());

    // 当 provider prop 变化时更新所有信号
    use_effect(move || {
        provider_name.set(provider.provider_name.clone());
        provider_code.set(provider.provider_code.clone());
        short_name.set(provider.short_name.clone());
        contact_person.set(provider.contact_person.clone());
        contact_phone.set(provider.contact_phone.clone());
        contact_email.set(provider.contact_email.clone());
        headquarters.set(provider.headquarters.clone());
        service_area.set(provider.service_area.clone());
        business_license.set(provider.business_license.clone());
        remarks.set(provider.remarks.clone().unwrap_or_default());
        status.set(provider.status.clone());
    });

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-screen overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-semibold text-gray-800", "编辑服务商" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |e| on_cancel.call(e),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商名称" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{provider_name}",
                                oninput: move |e| provider_name.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商编码" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{provider_code}",
                                oninput: move |e| provider_code.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "简称" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{short_name}",
                                oninput: move |e| short_name.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{contact_person}",
                                oninput: move |e| contact_person.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系电话" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{contact_phone}",
                                oninput: move |e| contact_phone.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "联系邮箱" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{contact_email}",
                                oninput: move |e| contact_email.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "总部地址" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{headquarters}",
                                oninput: move |e| headquarters.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务区域" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{service_area}",
                                oninput: move |e| service_area.set(e.value())
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "营业执照号" }
                            input {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{business_license}",
                                oninput: move |e| business_license.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                            textarea {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                rows: "2",
                                value: "{remarks}",
                                oninput: move |e| remarks.set(e.value())
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{status}",
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "活跃" }
                                option { value: "inactive", "停用" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-3 p-6 border-t bg-gray-50",
                    button {
                        class: "px-4 py-2 border rounded-lg hover:bg-gray-100",
                        onclick: move |e| on_cancel.call(e),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |_| {
                            let id = provider.id;
                            let updated_provider = ServiceProviderConfig {
                                id,
                                provider_name: provider_name.read().clone(),
                                provider_code: provider_code.read().clone(),
                                short_name: short_name.read().clone(),
                                logo_url: provider.logo_url.clone(),
                                contact_person: contact_person.read().clone(),
                                contact_phone: contact_phone.read().clone(),
                                contact_email: contact_email.read().clone(),
                                headquarters: headquarters.read().clone(),
                                service_area: service_area.read().clone(),
                                business_license: business_license.read().clone(),
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                status: status.read().clone(),
                                created_at: provider.created_at.clone(),
                                updated_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string()),
                            };
                            on_save.call(updated_provider);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 查看服务商模态框
#[component]
fn ViewProviderModal(show: Signal<bool>, provider: ServiceProviderConfig, on_close: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-screen overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-semibold text-gray-800", "服务商详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |e| on_close.call(e),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "flex items-center mb-6",
                        div { class: "flex-shrink-0 h-16 w-16 bg-blue-100 rounded-full flex items-center justify-center",
                            span { class: "text-2xl font-bold text-blue-600", "{provider.short_name.chars().next().unwrap()}" }
                        }
                        div { class: "ml-4",
                            h3 { class: "text-lg font-semibold text-gray-900", "{provider.provider_name}" }
                            p { class: "text-sm text-gray-500", "{provider.provider_code}" }
                        }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "简称" }
                            p { class: "mt-1 text-sm text-gray-900", "{provider.short_name}" }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "状态" }
                            p { class: "mt-1",
                                if provider.status == "active" {
                                    span { class: "inline-flex items-center text-sm text-green-600",
                                        Icon { icon: FaCircleCheck, width: 16, height: 16, class: "mr-1" }
                                        "活跃"
                                    }
                                } else {
                                    span { class: "inline-flex items-center text-sm text-gray-600",
                                        Icon { icon: FaCircleXmark, width: 16, height: 16, class: "mr-1" }
                                        "停用"
                                    }
                                }
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-500", "负责人" }
                            p { class: "mt-1 text-sm text-gray-900", "{provider.contact_person}" }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "联系电话" }
                            p { class: "mt-1 text-sm text-gray-900 flex items-center",
                                Icon { icon: FaPhone, width: 14, height: 14, class: "mr-1 text-gray-400" }
                                "{provider.contact_phone}"
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "联系邮箱" }
                            p { class: "mt-1 text-sm text-gray-900 flex items-center",
                                Icon { icon: FaEnvelope, width: 14, height: 14, class: "mr-1 text-gray-400" }
                                "{provider.contact_email}"
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-500", "总部地址" }
                            p { class: "mt-1 text-sm text-gray-900 flex items-center",
                                Icon { icon: FaBuilding, width: 14, height: 14, class: "mr-1 text-gray-400" }
                                "{provider.headquarters}"
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "服务区域" }
                            p { class: "mt-1 text-sm text-gray-900", "{provider.service_area}" }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "营业执照号" }
                            p { class: "mt-1 text-sm text-gray-900", "{provider.business_license}" }
                        }
                        if let Some(remarks) = &provider.remarks {
                            div { class: "col-span-2",
                                label { class: "block text-sm font-medium text-gray-500", "备注" }
                                p { class: "mt-1 text-sm text-gray-900", "{remarks}" }
                            }
                        }
                        div { class: "col-span-2 sm:col-span-1",
                            label { class: "block text-sm font-medium text-gray-500", "创建时间" }
                            p { class: "mt-1 text-sm text-gray-900", "{provider.created_at}" }
                        }
                        if let Some(updated) = &provider.updated_at {
                            div { class: "col-span-2 sm:col-span-1",
                                label { class: "block text-sm font-medium text-gray-500", "更新时间" }
                                p { class: "mt-1 text-sm text-gray-900", "{updated}" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end p-6 border-t bg-gray-50",
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                        onclick: move |e| on_close.call(e),
                        "关闭"
                    }
                }
            }
        }
    }
}
