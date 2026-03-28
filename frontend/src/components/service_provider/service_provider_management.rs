use super::provider_form::{FormMode, ProviderForm};
use crate::services::{
    create_service_provider, delete_service_provider, fetch_service_providers,
    update_service_provider,
};
use crate::state::service_provider::ServiceProviderConfig;
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBuilding, FaCircleCheck, FaCircleXmark, FaEnvelope, FaMagnifyingGlass, FaPhone, FaPlus,
};
use dioxus_free_icons::Icon;

/// 服务商管理页面
#[component]
pub fn ServiceProviderManagement() -> Element {
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| String::from("all"));

    // 数据和加载状态
    let providers = use_signal(Vec::<ServiceProviderConfig>::new);
    let is_loading = use_signal(|| true);

    // 模态框状态
    let mut show_add_modal = use_signal(|| false);
    let mut show_edit_modal = use_signal(|| false);
    let mut show_view_modal = use_signal(|| false);
    let mut selected_provider = use_signal(|| None);

    // 组件挂载时从API加载数据
    {
        let mut providers_clone = providers;
        let mut is_loading_clone = is_loading;
        use_effect(move || {
            spawn(async move {
                match fetch_service_providers().await {
                    Ok(data) => {
                        providers_clone.set(data);
                        is_loading_clone.set(false);
                    }
                    Err(e) => {
                        tracing::error!("加载服务商数据失败: {}", e);
                        is_loading_clone.set(false);
                    }
                }
            });
        });
    }

    // 刷新数据的函数
    let refresh_data = move || {
        let mut providers = providers;
        spawn(async move {
            match fetch_service_providers().await {
                Ok(data) => {
                    providers.set(data);
                }
                Err(e) => {
                    tracing::error!("刷新服务商数据失败: {}", e);
                }
            }
        });
    };

    // 统计数据
    let total_count = providers.read().len() as i32;
    let active_count = providers
        .read()
        .iter()
        .filter(|p| p.status == "active")
        .count() as i32;
    let inactive_count = providers
        .read()
        .iter()
        .filter(|p| p.status == "inactive")
        .count() as i32;
    let can_manage = current_auth.can_manage_operations();

    // 筛选逻辑
    let filtered_providers = providers
        .read()
        .iter()
        .filter(|provider| {
            let matches_search = search_query.read().is_empty()
                || provider
                    .provider_name
                    .contains(search_query.read().as_str())
                || provider.short_name.contains(search_query.read().as_str())
                || provider
                    .provider_code
                    .contains(search_query.read().as_str());

            let matches_status = status_filter.read().as_str() == "all"
                || status_filter.read().as_str() == provider.status;

            matches_search && matches_status
        })
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        div { class: "p-6",
            // 页面标题
            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold text-gray-800", "服务商管理" }
                if can_manage {
                    button {
                        class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg flex items-center gap-2",
                        onclick: move |_| show_add_modal.set(true),
                        Icon { icon: FaPlus, width: 16, height: 16 }
                        span { "添加服务商" }
                    }
                }
            }

            if !can_manage {
                div { class: "mb-6 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                    "当前账号仅可查看服务商信息，新增、编辑、删除操作已禁用。"
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

            // 加载状态
            if *is_loading.read() {
                div { class: "bg-white rounded-lg shadow p-12 text-center",
                    div { class: "text-gray-500", "加载数据中..." }
                }
            } else {
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
                                            if can_manage {
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
                                                        // refresh_data
                                                        move |_| {
                                                            // refresh_data
                                                            spawn(async move {
                                                                match delete_service_provider(provider_id).await {
                                                                    Ok(()) => {
                                                                        refresh_data();
                                                                    }
                                                                    Err(e) => {
                                                                        tracing::error!("删除服务商失败: {}", e);
                                                                    }
                                                                }
                                                            });
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
            }
        }

        // 添加服务商模态框
        if can_manage && *show_add_modal.read() {
            ProviderForm {
                mode: FormMode::New,
                provider: None,
                on_save: move |provider: ServiceProviderConfig| {
                    // refresh_data
                    spawn(async move {
                        match create_service_provider(&provider).await {
                            Ok(_) => {
                                refresh_data();
                            }
                            Err(e) => {
                                tracing::error!("创建服务商失败: {}", e);
                            }
                        }
                    });
                    show_add_modal.set(false);
                },
                on_close: move |_| show_add_modal.set(false),
            }
        }

        // 编辑服务商模态框
        if can_manage && *show_edit_modal.read() {
            if let Some(provider) = selected_provider.read().as_ref().cloned() {
                ProviderForm {
                    key: "provider-edit-{provider.id}",
                    mode: FormMode::Edit,
                    provider: Some(provider.clone()),
                    on_save: move |provider: ServiceProviderConfig| {
                        // refresh_data
                        spawn(async move {
                            match update_service_provider(provider.id, &provider).await {
                                Ok(_) => {
                                    refresh_data();
                                }
                                Err(e) => {
                                    tracing::error!("更新服务商失败: {}", e);
                                }
                            }
                        });
                        show_edit_modal.set(false);
                    },
                    on_close: move |_| show_edit_modal.set(false),
                }
            }
        }

        // 查看服务商模态框
        if *show_view_modal.read() {
            if let Some(provider) = selected_provider.read().as_ref().cloned() {
                ViewProviderModal {
                    provider: provider.clone(),
                    on_close: move |_| show_view_modal.set(false)
                }
            }
        }
    }
}

/// 查看服务商模态框
#[component]
fn ViewProviderModal(provider: ServiceProviderConfig, on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-screen overflow-y-auto",
                div { class: "flex justify-between items-center p-6 border-b",
                    h2 { class: "text-xl font-semibold text-gray-800", "服务商详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
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
                                "{provider.headquarters}"                            }
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
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                }
            }
        }
    }
}
