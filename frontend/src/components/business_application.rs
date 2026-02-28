// ============== Business Application Component (云资源申请) ==============

use crate::{api_url, Language};
use gloo_net::http::Request;
use shared::{
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
    CloudZone, CloudPlatform, CloudProviderConfig, CloudProvider,
    CloudServiceAsset, CloudServiceAssetStats,
    CreatePhysicalMachineInfo, CreateCloudVirtualMachineInfo,
    PhysicalMachineInfo, CloudVirtualMachineInfo,
    UpdatePhysicalMachineInfo, UpdateCloudVirtualMachineInfo,
    User, Role, LoginResponse,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlSelectElement, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use gloo_timers::callback::Timeout;
use serde_json;

// Helper function to get token from localStorage
fn get_auth_token() -> String {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_token") {
                Ok(token) => token.unwrap_or_default().trim().to_string(),
                _ => String::new(),
            },
            _ => String::new(),
        },
        _ => String::new(),
    }
}

#[function_component]
pub fn BusinessApplication() -> Html {
    let lang = use_state(|| Language::Zh);
    let resources = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Active tab state: "all", "cloud", "physical"
    let active_tab = use_state(|| "all".to_string());

    // Active cloud provider configs (from cloud-provider-configs/active API)
    let active_configs = use_state(|| Vec::<CloudProviderConfig>::new());

    // Cloud zones and platforms for selection
    let cloud_zones = use_state(|| Vec::<CloudZone>::new());
    let cloud_platforms = use_state(|| Vec::<CloudPlatform>::new());
    let selected_zone_id = use_state(|| None as Option<i32>);
    let selected_platform_id = use_state(|| None as Option<i32>);

    // Form state for adding/editing
    let show_form = use_state(|| false);
    let editing_id = use_state(|| None as Option<i32>);
    let form_data = use_state(|| CreateBusinessResourceRequest {
        resource_type: "cloud".to_string(),
        ecs_name: String::new(),
        ecs_status: "待交付".to_string(),
        resource_id: None,
        cloud_region: String::new(),
        cloud_category: String::new(),
        cloud_provider_config_id: None,
        zone_name: None,
        platform_name: None,
        county_city: None,
        vdc_name: None,
        customer_name: String::new(),
        application_name: None,
        contract_name: None,
        instance_id: None,
        ecs_type: String::new(),
        ecs_os: String::new(),
        cpu_cores: 2,
        memory_gb: 4,
        system_disk: "ESSD".to_string(),
        system_disk_size_gb: 40,
        data_disk: None,
        completion_time: None,
        release_time: None,
        has_security_product: false,
        ip_address: String::new(),
        ecs_login_method: None,
        ecs_login_username: None,
        ecs_initial_password: None,
        bastion_address: None,
        bastion_admin_account: None,
        bastion_initial_password: None,
        physical_machine_info: None,
        cloud_vm_info: None,
        remarks: None,
        application_status: Some("待审核".to_string()),
        delivery_status: Some("待交付".to_string()),
        delivery_confirmed_at: None,
        delivery_confirmed_by: None,
    });

    // Get token from localStorage
    let token = get_auth_token();

    // Fetch active cloud provider configs
    let fetch_active_configs = {
        let active_configs = active_configs.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let active_configs = active_configs.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("cloud-provider-configs/active"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudProviderConfig>>().await {
                        active_configs.set(data);
                    }
                }
            });
        })
    };

    // Fetch cloud zones
    let fetch_cloud_zones = {
        let cloud_zones = cloud_zones.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let cloud_zones = cloud_zones.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("providers"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudZone>>().await {
                        cloud_zones.set(data);
                    }
                }
            });
        })
    };

    // Fetch cloud platforms by zone
    let fetch_cloud_platforms = {
        let cloud_platforms = cloud_platforms.clone();
        let token = token.clone();

        Callback::from(move |zone_id: i32| {
            let cloud_platforms = cloud_platforms.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&format!("{}cloud-services/provider/{}", api_url(""), zone_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudPlatform>>().await {
                        cloud_platforms.set(data);
                    }
                }
            });
        })
    };

    // Fetch business resources (filter by status: 待审批, 审批中)
    let fetch_resources = {
        let resources = resources.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let resources = resources.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<BusinessResource>>().await {
                        // Show all resources
                        resources.set(data);
                    }
                }
                loading.set(false);
            });
        })
    };

    // Use effect to fetch data on mount
    use_effect_with((), {
        let fetch_resources = fetch_resources.clone();
        let fetch_active_configs = fetch_active_configs.clone();
        let fetch_cloud_zones = fetch_cloud_zones.clone();
        move |_| {
            fetch_active_configs.emit(());
            fetch_resources.emit(());
            fetch_cloud_zones.emit(());
            || ()
        }
    });

    // Status badge class
    let status_class = |status: &str| -> &'static str {
        match status {
            "待审批" => "is-warning",
            "审批中" => "is-info",
            "已驳回" => "is-danger",
            "已通过" => "is-success",
            _ => "is-light",
        }
    };

    // Helper function to get provider name in Chinese
    let provider_name = |provider: &CloudProvider| -> String {
        match provider {
            CloudProvider::Aliyun => "阿里云".to_string(),
            CloudProvider::Tencent => "腾讯云".to_string(),
            CloudProvider::Huawei => "华为云".to_string(),
            CloudProvider::Aws => "AWS".to_string(),
            CloudProvider::Azure => "Azure".to_string(),
            CloudProvider::Gcp => "GCP".to_string(),
            CloudProvider::Baidu => "百度云".to_string(),
            CloudProvider::Custom(s) => s.clone(),
        }
    };

    // Handle form submission
    let on_submit = {
        let form_data = form_data.clone();
        let editing_id = editing_id.clone();
        let show_form = show_form.clone();
        let token = token.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |_| {
            let data = form_data.clone();
            let edit_id = *editing_id;
            let token = token.clone();
            let fetch_resources = fetch_resources.clone();
            let show_form = show_form.clone();

            spawn_local(async move {
                let result = if let Some(id) = edit_id {
                    // For update, we need to convert to UpdateRequest
                    let update_req = UpdateBusinessResourceRequest {
                        resource_type: Some(data.resource_type.clone()),
                        ecs_name: Some(data.ecs_name.clone()),
                        ecs_status: Some(data.ecs_status.clone()),
                        resource_id: data.resource_id.clone(),
                        cloud_region: Some(data.cloud_region.clone()),
                        cloud_category: Some(data.cloud_category.clone()),
                        cloud_provider_config_id: data.cloud_provider_config_id,
                        zone_name: data.zone_name.clone(),
                        platform_name: data.platform_name.clone(),
                        county_city: data.county_city.clone(),
                        vdc_name: data.vdc_name.clone(),
                        customer_name: Some(data.customer_name.clone()),
                        application_name: data.application_name.clone(),
                        contract_name: data.contract_name.clone(),
                        instance_id: data.instance_id.clone(),
                        ecs_type: Some(data.ecs_type.clone()),
                        ecs_os: Some(data.ecs_os.clone()),
                        cpu_cores: Some(data.cpu_cores),
                        memory_gb: Some(data.memory_gb),
                        system_disk: Some(data.system_disk.clone()),
                        system_disk_size_gb: Some(data.system_disk_size_gb),
                        data_disk: data.data_disk.clone(),
                        completion_time: data.completion_time,
                        release_time: data.release_time,
                        has_security_product: Some(data.has_security_product),
                        ip_address: Some(data.ip_address.clone()),
                        ecs_login_method: data.ecs_login_method.clone(),
                        ecs_login_username: data.ecs_login_username.clone(),
                        ecs_initial_password: data.ecs_initial_password.clone(),
                        bastion_address: data.bastion_address.clone(),
                        bastion_admin_account: data.bastion_admin_account.clone(),
                        bastion_initial_password: data.bastion_initial_password.clone(),
                        // 物理机特有字段 - 使用嵌套结构
                        physical_machine_info: data.physical_machine_info.as_ref().map(|info| {
                            shared::UpdatePhysicalMachineInfo {
                                serial_number: info.serial_number.clone(),
                                rack_location: info.rack_location.clone(),
                                hardware_model: info.hardware_model.clone(),
                                warranty_expiry: info.warranty_expiry,
                                agent_status: info.agent_status.clone(),
                                ipmi_address: info.ipmi_address.clone(),
                            }
                        }),
                        // 云虚拟机特有字段 - 使用嵌套结构
                        cloud_vm_info: data.cloud_vm_info.as_ref().map(|info| {
                            shared::UpdateCloudVirtualMachineInfo {
                                billing_mode: info.billing_mode.clone(),
                                expire_time: info.expire_time,
                                charge_type: info.charge_type.clone(),
                                instance_charge_type: info.instance_charge_type.clone(),
                                internet_charge_type: info.internet_charge_type.clone(),
                                internet_max_bandwidth_out: info.internet_max_bandwidth_out,
                                image_id: info.image_id.clone(),
                                v_switch_id: info.v_switch_id.clone(),
                                vpc_id: info.vpc_id.clone(),
                                security_group_ids: info.security_group_ids.clone(),
                            }
                        }),
                        remarks: data.remarks.clone(),
                        application_status: data.application_status.clone(),
                        delivery_status: data.delivery_status.clone(),
                        delivery_confirmed_at: data.delivery_confirmed_at,
                        delivery_confirmed_by: data.delivery_confirmed_by.clone(),
                    };
                    let url = format!("{}/{}", api_url("business-resources"), id);
                    let json_body = serde_json::to_string(&update_req).unwrap_or_default();
                    Request::put(&url)
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(json_body)
                        .unwrap()
                        .send()
                        .await
                } else {
                    let json_body = serde_json::to_string(&*data).unwrap_or_default();
                    Request::post(&api_url("business-resources"))
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(json_body)
                        .unwrap()
                        .send()
                        .await
                };

                if result.is_ok() {
                    show_form.set(false);
                    fetch_resources.emit(());
                }
            });
        })
    };

    // Handle edit
    let on_edit = {
        let editing_id = editing_id.clone();
        let show_form = show_form.clone();
        let form_data = form_data.clone();

        Callback::from(move |resource: BusinessResource| {
            // Convert PhysicalMachineInfo to CreatePhysicalMachineInfo for editing
            let physical_machine_info = resource.physical_machine_info.as_ref().map(|pm| {
                shared::CreatePhysicalMachineInfo {
                    serial_number: pm.serial_number.clone(),
                    rack_location: pm.rack_location.clone(),
                    hardware_model: pm.hardware_model.clone(),
                    warranty_expiry: pm.warranty_expiry,
                    agent_status: pm.agent_status.clone(),
                    ipmi_address: pm.ipmi_address.clone(),
                }
            });

            // Convert CloudVirtualMachineInfo to CreateCloudVirtualMachineInfo for editing
            let cloud_vm_info = resource.cloud_vm_info.as_ref().map(|cvm| {
                shared::CreateCloudVirtualMachineInfo {
                    billing_mode: cvm.billing_mode.clone(),
                    expire_time: cvm.expire_time,
                    charge_type: cvm.charge_type.clone(),
                    instance_charge_type: cvm.instance_charge_type.clone(),
                    internet_charge_type: cvm.internet_charge_type.clone(),
                    internet_max_bandwidth_out: cvm.internet_max_bandwidth_out,
                    image_id: cvm.image_id.clone(),
                    v_switch_id: cvm.v_switch_id.clone(),
                    vpc_id: cvm.vpc_id.clone(),
                    security_group_ids: cvm.security_group_ids.clone(),
                }
            });

            let edit_data = CreateBusinessResourceRequest {
                resource_type: resource.resource_type.clone(),
                ecs_name: resource.ecs_name.clone(),
                ecs_status: resource.ecs_status.clone(),
                resource_id: Some(resource.resource_id.clone()),
                cloud_region: resource.cloud_region.clone(),
                cloud_category: resource.cloud_category.clone(),
                cloud_provider_config_id: resource.cloud_provider_config_id,
                zone_name: resource.zone_name.clone(),
                platform_name: resource.platform_name.clone(),
                county_city: resource.county_city.clone(),
                vdc_name: resource.vdc_name.clone(),
                customer_name: resource.customer_name.clone(),
                application_name: resource.application_name.clone(),
                contract_name: resource.contract_name.clone(),
                instance_id: Some(resource.instance_id.clone()),
                ecs_type: resource.ecs_type.clone(),
                ecs_os: resource.ecs_os.clone(),
                cpu_cores: resource.cpu_cores,
                memory_gb: resource.memory_gb,
                system_disk: resource.system_disk.clone(),
                system_disk_size_gb: resource.system_disk_size_gb,
                data_disk: resource.data_disk.clone(),
                completion_time: resource.completion_time,
                release_time: resource.release_time,
                has_security_product: resource.has_security_product,
                ip_address: resource.ip_address.clone(),
                ecs_login_method: resource.ecs_login_method.clone(),
                ecs_login_username: resource.ecs_login_username.clone(),
                ecs_initial_password: resource.ecs_initial_password.clone(),
                bastion_address: resource.bastion_address.clone(),
                bastion_admin_account: resource.bastion_admin_account.clone(),
                bastion_initial_password: resource.bastion_initial_password.clone(),
                physical_machine_info,
                cloud_vm_info,
                remarks: resource.remarks.clone(),
                application_status: resource.application_status.clone(),
                delivery_status: resource.delivery_status.clone(),
                delivery_confirmed_at: resource.delivery_confirmed_at,
                delivery_confirmed_by: resource.delivery_confirmed_by.clone(),
            };

            form_data.set(edit_data);
            editing_id.set(resource.id);
            show_form.set(true);
        })
    };

    // Handle delete
    let on_delete = {
        let token = token.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |id: i32| {
            let token = token.clone();
            let fetch_resources = fetch_resources.clone();

            spawn_local(async move {
                let url = format!("{}/{}", api_url("business-resources"), id);
                if Request::delete(&url)
                    .header("Authorization", &token)
                    .send()
                    .await
                    .is_ok()
                {
                    fetch_resources.emit(());
                }
            });
        })
    };

    // Reset form
    let reset_form = {
        let form_data = form_data.clone();
        let editing_id = editing_id.clone();
        let show_form = show_form.clone();
        let selected_zone_id = selected_zone_id.clone();
        let selected_platform_id = selected_platform_id.clone();

        Callback::from(move |_| {
            form_data.set(CreateBusinessResourceRequest {
                resource_type: "cloud".to_string(),
                ecs_name: String::new(),
                ecs_status: "待交付".to_string(),
                resource_id: None,
                cloud_region: String::new(),
                cloud_category: String::new(),
                cloud_provider_config_id: None,
                zone_name: None,
                platform_name: None,
                county_city: None,
                vdc_name: None,
                customer_name: String::new(),
                application_name: None,
                contract_name: None,
                instance_id: None,
                ecs_type: String::new(),
                ecs_os: String::new(),
                cpu_cores: 2,
                memory_gb: 4,
                system_disk: "ESSD".to_string(),
                system_disk_size_gb: 40,
                data_disk: None,
                completion_time: None,
                release_time: None,
                has_security_product: false,
                ip_address: String::new(),
                ecs_login_method: None,
                ecs_login_username: None,
                ecs_initial_password: None,
                bastion_address: None,
                bastion_admin_account: None,
                bastion_initial_password: None,
                physical_machine_info: None,
                cloud_vm_info: None,
                remarks: None,
                application_status: Some("待审核".to_string()),
                delivery_status: Some("待交付".to_string()),
                delivery_confirmed_at: None,
                delivery_confirmed_by: None,
            });
            editing_id.set(None);
            selected_zone_id.set(None);
            selected_platform_id.set(None);
            show_form.set(false);
        })
    };

    // Handle input change
    let on_input_change = {
        let form_data = form_data.clone();

        Callback::from(move |(field, value): (String, String)| {
            let mut data = (*form_data).clone();

            match field.as_str() {
                "ecs_name" => data.ecs_name = value,
                "ecs_status" => data.ecs_status = value,
                "resource_id" => data.resource_id = if value.is_empty() { None } else { Some(value) },
                "resource_type" => data.resource_type = value,
                "cloud_region" => data.cloud_region = value,
                "cloud_category" => data.cloud_category = value,
                "county_city" => data.county_city = if value.is_empty() { None } else { Some(value) },
                "vdc_name" => data.vdc_name = if value.is_empty() { None } else { Some(value) },
                "customer_name" => data.customer_name = value,
                "application_name" => data.application_name = if value.is_empty() { None } else { Some(value) },
                "contract_name" => data.contract_name = if value.is_empty() { None } else { Some(value) },
                "instance_id" => data.instance_id = if value.is_empty() { None } else { Some(value) },
                "ecs_type" => data.ecs_type = value,
                "ecs_os" => data.ecs_os = value,
                "system_disk" => data.system_disk = value,
                "data_disk" => data.data_disk = if value.is_empty() { None } else { Some(value) },
                "ip_address" => data.ip_address = value,
                "ecs_login_method" => data.ecs_login_method = if value.is_empty() { None } else { Some(value) },
                "ecs_login_username" => data.ecs_login_username = if value.is_empty() { None } else { Some(value) },
                "ecs_initial_password" => data.ecs_initial_password = if value.is_empty() { None } else { Some(value) },
                "bastion_address" => data.bastion_address = if value.is_empty() { None } else { Some(value) },
                "bastion_admin_account" => data.bastion_admin_account = if value.is_empty() { None } else { Some(value) },
                "bastion_initial_password" => data.bastion_initial_password = if value.is_empty() { None } else { Some(value) },
                "remarks" => data.remarks = if value.is_empty() { None } else { Some(value) },
                _ => {}
            }

            form_data.set(data);
        })
    };

    // Handle numeric input
    let on_number_input = {
        let form_data = form_data.clone();

        Callback::from(move |(field, value): (String, u32)| {
            let mut data = (*form_data).clone();

            match field.as_str() {
                "cpu_cores" => data.cpu_cores = value,
                "memory_gb" => data.memory_gb = value,
                "system_disk_size_gb" => data.system_disk_size_gb = value,
                _ => {}
            }

            form_data.set(data);
        })
    };

    // Handle checkbox
    let on_checkbox_change = {
        let form_data = form_data.clone();

        Callback::from(move |value: bool| {
            let mut data = (*form_data).clone();
            data.has_security_product = value;
            form_data.set(data);
        })
    };

    // Handle cloud provider config selection
    let on_config_select = {
        let form_data = form_data.clone();
        let active_configs = active_configs.clone();
        let provider_name_for_callback = provider_name;

        Callback::from(move |config_id_str: String| {
            let mut data = (*form_data).clone();

            if config_id_str.is_empty() {
                // No selection
                data.cloud_provider_config_id = None;
                // 不清空cloud_category，因为用户可能已经选择了云类别
                data.cloud_region = String::new();
            } else if let Ok(config_id) = config_id_str.parse::<i32>() {
                // Find the selected config
                data.cloud_provider_config_id = Some(config_id);
                if let Some(config) = active_configs.iter().find(|c| c.id == Some(config_id)) {
                    // Auto-fill cloud_category (使用中文名称) and cloud_region from the selected config
                    data.cloud_category = provider_name_for_callback(&config.provider);
                    data.cloud_region = config.region_name.clone();
                    // 云资源类型
                    data.resource_type = "cloud".to_string();
                }
            }

            form_data.set(data);
        })
    };

    // 根据选中的标签过滤资源列表 - 在 html! 宏之外计算
    let filtered_resources: Vec<&BusinessResource> = (*resources).iter().filter(|resource| {
        match (*active_tab).as_str() {
            "all" => true,
            "cloud" => resource.resource_type == "cloud",
            "physical" => resource.resource_type == "physical",
            _ => true,
        }
    }).collect();

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("business_application") }</h1>

            <div class="box">
                <div class="level">
                    <div class="level-left">
                        <button class="button is-primary" onclick={
                            let show_form = show_form.clone();
                            let selected_zone_id = selected_zone_id.clone();
                            let selected_platform_id = selected_platform_id.clone();
                            let active_tab = active_tab.clone();
                            let form_data = form_data.clone();
                            Callback::from(move |_| {
                                show_form.set(true);
                                // 打开新表单时重置云区和云服务选择
                                selected_zone_id.set(None);
                                selected_platform_id.set(None);

                                // 根据当前选中的标签预设资源类型
                                let mut data = (*form_data).clone();
                                match (*active_tab).as_str() {
                                    "cloud" => {
                                        data.resource_type = "cloud".to_string();
                                        data.cloud_vm_info = Some(shared::CreateCloudVirtualMachineInfo {
                                            billing_mode: None,
                                            expire_time: None,
                                            charge_type: None,
                                            instance_charge_type: None,
                                            internet_charge_type: None,
                                            internet_max_bandwidth_out: None,
                                            image_id: None,
                                            v_switch_id: None,
                                            vpc_id: None,
                                            security_group_ids: None,
                                        });
                                        data.physical_machine_info = None;
                                    }
                                    "physical" => {
                                        data.resource_type = "physical".to_string();
                                        data.physical_machine_info = Some(shared::CreatePhysicalMachineInfo {
                                            serial_number: None,
                                            rack_location: None,
                                            hardware_model: None,
                                            warranty_expiry: None,
                                            agent_status: None,
                                            ipmi_address: None,
                                        });
                                        data.cloud_vm_info = None;
                                    }
                                    _ => {
                                        // "all" tab - don't pre-select
                                    }
                                }
                                form_data.set(data);
                            })
                        }>
                            <span class="icon"><i class="fas fa-plus"></i></span>
                            <span>{ lang.t("add_business_resource") }</span>
                        </button>
                        <button class="button is-info ml-2" onclick={
                            let token = token.clone();
                            Callback::from(move |_| {
                                let token = token.clone();
                                spawn_local(async move {
                                    let _ = Request::get(&api_url("business-resources/export"))
                                        .header("Authorization", &token)
                                        .send()
                                        .await;
                                });
                            })
                        }>
                            <span class="icon"><i class="fas fa-download"></i></span>
                            <span>{ lang.t("export") }</span>
                        </button>
                    </div>
                </div>

                // 标签页：全部 | 云服务器 | 物理机
                <div class="tabs is-boxed mb-4">
                    <ul>
                        <li class={if (*active_tab) == "all" { "is-active" } else { "" }}>
                            <a onclick={
                                let active_tab = active_tab.clone();
                                Callback::from(move |_| {
                                    active_tab.set("all".to_string());
                                })
                            }>
                                { "全部" }
                            </a>
                        </li>
                        <li class={if (*active_tab) == "cloud" { "is-active" } else { "" }}>
                            <a onclick={
                                let active_tab = active_tab.clone();
                                Callback::from(move |_| {
                                    active_tab.set("cloud".to_string());
                                })
                            }>
                                { "云服务器" }
                            </a>
                        </li>
                        <li class={if (*active_tab) == "physical" { "is-active" } else { "" }}>
                            <a onclick={
                                let active_tab = active_tab.clone();
                                Callback::from(move |_| {
                                    active_tab.set("physical".to_string());
                                })
                            }>
                                { "物理机" }
                            </a>
                        </li>
                    </ul>
                </div>

                if *show_form {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={reset_form.clone()}></div>
                        <div class="modal-card" style="width: 800px;">
                            <header class="modal-card-head">
                                <p class="modal-card-title">
                                    { if (*editing_id).is_some() { lang.t("edit") } else { lang.t("add_business_resource") } }
                                </p>
                                <button class="delete" onclick={reset_form.clone()}></button>
                            </header>
                            <section class="modal-card-body" style="max-height: calc(100vh - 200px); overflow-y: auto;">
                                <div class="columns is-multiline">
                                    // Basic Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_name.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_status") }</label>
                                        <div class="select is-fullwidth">
                                            <select
                                                onchange={
                                                    let on_input_change = on_input_change.clone();
                                                    Callback::from(move |e: Event| {
                                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                                        on_input_change.emit(("ecs_status".to_string(), select.value()));
                                                    })
                                                }
                                            >
                                                <option selected={(*form_data).ecs_status == "待交付"}>{ "待交付" }</option>
                                                <option selected={(*form_data).ecs_status == "运行中"}>{ "运行中" }</option>
                                                <option selected={(*form_data).ecs_status == "已停止"}>{ "已停止" }</option>
                                                <option selected={(*form_data).ecs_status == "已释放"}>{ "已释放" }</option>
                                            </select>
                                        </div>
                                    </div>

                                    // Cloud Zone Selection - 资源类型选择
                                    <div class="column is-6">
                                        <label class="label">{ "资源类型" }</label>
                                        <div class="select is-fullwidth">
                                            <select
                                                value={(*form_data).resource_type.clone()}
                                                onchange={
                                                    let on_input_change = on_input_change.clone();
                                                    let form_data = form_data.clone();
                                                    let selected_zone_id = selected_zone_id.clone();
                                                    let selected_platform_id = selected_platform_id.clone();
                                                    Callback::from(move |e: Event| {
                                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                                        let value = select.value();
                                                        let mut data = (*form_data).clone();
                                                        data.resource_type = value.clone();
                                                        // 切换资源类型时重置相关字段
                                                        if value == "physical" {
                                                            data.cloud_provider_config_id = None;
                                                            data.cloud_category = "物理机".to_string();
                                                            // 切换到物理机时，清除云区和云服务选择
                                                            selected_zone_id.set(None);
                                                            selected_platform_id.set(None);
                                                        } else {
                                                            data.cloud_category = String::new();
                                                        }
                                                        form_data.set(data);
                                                        on_input_change.emit(("resource_type".to_string(), value));
                                                    })
                                                }
                                            >
                                                <option value="cloud" selected={(*form_data).resource_type == "cloud"}>{ "云资源" }</option>
                                                <option value="physical" selected={(*form_data).resource_type == "physical"}>{ "物理机" }</option>
                                            </select>
                                        </div>
                                        <p class="help">
                                            { "选择资源类型" }
                                        </p>
                                    </div>

                                    // 运营商/厂家选择 - 云资源和物理机都需要选择
                                    <div class="column is-6">
                                        <label class="label">{ "所属运营商/厂家" }</label>
                                        <div class="select is-fullwidth">
                                            <select
                                                onchange={
                                                    let on_input_change = on_input_change.clone();
                                                    let selected_zone_id = selected_zone_id.clone();
                                                    let selected_platform_id = selected_platform_id.clone();
                                                    let fetch_cloud_platforms = fetch_cloud_platforms.clone();
                                                    let form_data = form_data.clone();
                                                    Callback::from(move |e: Event| {
                                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                                        let zone_id: i32 = select.value().parse().unwrap_or(0);
                                                        if zone_id > 0 {
                                                            selected_zone_id.set(Some(zone_id));
                                                            // 重置云服务选择
                                                            selected_platform_id.set(None);
                                                            // 清除 cloud_provider_config_id
                                                            let mut data = (*form_data).clone();
                                                            data.cloud_provider_config_id = None;
                                                            form_data.set(data);
                                                            // 获取该云区的云服务列表
                                                            fetch_cloud_platforms.emit(zone_id);
                                                        } else {
                                                            selected_zone_id.set(None);
                                                            selected_platform_id.set(None);
                                                            // 清除 cloud_provider_config_id
                                                            let mut data = (*form_data).clone();
                                                            data.cloud_provider_config_id = None;
                                                            form_data.set(data);
                                                        }
                                                    })
                                                }
                                            >
                                                <option value="" selected={(*selected_zone_id).is_none()}>{ "选择运营商/厂家..." }</option>
                                                { for (*cloud_zones).iter().map(|zone| {
                                                    let zone_id_value = zone.id.unwrap_or(0);
                                                    let selected = (*selected_zone_id).map(|z| z == zone_id_value).unwrap_or(false);
                                                    html! {
                                                        <option value={zone_id_value.to_string()} {selected}>
                                                            { &zone.zone_name }
                                                        </option>
                                                    }
                                                })}
                                            </select>
                                        </div>
                                        <p class="help">
                                            { if (*form_data).resource_type == "physical" { "选择物理机所在运营商/厂家" } else { "选择系统定义的运营商/厂家" } }
                                        </p>
                                    </div>

                                    // 云服务选择 - 选择云区后显示（云资源和物理机都需要）
                                    if (*selected_zone_id).is_some() {
                                        <div class="column is-6">
                                            <label class="label">{ "云服务" }</label>
                                            <div class="select is-fullwidth">
                                                <select
                                                    onchange={
                                                        let selected_platform_id = selected_platform_id.clone();
                                                        let selected_zone_id = selected_zone_id.clone();
                                                        let active_configs = active_configs.clone();
                                                        let form_data = form_data.clone();
                                                        let cloud_zones = cloud_zones.clone();
                                                        let cloud_platforms = cloud_platforms.clone();
                                                        Callback::from(move |e: Event| {
                                                            let select: HtmlSelectElement = e.target_unchecked_into();
                                                            let platform_id: i32 = select.value().parse().unwrap_or(0);
                                                            if platform_id > 0 {
                                                                selected_platform_id.set(Some(platform_id));
                                                                // 根据云区和云服务找到对应的云厂商配置ID
                                                                if let Some(zone_id) = *selected_zone_id {
                                                                    let matching_config = (*active_configs).iter()
                                                                        .find(|c| c.zone_id == Some(zone_id) && c.platform_id == Some(platform_id));
                                                                    let config_id = matching_config.and_then(|c| c.id);
                                                                    // 更新 form_data 中的相关字段
                                                                    let mut data = (*form_data).clone();
                                                                    data.cloud_provider_config_id = config_id;
                                                                    // 同时更新 cloud_category 和 cloud_region
                                                                    if let Some(config) = matching_config {
                                                                        data.cloud_category = format!("{:?}", config.provider);
                                                                        data.cloud_region = config.region_name.clone();
                                                                    }
                                                                    // 获取云区和云服务名称
                                                                    let zone_name = (*cloud_zones).iter()
                                                                        .find(|z| z.id == Some(zone_id))
                                                                        .map(|z| z.zone_name.clone());
                                                                    let platform_name = (*cloud_platforms).iter()
                                                                        .find(|p| p.id == Some(platform_id))
                                                                        .map(|p| p.platform_name.clone());
                                                                    data.zone_name = zone_name;
                                                                    data.platform_name = platform_name;
                                                                    form_data.set(data);
                                                                }
                                                            } else {
                                                                selected_platform_id.set(None);
                                                                // 清除相关字段
                                                                let mut data = (*form_data).clone();
                                                                data.cloud_provider_config_id = None;
                                                                data.zone_name = None;
                                                                data.platform_name = None;
                                                                form_data.set(data);
                                                            }
                                                        })
                                                    }
                                                >
                                                    <option value="" selected={(*selected_platform_id).is_none()}>{ "选择云服务..." }</option>
                                                    { for (*cloud_platforms).iter().map(|platform| {
                                                        let platform_id_value = platform.id.unwrap_or(0);
                                                        let selected = (*selected_platform_id).map(|p| p == platform_id_value).unwrap_or(false);
                                                        html! {
                                                            <option value={platform_id_value.to_string()} {selected}>
                                                                { &platform.platform_name }
                                                            </option>
                                                        }
                                                    })}
                                                </select>
                                            </div>
                                            <p class="help">
                                                { if (*form_data).resource_type == "physical" { "选择物理机所在机房/环境" } else { "选择系统定义的云服务" } }
                                            </p>
                                        </div>
                                    } else {
                                        <div class="column is-6">
                                            <label class="label">{ "云服务" }</label>
                                            <div class="select is-fullwidth">
                                                <select disabled={true}>
                                                    <option>{ "请先选择运营商/厂家" }</option>
                                                </select>
                                            </div>
                                        </div>
                                    }

                                    // 云平台（技术底座）选择 - 只有选择云资源且已选择运营商、云服务时才显示
                                    if (*form_data).resource_type == "cloud" && (*selected_platform_id).is_some() {
                                        <div class="column is-12">
                                            <label class="label">{ "云平台（技术底座）" }</label>
                                            <div class="select is-fullwidth">
                                                <select
                                                    onchange={
                                                        let on_config_select = on_config_select.clone();
                                                        Callback::from(move |e: Event| {
                                                            let select: HtmlSelectElement = e.target_unchecked_into();
                                                            on_config_select.emit(select.value());
                                                        })
                                                    }
                                                >
                                                    <option value="">{ "选择技术底座..." }</option>
                                                    {
                                                        (*active_configs).iter().filter(|c| {
                                                            // 过滤条件：运营商和云服务匹配
                                                            let zone_match = (*selected_zone_id).and_then(|z| c.zone_id.map(|cz| z == cz)).unwrap_or(false);
                                                            let platform_match = (*selected_platform_id).and_then(|p| c.platform_id.map(|cp| p == cp)).unwrap_or(false);
                                                            zone_match && platform_match
                                                        }).map(|config| {
                                                            let selected = (*form_data).cloud_provider_config_id == config.id;
                                                            let provider_name = provider_name(&config.provider);
                                                            let label = format!("[{}] {} - {}", provider_name, config.account_name, config.region_name);
                                                            html! {
                                                                <option
                                                                    value={config.id.unwrap_or(0).to_string()}
                                                                    selected={selected}
                                                                >
                                                                    { label }
                                                                </option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>
                                            <p class="help">
                                                { "选择该云服务的技术底座（如阿里云、腾讯云等）" }
                                            </p>
                                        </div>
                                    } else if (*form_data).resource_type == "cloud" {
                                        <div class="column is-12">
                                            <label class="label">{ "云平台（技术底座）" }</label>
                                            <div class="select is-fullwidth">
                                                <select disabled={true}>
                                                    <option>{ if (*selected_platform_id).is_none() { "请先选择云服务" } else { "暂无可用配置" } }</option>
                                                </select>
                                            </div>
                                        </div>
                                    } else {
                                        <div class="column is-12"></div>
                                    }

                                    <div class="column is-6">
                                        <label class="label">{ lang.t("county_city") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).county_city.clone().unwrap_or_default()}
                                            placeholder={ if (*form_data).cloud_category == "物理机" { "如：杭州机房" } else { "杭州市" } }
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("county_city".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Customer Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("customer_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).customer_name.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("customer_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("application_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).application_name.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("application_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("contract_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).contract_name.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("contract_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // ECS Specs
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_type") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_type.clone()}
                                            placeholder="ecs.g6.large"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_type".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_os") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_os.clone()}
                                            placeholder="CentOS 7.9"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_os".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-4">
                                        <label class="label">{ lang.t("cpu_cores") }</label>
                                        <input
                                            type="number"
                                            class="input"
                                            value={(*form_data).cpu_cores.to_string()}
                                            onchange={
                                                let on_number_input = on_number_input.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(val) = input.value().parse::<u32>() {
                                                        on_number_input.emit(("cpu_cores".to_string(), val));
                                                    }
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-4">
                                        <label class="label">{ lang.t("memory_gb") }</label>
                                        <input
                                            type="number"
                                            class="input"
                                            value={(*form_data).memory_gb.to_string()}
                                            onchange={
                                                let on_number_input = on_number_input.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(val) = input.value().parse::<u32>() {
                                                        on_number_input.emit(("memory_gb".to_string(), val));
                                                    }
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-4">
                                        <label class="label">{ lang.t("system_disk_size_gb") }</label>
                                        <input
                                            type="number"
                                            class="input"
                                            value={(*form_data).system_disk_size_gb.to_string()}
                                            onchange={
                                                let on_number_input = on_number_input.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(val) = input.value().parse::<u32>() {
                                                        on_number_input.emit(("system_disk_size_gb".to_string(), val));
                                                    }
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("system_disk") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).system_disk.clone()}
                                            placeholder="ESSD"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("system_disk".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("data_disk") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).data_disk.clone().unwrap_or_default()}
                                            placeholder="ESSD 100GB"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("data_disk".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Security Product
                                    <div class="column is-12">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                checked={(*form_data).has_security_product}
                                                onchange={
                                                    let on_checkbox_change = on_checkbox_change.clone();
                                                    Callback::from(move |e: Event| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        on_checkbox_change.emit(input.checked());
                                                    })
                                                }
                                            />
                                            { format!(" {} {}", lang.t("has_security_product"), if (*form_data).has_security_product { "(是)" } else { "(否)" }) }
                                        </label>
                                    </div>

                                    // 物理机专用字段 - 由运维人员填写
                                    if (*form_data).resource_type == "physical" {
                                        <div class="column is-12">
                                            <div class="notification is-info is-light">
                                                <p class="has-text-weight-bold">{"物理机详细信息"}</p>
                                                <p>{"物理机的序列号、机架位置、硬件型号、Agent状态、IPMI地址等信息由运维人员在交付时填写"}</p>
                                            </div>
                                        </div>
                                    }

                                    // 云虚拟机专用字段 - 只有选择云资源时才显示
                                    if (*form_data).resource_type == "cloud" {
                                        <div class="column is-6">
                                            <label class="label">{ "计费模式" }</label>
                                            <div class="select is-fullwidth">
                                                <select
                                                    value={(*form_data).cloud_vm_info.as_ref().map(|i| i.billing_mode.clone()).unwrap_or_default()}
                                                    onchange={
                                                        let on_input_change = on_input_change.clone();
                                                        let form_data = form_data.clone();
                                                        Callback::from(move |e: Event| {
                                                            let input: HtmlSelectElement = e.target_unchecked_into();
                                                            let value = input.value();
                                                            let mut data = (*form_data).clone();
                                                            data.cloud_vm_info = Some(shared::CreateCloudVirtualMachineInfo {
                                                                billing_mode: if value.is_empty() { None } else { Some(value.clone()) },
                                                                ..data.cloud_vm_info.clone().unwrap_or_else(|| shared::CreateCloudVirtualMachineInfo {
                                                                    billing_mode: None,
                                                                    expire_time: None,
                                                                    charge_type: None,
                                                                    instance_charge_type: None,
                                                                    internet_charge_type: None,
                                                                    internet_max_bandwidth_out: None,
                                                                    image_id: None,
                                                                    v_switch_id: None,
                                                                    vpc_id: None,
                                                                    security_group_ids: None,
                                                                })
                                                            });
                                                            form_data.set(data);
                                                            on_input_change.emit(("cloud_vm_info.billing_mode".to_string(), value));
                                                        })
                                                    }
                                                >
                                                    <option value="">{ "请选择" }</option>
                                                    <option value="PostPaid">{ "按量付费" }</option>
                                                    <option value="PrePaid">{ "包年包月" }</option>
                                                </select>
                                            </div>
                                        </div>
                                        // 云虚拟机其他技术字段 - 由运维人员填写
                                        <div class="column is-12">
                                            <div class="notification is-info is-light">
                                                <p class="has-text-weight-bold">{"云虚拟机详细信息"}</p>
                                                <p>{"云虚拟机的到期时间、镜像ID、VPC ID、交换机ID、安全组ID、公网带宽等技术信息由运维人员在交付时填写"}</p>
                                            </div>
                                        </div>
                                    }

                                    // Remarks
                                    <div class="column is-12">
                                        <label class="label">{ lang.t("remarks") }</label>
                                        <textarea
                                            class="textarea"
                                            rows="3"
                                            value={(*form_data).remarks.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                                    on_input_change.emit(("remarks".to_string(), input.value()));
                                                })
                                            }
                                        ></textarea>
                                    </div>
                                </div>
                            </section>
                            <footer class="modal-card-foot">
                                <button class="button is-success" onclick={on_submit.clone()}>{ lang.t("save") }</button>
                                <button class="button" onclick={reset_form.clone()}>{ lang.t("cancel") }</button>
                            </footer>
                        </div>
                    </div>
                }

                if filtered_resources.is_empty() && !*loading {
                    <p class="has-text-centered has-text-grey">{ "暂无业务资源" }</p>
                } else if filtered_resources.is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else {
                    <div class="table-container" style="overflow-x: auto;">
                        <table class="table is-fullwidth is-hoverable is-striped" style="min-width: max-content;">
                            <thead>
                                <tr>
                                    <th>{ lang.t("ecs_name") }</th>
                                    <th>{ "资源类型" }</th>
                                    <th>{ lang.t("ecs_status") }</th>
                                    <th>{ "地区" }</th>
                                    <th>{ "云服务" }</th>
                                    <th>{ lang.t("resource_id") }</th>
                                    <th>{ lang.t("cloud_region") }</th>
                                    <th>{ lang.t("cloud_category") }</th>
                                    <th>{ lang.t("customer_name") }</th>
                                    <th>{ lang.t("application_name") }</th>
                                    <th>{ "业务资源ID" }</th>
                                    <th>{ lang.t("ecs_type") }</th>
                                    <th>{ lang.t("ecs_os") }</th>
                                    <th>{ lang.t("cpu_cores") }</th>
                                    <th>{ lang.t("memory_gb") }</th>
                                    <th>{ lang.t("ip_address") }</th>
                                    <th>{ lang.t("actions") }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for filtered_resources.iter().map(|resource| {
                                    let resource_clone = resource.clone();
                                    let resource_for_delete = resource.clone();
                                    let on_edit = on_edit.clone();
                                    let on_delete = on_delete.clone();

                                    html! {
                                        <tr>
                                            <td><strong>{ &resource_clone.ecs_name }</strong></td>
                                            <td>
                                                <span class={classes!("tag", if resource_clone.resource_type == "cloud" { "is-info" } else { "is-primary" })}>
                                                    { if resource_clone.resource_type == "cloud" { "云资源" } else { "物理机" } }
                                                </span>
                                            </td>
                                            <td>
                                                <span class={classes!("tag", status_class(&resource_clone.ecs_status))}>
                                                    { &resource_clone.ecs_status }
                                                </span>
                                            </td>
                                            <td>{ resource_clone.zone_name.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.platform_name.as_ref().unwrap_or(&String::new()) }</td>
                                            <td><code>{ &resource_clone.resource_id }</code></td>
                                            <td>{ &resource_clone.cloud_region }</td>
                                            <td>{ &resource_clone.cloud_category }</td>
                                            <td>{ &resource_clone.customer_name }</td>
                                            <td>{ resource_clone.application_name.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.id }</td>
                                            <td>{ &resource_clone.ecs_type }</td>
                                            <td>{ &resource_clone.ecs_os }</td>
                                            <td>{ resource_clone.cpu_cores }</td>
                                            <td>{ resource_clone.memory_gb }</td>
                                            <td><code>{ &resource_clone.ip_address }</code></td>
                                            <td>
                                                <div class="buttons are-small">
                                                    <button
                                                        class="button is-info is-outlined"
                                                        onclick={
                                                            let on_edit = on_edit.clone();
                                                            let r = resource_clone.clone();
                                                            Callback::from(move |_| on_edit.emit(r.clone()))
                                                        }
                                                    >
                                                        <span class="icon"><i class="fas fa-edit"></i></span>
                                                        <span>{ lang.t("edit") }</span>
                                                    </button>
                                                    <button
                                                        class="button is-danger is-outlined"
                                                        onclick={
                                                            let on_delete = on_delete.clone();
                                                            let id = resource_for_delete.id;
                                                            Callback::from(move |_| {
                                                                if let Some(id_val) = id {
                                                                    on_delete.emit(id_val);
                                                                }
                                                            })
                                                        }
                                                    >
                                                        <span class="icon"><i class="fas fa-trash"></i></span>
                                                        <span>{ lang.t("delete") }</span>
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                })}
                            </tbody>
                        </table>
                    </div>
                }
            </div>
        </div>
    }
}

// ============== Operations Management Component (运维管理) ==============
// 运维人员使用：交付信息

#[function_component]
fn OperationsManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let resources = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Tab state for resource type filtering: "all", "cloud", "physical"
    let active_tab = use_state(|| "all".to_string());

    // Modal states
    let show_detail_modal = use_state(|| false);
    let show_supplement_modal = use_state(|| false);
    let selected_resource = use_state(|| None as Option<BusinessResource>);

    // Form state for supplementing info
    let supplement_form = use_state(|| UpdateBusinessResourceRequest::default());

    // Get token from localStorage
    let token = get_auth_token();

    // Fetch all pending resources (待审批, 审批中, 待交付)
    let fetch_resources = {
        let resources = resources.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let resources = resources.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<BusinessResource>>().await {
                        // Filter for resources that need delivery info (待交付)
                        let filtered: Vec<BusinessResource> = data
                            .into_iter()
                            .filter(|r| r.ecs_status == "待交付")
                            .collect();
                        resources.set(filtered);
                    }
                }
                loading.set(false);
            });
        })
    };

    // Use effect to fetch data on mount
    use_effect_with((), {
        let fetch_resources = fetch_resources.clone();
        move |_| {
            fetch_resources.emit(());
            || ()
        }
    });

    // Status badge class
    let status_class = |status: &str| -> &'static str {
        match status {
            "待审批" => "is-warning",
            "审批中" => "is-info",
            "待交付" => "is-primary",
            "已驳回" => "is-danger",
            "已交付" => "is-success",
            _ => "is-light",
        }
    };

    // Resource type label and class
    let resource_type_display = |resource_type: String| -> (String, &'static str) {
        match resource_type.as_str() {
            "physical" => ("物理机".to_string(), "is-primary"),
            "cloud" => ("云服务器".to_string(), "is-link"),
            _ => (resource_type, "is-light"),
        }
    };

    // Handle view detail
    let on_view_detail = {
        let selected_resource = selected_resource.clone();
        let show_detail_modal = show_detail_modal.clone();

        Callback::from(move |resource: BusinessResource| {
            selected_resource.set(Some(resource));
            show_detail_modal.set(true);
        })
    };

    // Handle supplement info
    let on_supplement_click = {
        let selected_resource = selected_resource.clone();
        let show_supplement_modal = show_supplement_modal.clone();
        let supplement_form = supplement_form.clone();

        Callback::from(move |resource: BusinessResource| {
            let resource_clone = resource.clone();
            selected_resource.set(Some(resource));

            // Pre-fill form with current resource data
            supplement_form.set(UpdateBusinessResourceRequest {
                resource_id: if resource_clone.resource_id.is_empty() { None } else { Some(resource_clone.resource_id.clone()) },
                instance_id: if resource_clone.instance_id.is_empty() { None } else { Some(resource_clone.instance_id.clone()) },
                ip_address: if resource_clone.ip_address.is_empty() { None } else { Some(resource_clone.ip_address.clone()) },
                ecs_login_method: resource_clone.ecs_login_method.clone(),
                ecs_login_username: resource_clone.ecs_login_username.clone(),
                ecs_initial_password: resource_clone.ecs_initial_password.clone(),
                bastion_address: resource_clone.bastion_address.clone(),
                bastion_admin_account: resource_clone.bastion_admin_account.clone(),
                bastion_initial_password: resource_clone.bastion_initial_password.clone(),
                remarks: resource_clone.remarks.clone(),
                // Include physical machine and cloud VM info
                physical_machine_info: resource_clone.physical_machine_info.as_ref().map(|pm| shared::UpdatePhysicalMachineInfo {
                    serial_number: pm.serial_number.clone(),
                    rack_location: pm.rack_location.clone(),
                    hardware_model: pm.hardware_model.clone(),
                    warranty_expiry: pm.warranty_expiry,
                    agent_status: pm.agent_status.clone(),
                    ipmi_address: pm.ipmi_address.clone(),
                }),
                cloud_vm_info: resource_clone.cloud_vm_info.as_ref().map(|cvm| shared::UpdateCloudVirtualMachineInfo {
                    billing_mode: cvm.billing_mode.clone(),
                    expire_time: cvm.expire_time,
                    charge_type: cvm.charge_type.clone(),
                    instance_charge_type: cvm.instance_charge_type.clone(),
                    internet_charge_type: cvm.internet_charge_type.clone(),
                    internet_max_bandwidth_out: cvm.internet_max_bandwidth_out,
                    image_id: cvm.image_id.clone(),
                    v_switch_id: cvm.v_switch_id.clone(),
                    vpc_id: cvm.vpc_id.clone(),
                    security_group_ids: cvm.security_group_ids.clone(),
                }),
                ..Default::default()
            });

            show_supplement_modal.set(true);
        })
    };

    // Submit supplement
    let on_submit_supplement = {
        let token = token.clone();
        let selected_resource = selected_resource.clone();
        let show_supplement_modal = show_supplement_modal.clone();
        let supplement_form = supplement_form.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |_| {
            let token = token.clone();
            let selected_resource = selected_resource.clone();
            let show_supplement_modal = show_supplement_modal.clone();
            let supplement_form = supplement_form.clone();
            let fetch_resources = fetch_resources.clone();

            spawn_local(async move {
                if let Some(resource) = &*selected_resource {
                    if let Some(id) = resource.id {
                        let url = format!("{}/{}", api_url("business-resources"), id);
                        let json_body = serde_json::to_string(&*supplement_form).unwrap_or_default();
                        if Request::put(&url)
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(json_body)
                            .unwrap()
                            .send()
                            .await
                            .is_ok()
                        {
                            fetch_resources.emit(());
                        }
                    }
                }
                selected_resource.set(None);
                show_supplement_modal.set(false);
            });
        })
    };

    // Handle form input change
    let on_form_input = {
        let supplement_form = supplement_form.clone();
        Callback::from(move |(field, value): (String, String)| {
            let mut data = (*supplement_form).clone();
            match field.as_str() {
                "resource_id" => data.resource_id = Some(value),
                "instance_id" => data.instance_id = Some(value),
                "ip_address" => data.ip_address = Some(value),
                "ecs_login_method" => data.ecs_login_method = if value.is_empty() { None } else { Some(value) },
                "ecs_login_username" => data.ecs_login_username = if value.is_empty() { None } else { Some(value) },
                "ecs_initial_password" => data.ecs_initial_password = if value.is_empty() { None } else { Some(value) },
                "bastion_address" => data.bastion_address = if value.is_empty() { None } else { Some(value) },
                "bastion_admin_account" => data.bastion_admin_account = if value.is_empty() { None } else { Some(value) },
                "bastion_initial_password" => data.bastion_initial_password = if value.is_empty() { None } else { Some(value) },
                "remarks" => data.remarks = if value.is_empty() { None } else { Some(value) },
                // Physical machine fields
                "pm_serial_number" => {
                    if let Some(ref mut pm) = data.physical_machine_info {
                        pm.serial_number = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.physical_machine_info = Some(shared::UpdatePhysicalMachineInfo {
                            serial_number: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "pm_rack_location" => {
                    if let Some(ref mut pm) = data.physical_machine_info {
                        pm.rack_location = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.physical_machine_info = Some(shared::UpdatePhysicalMachineInfo {
                            rack_location: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "pm_hardware_model" => {
                    if let Some(ref mut pm) = data.physical_machine_info {
                        pm.hardware_model = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.physical_machine_info = Some(shared::UpdatePhysicalMachineInfo {
                            hardware_model: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "pm_agent_status" => {
                    if let Some(ref mut pm) = data.physical_machine_info {
                        pm.agent_status = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.physical_machine_info = Some(shared::UpdatePhysicalMachineInfo {
                            agent_status: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "pm_warranty_expiry" => {
                    let expiry = if value.is_empty() {
                        None
                    } else {
                        chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", value)).ok().map(|dt| dt.with_timezone(&chrono::Utc))
                    };
                    if let Some(ref mut pm) = data.physical_machine_info {
                        pm.warranty_expiry = expiry;
                    } else {
                        data.physical_machine_info = Some(shared::UpdatePhysicalMachineInfo {
                            warranty_expiry: expiry,
                            ..Default::default()
                        });
                    }
                }
                "pm_ipmi_address" => {
                    if let Some(ref mut pm) = data.physical_machine_info {
                        pm.ipmi_address = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.physical_machine_info = Some(shared::UpdatePhysicalMachineInfo {
                            ipmi_address: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                // Cloud VM fields
                "cvm_billing_mode" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.billing_mode = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            billing_mode: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_expire_time" => {
                    let expiry = if value.is_empty() {
                        None
                    } else {
                        chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", value)).ok().map(|dt| dt.with_timezone(&chrono::Utc))
                    };
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.expire_time = expiry;
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            expire_time: expiry,
                            ..Default::default()
                        });
                    }
                }
                "cvm_charge_type" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.charge_type = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            charge_type: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_instance_charge_type" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.instance_charge_type = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            instance_charge_type: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_internet_charge_type" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.internet_charge_type = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            internet_charge_type: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_image_id" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.image_id = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            image_id: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_vpc_id" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.vpc_id = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            vpc_id: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_v_switch_id" => {
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.v_switch_id = if value.is_empty() { None } else { Some(value) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            v_switch_id: if value.is_empty() { None } else { Some(value) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_security_group_ids" => {
                    let ids: Vec<String> = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.security_group_ids = if ids.is_empty() { None } else { Some(ids) };
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            security_group_ids: if ids.is_empty() { None } else { Some(ids) },
                            ..Default::default()
                        });
                    }
                }
                "cvm_internet_max_bandwidth_out" => {
                    let bandwidth = value.parse::<i32>().ok();
                    if let Some(ref mut cvm) = data.cloud_vm_info {
                        cvm.internet_max_bandwidth_out = bandwidth;
                    } else {
                        data.cloud_vm_info = Some(shared::UpdateCloudVirtualMachineInfo {
                            internet_max_bandwidth_out: bandwidth,
                            ..Default::default()
                        });
                    }
                }
                _ => {}
            }
            supplement_form.set(data);
        })
    };

    // Stats - based on filtered resources
    let filtered_resources: Vec<&BusinessResource> = (*resources).iter().filter(|resource| {
        match (*active_tab).as_str() {
            "all" => true,
            "cloud" => resource.resource_type == "cloud",
            "physical" => resource.resource_type == "physical",
            _ => true,
        }
    }).collect();

    let total_count = filtered_resources.len();
    let pending_count = filtered_resources.iter().filter(|r| r.ecs_status == "待交付").count();
    let running_count = filtered_resources.iter().filter(|r| r.ecs_status == "运行中").count();
    let stopped_count = filtered_resources.iter().filter(|r| r.ecs_status == "已停止").count();

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("operations_management") }</h1>

            // Stats cards
            <div class="columns is-multiline">
                <div class="column is-3">
                    <div class="box has-background-white-bis">
                        <p class="heading">{"总数"}</p>
                        <p class="title is-4">{ total_count }</p>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-warning-light">
                        <p class="heading">{"待交付"}</p>
                        <p class="title is-4 has-text-warning">{ pending_count }</p>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-success-light">
                        <p class="heading">{"运行中"}</p>
                        <p class="title is-4 has-text-success">{ running_count }</p>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-danger-light">
                        <p class="heading">{"已停止"}</p>
                        <p class="title is-4 has-text-danger">{ stopped_count }</p>
                    </div>
                </div>
            </div>

            // Resource type filter tabs: 全部 | 云服务器 | 物理机
            <div class="tabs is-boxed mb-4">
                <ul>
                    <li class={if *active_tab == "all" { "is-active" } else { "" }}>
                        <a onclick={
                            let active_tab = active_tab.clone();
                            Callback::from(move |_| {
                                active_tab.set("all".to_string());
                            })
                        }>{"全部"}</a>
                    </li>
                    <li class={if *active_tab == "cloud" { "is-active" } else { "" }}>
                        <a onclick={
                            let active_tab = active_tab.clone();
                            Callback::from(move |_| {
                                active_tab.set("cloud".to_string());
                            })
                        }>{"云服务器"}</a>
                    </li>
                    <li class={if *active_tab == "physical" { "is-active" } else { "" }}>
                        <a onclick={
                            let active_tab = active_tab.clone();
                            Callback::from(move |_| {
                                active_tab.set("physical".to_string());
                            })
                        }>{"物理机"}</a>
                    </li>
                </ul>
            </div>

            // Resources table
            if *loading {
                <div class="has-text-centered py-6">
                    <span class="icon is-large">
                        <i class="fas fa-spinner fa-spin"></i>
                    </span>
                    <p>{"加载中..."}</p>
                </div>
            } else if filtered_resources.is_empty() {
                <div class="box has-background-white-bis has-text-centered py-6">
                    <p class="is-size-5 has-text-grey">{"暂无数据"}</p>
                </div>
            } else {
                <div class="box">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{"ID"}</th>
                                <th>{"资源类型"}</th>
                                <th>{"名称"}</th>
                                <th>{"云服务名称"}</th>
                                <th>{"区域"}</th>
                                <th>{"客户"}</th>
                                <th>{"状态"}</th>
                                <th>{"操作"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {
                                filtered_resources.iter().map(|resource| {
                                    let on_view_detail = on_view_detail.clone();
                                    let on_supplement_click = on_supplement_click.clone();
                                    let resource_clone = resource.clone();
                                    let (rt_label, rt_class) = resource_type_display(resource_clone.resource_type.clone());

                                    html! {
                                        <tr key={resource.id.unwrap_or(0)}>
                                            <td>{ resource.id.unwrap_or(0) }</td>
                                            <td>
                                                <span class={classes!("tag", rt_class)}>
                                                    { rt_label }
                                                </span>
                                            </td>
                                            <td>{ &resource_clone.ecs_name }</td>
                                            <td>{ &resource_clone.platform_name.clone().unwrap_or_else(|| "-".to_string()) }</td>
                                            <td>{ &resource_clone.cloud_region }</td>
                                            <td>{ &resource_clone.customer_name }</td>
                                            <td>
                                                <span class={classes!("tag", status_class(&resource_clone.ecs_status))}>
                                                    { &resource_clone.ecs_status }
                                                </span>
                                            </td>
                                            <td>
                                                <div class="buttons are-small">
                                                    <button class="button is-info is-light"
                                                        onclick={
                                                            let resource_clone = resource_clone.clone();
                                                            Callback::from(move |_| on_view_detail.emit(resource_clone.clone()))
                                                        }>
                                                        <span class="icon"><i class="fas fa-eye"></i></span>
                                                        <span>{"查看"}</span>
                                                    </button>
                                                    {
                                                        if resource_clone.ecs_status == "待审批" || resource_clone.ecs_status == "审批中" || resource_clone.ecs_status == "待交付" {
                                                            html! {
                                                                <button class="button is-warning is-light"
                                                                    onclick={
                                                                        let resource_clone = resource_clone.clone();
                                                                        Callback::from(move |_| on_supplement_click.emit(resource_clone.clone()))
                                                                    }>
                                                                    <span class="icon"><i class="fas fa-edit"></i></span>
                                                                    <span>{ if resource_clone.ecs_status == "待交付" { "录入信息" } else { "交付信息" } }</span>
                                                                </button>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            }
                        </tbody>
                    </table>
                </div>
            }

            // Detail Modal
            {
                if *show_detail_modal {
                    if let Some(resource) = &*selected_resource {
                        html! {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={
                                    let show_detail_modal = show_detail_modal.clone();
                                    Callback::from(move |_| show_detail_modal.set(false))
                                }></div>
                                <div class="modal-card">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">{"云资源申请详情"}</p>
                                        <button class="delete" onclick={
                                            let show_detail_modal = show_detail_modal.clone();
                                            Callback::from(move |_| show_detail_modal.set(false))
                                        }></button>
                                    </header>
                                    <section class="modal-card-body">
                                        <div class="content">
                                            <table class="table is-fullwidth">
                                                <tr><td><strong>{"ID"}</strong></td><td>{ resource.id.unwrap_or(0) }</td></tr>
                                                <tr><td><strong>{"资源类型"}</strong></td><td>{ &resource.resource_type }</td></tr>
                                                <tr><td><strong>{"ECS名称"}</strong></td><td>{ &resource.ecs_name }</td></tr>
                                                <tr><td><strong>{"云平台（技术底座）"}</strong></td><td>{ &resource.cloud_category }</td></tr>
                                                <tr><td><strong>{"区域"}</strong></td><td>{ &resource.cloud_region }</td></tr>
                                                <tr><td><strong>{"客户"}</strong></td><td>{ &resource.customer_name }</td></tr>
                                                <tr><td><strong>{"应用"}</strong></td><td>{ resource.application_name.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                <tr><td><strong>{"合同"}</strong></td><td>{ resource.contract_name.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                <tr><td><strong>{"实例类型"}</strong></td><td>{ &resource.ecs_type }</td></tr>
                                                <tr><td><strong>{"操作系统"}</strong></td><td>{ &resource.ecs_os }</td></tr>
                                                <tr><td><strong>{"CPU"}</strong></td><td>{ resource.cpu_cores }</td></tr>
                                                <tr><td><strong>{"内存"}</strong></td><td>{ resource.memory_gb }{" GB"}</td></tr>
                                                <tr><td><strong>{"系统盘"}</strong></td><td>{ format!("{} GB {}", resource.system_disk_size_gb, resource.system_disk) }</td></tr>
                                                <tr><td><strong>{"安全产品"}</strong></td><td>{ if resource.has_security_product { "是" } else { "否" } }</td></tr>
                                                <tr><td><strong>{"状态"}</strong></td><td>
                                                    <span class={classes!("tag", status_class(&resource.ecs_status))}>
                                                        { &resource.ecs_status }
                                                    </span>
                                                </td></tr>
                                                <tr><td><strong>{"IP地址"}</strong></td><td>{ if resource.ip_address.is_empty() { "-" } else { &resource.ip_address } }</td></tr>
                                                <tr><td><strong>{"创建时间"}</strong></td><td>{
                                                    resource.created_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default()
                                                }</td></tr>
                                            </table>

                                            // 物理机特定信息
                                            if resource.resource_type == "physical" {
                                                if let Some(ref pm_info) = resource.physical_machine_info {
                                                    <div class="box has-background-light mt-4">
                                                        <h4 class="title is-6 has-text-primary">{"物理机详细信息"}</h4>
                                                        <table class="table is-fullwidth">
                                                            <tr><td><strong>{"序列号"}</strong></td><td>{ pm_info.serial_number.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                            <tr><td><strong>{"机架位置"}</strong></td><td>{ pm_info.rack_location.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                            <tr><td><strong>{"硬件型号"}</strong></td><td>{ pm_info.hardware_model.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                            <tr><td><strong>{"保修到期"}</strong></td><td>{ pm_info.warranty_expiry.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "-".to_string()) }</td></tr>
                                                            <tr><td><strong>{"Agent状态"}</strong></td><td>{ pm_info.agent_status.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                            <tr><td><strong>{"IPMI地址"}</strong></td><td>{ pm_info.ipmi_address.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                        </table>
                                                    </div>
                                                }
                                            }

                                            // 云虚拟机特定信息
                                            if resource.resource_type == "cloud" {
                                                if let Some(ref cvm_info) = resource.cloud_vm_info {
                                                    <div class="box has-background-light mt-4">
                                                        <h4 class="title is-6 has-text-info">{"云虚拟机详细信息"}</h4>
                                                        <table class="table is-fullwidth">
                                                            <tr><td><strong>{"计费模式"}</strong></td><td>{ cvm_info.billing_mode.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                            <tr><td><strong>{"到期时间"}</strong></td><td>{ cvm_info.expire_time.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "-".to_string()) }</td></tr>
                                                            <tr><td><strong>{"镜像ID"}</strong></td><td><code>{ cvm_info.image_id.as_ref().map(|s| s.as_str()).unwrap_or("-") }</code></td></tr>
                                                            <tr><td><strong>{"VPC ID"}</strong></td><td><code>{ cvm_info.vpc_id.as_ref().map(|s| s.as_str()).unwrap_or("-") }</code></td></tr>
                                                            <tr><td><strong>{"交换机ID"}</strong></td><td><code>{ cvm_info.v_switch_id.as_ref().map(|s| s.as_str()).unwrap_or("-") }</code></td></tr>
                                                            <tr><td><strong>{"安全组ID"}</strong></td><td><code>{ cvm_info.security_group_ids.as_ref().map(|ids| ids.first().map(|s| s.as_str()).unwrap_or("-")).unwrap_or("-") }</code></td></tr>
                                                            <tr><td><strong>{"公网带宽"}</strong></td><td>{ cvm_info.internet_max_bandwidth_out.map(|v| format!("{} Mbps", v)).unwrap_or_else(|| "-".to_string()) }</td></tr>
                                                        </table>
                                                    </div>
                                                }
                                            }
                                        </div>
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button" onclick={
                                            let show_detail_modal = show_detail_modal.clone();
                                            Callback::from(move |_| show_detail_modal.set(false))
                                        }>{"关闭"}</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    } else { html! {} }
                } else { html! {} }
            }

            // Supplement Info Modal
            {
                if *show_supplement_modal {
                    if let Some(resource) = &*selected_resource {
                        html! {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={
                                    let show_supplement_modal = show_supplement_modal.clone();
                                    Callback::from(move |_| show_supplement_modal.set(false))
                                }></div>
                                <div class="modal-card" style="width: 800px;">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">
                                            { if resource.ecs_status == "待交付" { "录入云资源" } else { "交付信息" } }
                                        </p>
                                        <button class="delete" onclick={
                                            let show_supplement_modal = show_supplement_modal.clone();
                                            Callback::from(move |_| show_supplement_modal.set(false))
                                        }></button>
                                    </header>
                                    <section class="modal-card-body">
                                        // 重要提示
                                        <div class="notification is-warning is-light" style="margin-bottom: 20px;">
                                            <p class="has-text-weight-bold">{"📋 云资源交付信息录入"}</p>
                                            <p>{"请在云平台上创建资源后，将云平台返回的资源ID和实例ID填写到下方。"}</p>
                                        </div>

                                        // 关键信息：资源ID和实例ID
                                        <div class="box has-background-primary-light" style="margin-bottom: 20px; border-left: 4px solid #3e8ed0;">
                                            <p class="has-text-weight-bold mb-3">{"🔑 必填：云平台资源信息"}</p>
                                            <div class="columns">
                                                <div class="column is-6">
                                                    <div class="field">
                                                        <label class="label has-text-primary">{"资源ID"}</label>
                                                        <div class="control has-icons-left">
                                                            <input class="input is-primary" type="text"
                                                                value={(*supplement_form).resource_id.clone().unwrap_or_default()}
                                                                placeholder="从云平台复制的资源ID"
                                                                onchange={
                                                                    let on_form_input = on_form_input.clone();
                                                                    Callback::from(move |e: Event| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        on_form_input.emit(("resource_id".to_string(), input.value()));
                                                                    })
                                                                }
                                                            />
                                                            <span class="icon is-small is-left">
                                                                <i class="fas fa-key"></i>
                                                            </span>
                                                        </div>
                                                        <p class="help is-primary">{"必填 - 云平台分配的资源唯一标识"}</p>
                                                    </div>
                                                </div>
                                                <div class="column is-6">
                                                    <div class="field">
                                                        <label class="label has-text-primary">{"实例ID"}</label>
                                                        <div class="control has-icons-left">
                                                            <input class="input is-primary" type="text"
                                                                value={(*supplement_form).instance_id.clone().unwrap_or_default()}
                                                                placeholder="从云平台复制的实例ID"
                                                                onchange={
                                                                    let on_form_input = on_form_input.clone();
                                                                    Callback::from(move |e: Event| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        on_form_input.emit(("instance_id".to_string(), input.value()));
                                                                    })
                                                                }
                                                            />
                                                            <span class="icon is-small is-left">
                                                                <i class="fas fa-server"></i>
                                                            </span>
                                                        </div>
                                                        <p class="help is-primary">{"必填 - 云主机实例的唯一标识"}</p>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 连接信息
                                        <p class="has-text-weight-bold mb-2">{"🔐 连接信息"}</p>
                                        <div class="columns">
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"IP地址"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).ip_address.clone().unwrap_or_default()}
                                                            placeholder="192.168.1.100"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("ip_address".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="columns">
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"登录用户名"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).ecs_login_username.clone().unwrap_or_default()}
                                                            placeholder="root / administrator"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("ecs_login_username".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"初始密码"}</label>
                                                    <div class="control">
                                                        <input class="input" type="password"
                                                            value={(*supplement_form).ecs_initial_password.clone().unwrap_or_default()}
                                                            placeholder="••••••••"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("ecs_initial_password".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 堡垒机信息
                                        <div class="columns">
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"堡垒机地址"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).bastion_address.clone().unwrap_or_default()}
                                                            placeholder="bastion.example.com"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("bastion_address".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"堡垒机账号"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).bastion_admin_account.clone().unwrap_or_default()}
                                                            placeholder="admin"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("bastion_admin_account".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"堡垒机密码"}</label>
                                                    <div class="control">
                                                        <input class="input" type="password"
                                                            value={(*supplement_form).bastion_initial_password.clone().unwrap_or_default()}
                                                            placeholder="••••••••"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("bastion_initial_password".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 物理机特定字段
                                        if resource.resource_type == "physical" {
                                            <div class="box has-background-light" style="margin-bottom: 20px;">
                                                <p class="has-text-weight-bold mb-3">{"🖥️ 物理机信息"}</p>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"序列号"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.serial_number.clone()).unwrap_or_default()}
                                                                    placeholder="SN-XXXXXX"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("pm_serial_number".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"机架位置"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.rack_location.clone()).unwrap_or_default()}
                                                                    placeholder="机架-机箱-U位置"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("pm_rack_location".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"硬件型号"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.hardware_model.clone()).unwrap_or_default()}
                                                                    placeholder="Dell R740"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("pm_hardware_model".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"Agent状态"}</label>
                                                            <div class="control">
                                                                <div class="select is-fullwidth">
                                                                    <select
                                                                        onchange={
                                                                            let on_form_input = on_form_input.clone();
                                                                            Callback::from(move |e: Event| {
                                                                                let select: HtmlSelectElement = e.target_unchecked_into();
                                                                                on_form_input.emit(("pm_agent_status".to_string(), select.value()));
                                                                            })
                                                                        }
                                                                    >
                                                                        <option value="" selected={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.agent_status.clone()).is_none()}>{"请选择"}</option>
                                                                        <option value="online" selected={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.agent_status.clone()) == Some("online".to_string())}>{"在线"}</option>
                                                                        <option value="offline" selected={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.agent_status.clone()) == Some("offline".to_string())}>{"离线"}</option>
                                                                        <option value="none" selected={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.agent_status.clone()) == Some("none".to_string())}>{"未安装"}</option>
                                                                    </select>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"保修到期"}</label>
                                                            <div class="control">
                                                                <input class="input" type="date"
                                                                    value={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.warranty_expiry.map(|d| d.format("%Y-%m-%d").to_string())).unwrap_or_default()}
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("pm_warranty_expiry".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"IPMI地址"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).physical_machine_info.as_ref().and_then(|p| p.ipmi_address.clone()).unwrap_or_default()}
                                                                    placeholder="https://192.168.1.1"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("pm_ipmi_address".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }

                                        // 云虚拟机特定字段
                                        if resource.resource_type == "cloud" {
                                            <div class="box has-background-light" style="margin-bottom: 20px;">
                                                <p class="has-text-weight-bold mb-3">{"☁️ 云虚拟机信息"}</p>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"计费模式"}</label>
                                                            <div class="control">
                                                                <div class="select is-fullwidth">
                                                                    <select
                                                                        onchange={
                                                                            let on_form_input = on_form_input.clone();
                                                                            Callback::from(move |e: Event| {
                                                                                let select: HtmlSelectElement = e.target_unchecked_into();
                                                                                on_form_input.emit(("cvm_billing_mode".to_string(), select.value()));
                                                                            })
                                                                        }
                                                                    >
                                                                        <option value="" selected={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.billing_mode.clone()).is_none()}>{"请选择"}</option>
                                                                        <option value="PostPaid" selected={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.billing_mode.clone()) == Some("PostPaid".to_string())}>{"按量付费"}</option>
                                                                        <option value="PrePaid" selected={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.billing_mode.clone()) == Some("PrePaid".to_string())}>{"包年包月"}</option>
                                                                    </select>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"公网带宽"}</label>
                                                            <div class="control">
                                                                <input class="input" type="number"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.internet_max_bandwidth_out.map(|v| v.to_string())).unwrap_or_default()}
                                                                    placeholder="Mbps"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_internet_max_bandwidth_out".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"到期时间"}</label>
                                                            <div class="control">
                                                                <input class="input" type="date"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.expire_time.map(|d| d.format("%Y-%m-%d").to_string())).unwrap_or_default()}
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_expire_time".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"付费类型"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.charge_type.clone()).unwrap_or_default()}
                                                                    placeholder="PostPaid/PrePaid"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_charge_type".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"实例计费类型"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.instance_charge_type.clone()).unwrap_or_default()}
                                                                    placeholder="SpotTC/PrePaid/PostPaid"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_instance_charge_type".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"网络计费类型"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.internet_charge_type.clone()).unwrap_or_default()}
                                                                    placeholder="PayByBandwidth/PayByTraffic"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_internet_charge_type".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"镜像ID"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.image_id.clone()).unwrap_or_default()}
                                                                    placeholder="img-xxxx"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_image_id".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"VPC ID"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.vpc_id.clone()).unwrap_or_default()}
                                                                    placeholder="vpc-xxxx"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_vpc_id".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="columns">
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"交换机ID"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.v_switch_id.clone()).unwrap_or_default()}
                                                                    placeholder="vsw-xxxx"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_v_switch_id".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="column is-6">
                                                        <div class="field">
                                                            <label class="label">{"安全组ID"}</label>
                                                            <div class="control">
                                                                <input class="input" type="text"
                                                                    value={(*supplement_form).cloud_vm_info.as_ref().and_then(|c| c.security_group_ids.as_ref().map(|v| v.join(", "))).unwrap_or_default()}
                                                                    placeholder="sg-xxxx, sg-yyyy"
                                                                    onchange={
                                                                        let on_form_input = on_form_input.clone();
                                                                        Callback::from(move |e: Event| {
                                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                                            on_form_input.emit(("cvm_security_group_ids".to_string(), input.value()));
                                                                        })
                                                                    }
                                                                />
                                                                <p class="help">{"多个安全组用逗号分隔"}</p>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }

                                        <div class="field">
                                            <label class="label">{"备注"}</label>
                                            <div class="control">
                                                <textarea class="textarea"
                                                    placeholder="添加备注信息..."
                                                    rows="3"
                                                    onchange={
                                                        let on_form_input = on_form_input.clone();
                                                        Callback::from(move |e: Event| {
                                                            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
                                                            on_form_input.emit(("remarks".to_string(), textarea.value()));
                                                        })
                                                    }
                                                >
                                                    { (*supplement_form).remarks.clone().unwrap_or_default() }
                                                </textarea>
                                            </div>
                                        </div>

                                        {
                                            if resource.ecs_status == "待交付" {
                                                html! {
                                                    <div class="notification is-info">
                                                        <p class="heading">{"提示"}</p>
                                                        <p>{"提交后将创建云资产记录，请确保信息填写完整。"}</p>
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button is-primary" onclick={on_submit_supplement.clone()}>
                                            {"保存"}
                                        </button>
                                        <button class="button" onclick={
                                            let show_supplement_modal = show_supplement_modal.clone();
                                            Callback::from(move |_| show_supplement_modal.set(false))
                                        }>{"取消"}</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    } else { html! {} }
                } else { html! {} }
            }
        </div>
    }
}

// Helper function for active tab
fn active_tab_filter<F>(f: F, resources: &UseStateHandle<Vec<BusinessResource>>) -> &'static str
where
    F: Fn(&BusinessResource) -> bool,
{
    if (*resources).iter().filter(|r| f(r)).count() > 0 {
        "is-active"
    } else {
        ""
    }
}

// Helper function to get provider name
fn provider_name(provider: &str) -> String {
    match provider {
        "aliyun" => "阿里云".to_string(),
        "tencent" => "腾讯云".to_string(),
        "huawei" => "华为云".to_string(),
        "aws" => "AWS".to_string(),
        _ => provider.to_string(),
    }
}
