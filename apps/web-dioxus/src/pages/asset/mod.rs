use crate::{components::*, services::api};
use dioxus::prelude::*;
use serde_json::{Value, json};

#[component]
pub fn AssetPage() -> Element {
    let mut tab = use_signal(|| "assets");
    rsx! {
        div {
            h1 { class: "text-xl font-bold text-gray-800 mb-4", "资产管理" }
            div { class: "flex gap-2 mb-4",
                button { class: if tab()=="assets"{"ant-btn ant-btn-primary"}else{"ant-btn"}, onclick:move|_|tab.set("assets"), "资产列表" }
                button { class: if tab()=="cloud"{"ant-btn ant-btn-primary"}else{"ant-btn"}, onclick:move|_|tab.set("cloud"), "云资产" }
                button { class: if tab()=="business"{"ant-btn ant-btn-primary"}else{"ant-btn"}, onclick:move|_|tab.set("business"), "业务资源" }
            }
            if tab()=="assets" { AssetTab {} } else if tab()=="cloud" { CloudTab {} } else { BizTab {} }
        }
    }
}

#[component]
fn AssetTab() -> Element {
    let mut data = use_signal(Vec::<Value>::new);
    let mut total = use_signal(|| 0i64);
    let mut page = use_signal(|| 1i64);
    let mut loading = use_signal(|| true);
    let mut show = use_signal(|| false);
    let mut form = use_signal(|| json!({}));
    let mut saving = use_signal(|| false);
    let load = move || {
        spawn(async move {
            loading.set(true);
            if let Ok((l, t)) = api::page("asset", page(), 10).await {
                data.set(l);
                total.set(t);
            }
            loading.set(false);
        });
    };
    use_effect(load);
    let cols = vec![
        Column {
            key: "name".to_string(),
            title: "名称".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "ip".to_string(),
            title: "IP地址".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "zone".to_string(),
            title: "区域".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "os".to_string(),
            title: "系统".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "deviceType".to_string(),
            title: "设备类型".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "owner".to_string(),
            title: "负责人".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "lastScanned".to_string(),
            title: "最近扫描".to_string(),
            width: Some("140px".to_string()),
            render: None,
        },
    ];
    rsx! {
        DataTable{columns:cols,data:data(),loading:loading(),page:page(),page_size:10,total:total(),on_page_change:EventHandler::new(move|p: i64|page.set(p)),on_edit:None,on_delete:Some(EventHandler::new(move|id: String|{spawn(async move{let _=api::remove("asset",&id).await;load();});})),extra_actions:None}
        button{class:"ant-btn ant-btn-primary mt-4",onclick:move|_|{form.set(json!({}));show.set(true);},"+ 新增资产"}
        Modal{title:"新增资产",visible:show(),loading:saving(),on_close:EventHandler::new(move|_: ()|show.set(false)),on_save:EventHandler::new(move|_: ()|{let f=form();saving.set(true);spawn(async move{if api::create("asset",f).await.is_ok(){show_toast("保存成功",ToastKind::Success);show.set(false);page.set(1);}saving.set(false);});}),children:rsx!{
            FormField{label:"名称",required:true,children:rsx!{input{class:"ant-input",oninput:move|e|{let mut f=form();f["name"]=json!(e.value());form.set(f);}}}}
            FormField{label:"IP地址",required:true,children:rsx!{input{class:"ant-input",oninput:move|e|{let mut f=form();f["ip"]=json!(e.value());form.set(f);}}}}
            FormField{label:"负责人",required:false,children:rsx!{input{class:"ant-input",oninput:move|e|{let mut f=form();f["owner"]=json!(e.value());form.set(f);}}}}
        }}
    }
}

#[component]
fn CloudTab() -> Element {
    let mut data = use_signal(Vec::<Value>::new);
    let mut total = use_signal(|| 0i64);
    let mut page = use_signal(|| 1i64);
    let mut loading = use_signal(|| true);
    let load = move || {
        spawn(async move {
            loading.set(true);
            if let Ok((l, t)) = api::page("cloud-asset", page(), 10).await {
                data.set(l);
                total.set(t);
            }
            loading.set(false);
        });
    };
    use_effect(load);
    let cols = vec![
        Column {
            key: "name".to_string(),
            title: "名称".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "instanceId".to_string(),
            title: "实例ID".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "platformName".to_string(),
            title: "云平台".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "publicIp".to_string(),
            title: "公网IP".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "instanceType".to_string(),
            title: "类型".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "status".to_string(),
            title: "状态".to_string(),
            width: Some("80px".to_string()),
            render: None,
        },
    ];
    rsx! { DataTable{columns:cols,data:data(),loading:loading(),page:page(),page_size:10,total:total(),on_page_change:EventHandler::new(move|p: i64|page.set(p)),on_edit:None,on_delete:None,extra_actions:None} }
}

#[component]
fn BizTab() -> Element {
    let mut data = use_signal(Vec::<Value>::new);
    let mut total = use_signal(|| 0i64);
    let mut page = use_signal(|| 1i64);
    let mut loading = use_signal(|| true);
    let mut show = use_signal(|| false);
    let mut form = use_signal(|| json!({"resourceType":"cloud","ecsStatus":"运行中"}));
    let mut saving = use_signal(|| false);
    let load = move || {
        spawn(async move {
            loading.set(true);
            if let Ok((l, t)) = api::page("business-resource", page(), 10).await {
                data.set(l);
                total.set(t);
            }
            loading.set(false);
        });
    };
    use_effect(load);
    let cols = vec![
        Column {
            key: "ecsName".to_string(),
            title: "名称".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "resourceType".to_string(),
            title: "类型".to_string(),
            width: Some("80px".to_string()),
            render: None,
        },
        Column {
            key: "customerName".to_string(),
            title: "客户".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "ipAddress".to_string(),
            title: "IP".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "applicationStatus".to_string(),
            title: "申请状态".to_string(),
            width: None,
            render: None,
        },
        Column {
            key: "deliveryStatus".to_string(),
            title: "交付状态".to_string(),
            width: None,
            render: None,
        },
    ];
    rsx! {
        DataTable{columns:cols,data:data(),loading:loading(),page:page(),page_size:10,total:total(),on_page_change:EventHandler::new(move|p: i64|page.set(p)),on_edit:None,on_delete:Some(EventHandler::new(move|id: String|{spawn(async move{let _=api::remove("business-resource",&id).await;load();});})),extra_actions:None}
        button{class:"ant-btn ant-btn-primary mt-4",onclick:move|_|{form.set(json!({"resourceType":"cloud","ecsStatus":"运行中"}));show.set(true);},"+ 新增业务资源"}
        Modal{title:"新增业务资源",visible:show(),loading:saving(),on_close:EventHandler::new(move|_: ()|show.set(false)),on_save:EventHandler::new(move|_: ()|{let f=form();saving.set(true);spawn(async move{if api::create("business-resource",f).await.is_ok(){show_toast("保存成功",ToastKind::Success);show.set(false);page.set(1);}saving.set(false);});}),children:rsx!{
            FormField{label:"名称",required:true,children:rsx!{input{class:"ant-input",oninput:move|e|{let mut f=form();f["ecsName"]=json!(e.value());form.set(f);}}}}
            FormField{label:"类型",required:true,children:rsx!{select{class:"ant-input",onchange:move|e|{let mut f=form();f["resourceType"]=json!(e.value());form.set(f);},option{value:"cloud","云服务器"}option{value:"physical","物理机"}option{value:"network","网络策略"}}}}
            FormField{label:"客户",required:true,children:rsx!{input{class:"ant-input",oninput:move|e|{let mut f=form();f["customerName"]=json!(e.value());form.set(f);}}}}
            FormField{label:"IP地址",required:true,children:rsx!{input{class:"ant-input",oninput:move|e|{let mut f=form();f["ipAddress"]=json!(e.value());form.set(f);}}}}
        }}
    }
}
