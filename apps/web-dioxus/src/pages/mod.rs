pub mod _core;
pub mod ai;
pub mod business;
pub mod cloud;
pub mod dashboard;
pub mod infra;
pub mod provider;
pub mod risk;
pub mod room;
pub mod security;
pub mod system;
pub mod task;
pub mod ticket;
pub mod zone;

// Shared infra CRUD helper used by standalone resource pages.
// NOT a #[component] — hooks inherit scope from the calling component.
#[allow(dead_code)]
pub(crate) fn infra_crud(title: &str, module: &str, columns: Vec<crate::components::Column>) -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    use serde_json::{json, Value};
    use crate::components::*;
    use crate::services::api;

    let m = module.to_string();let mut data = use_signal(Vec::<Value>::new);let mut total = use_signal(|| 0i64);
    let mut pg = use_signal(|| 1i64);let mut loading = use_signal(|| true);let mut show = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);let mut form = use_signal(|| json!({}));let mut saving = use_signal(|| false);let mut keyword=use_signal(String::new);
    let load={let m=m.clone();let cols=columns.clone();move||{let m=m.clone();let kw=keyword();let sk=cols.first().map(|c|c.key.clone()).unwrap_or_else(||"name".to_string());spawn(async move{loading.set(true);if!kw.is_empty(){let path=format!("/infra/{}/page?pageNo={}&pageSize=10&{}={}",m,pg(),sk,kw);if let Ok(v)=crate::services::api::get_path(&path).await{if let Some(arr)=v["data"]["list"].as_array(){data.set(arr.clone());}total.set(v["data"]["total"].as_i64().unwrap_or(0));}}else{if let Ok((l,t))=api::page(&m,pg(),10).await{data.set(l);total.set(t);}}loading.set(false);});}};
    use_effect(load);let on_save={let m=m.clone();move|_|{let f=form();let m=m.clone();saving.set(true);spawn(async move{let r=if edit_id().is_some(){api::update(&m,f).await}else{api::create(&m,f).await.map(|_|())};saving.set(false);if r.is_ok(){toast("保存成功",ToastKind::Success);show.set(false);pg.set(1);}else{toast(&format!("失败:{}",r.unwrap_err()),ToastKind::Error);}});}};
    let t=title.to_string();let dt=data();let tot=total();let p=pg();let ld=loading();let sv=saving();let sh=show();let kw=keyword();let m2=m.clone();let cols2=columns.clone();let m3=m.clone();let cols3=columns.clone();let m4=m.clone();
    rsx!{
        PageHeader{title:t,actions:rsx!{button{class:"ant-btn ant-btn-primary",onclick:move|_|{form.set(json!({}));edit_id.set(None);show.set(true);},"+ 新增"}}}
        div{class:"flex gap-2 mb-4",
            input{class:"ant-input w-[260px]",placeholder:"搜索...",value:"{kw}",oninput:move|e|keyword.set(e.value()),onkeydown:{let sk=cols2.first().map(|c|c.key.clone()).unwrap_or_else(||"name".to_string());move|e|{if e.key()==Key::Enter{let m=m2.clone();let sk=sk.clone();let kw2=keyword();spawn(async move{loading.set(true);if!kw2.is_empty(){let path=format!("/infra/{}/page?pageNo={}&pageSize=10&{}={}",m,pg(),sk,kw2);if let Ok(v)=crate::services::api::get_path(&path).await{if let Some(arr)=v["data"]["list"].as_array(){data.set(arr.clone());}total.set(v["data"]["total"].as_i64().unwrap_or(0));}}else{if let Ok((l,t))=api::page(&m,pg(),10).await{data.set(l);total.set(t);}}loading.set(false);});}}}}
            button{class:"ant-btn ant-btn-primary text-sm",onclick:{let sk=cols3.first().map(|c|c.key.clone()).unwrap_or_else(||"name".to_string());move|_|{let m=m3.clone();let sk=sk.clone();let kw2=keyword();spawn(async move{loading.set(true);if!kw2.is_empty(){let path=format!("/infra/{}/page?pageNo={}&pageSize=10&{}={}",m,pg(),sk,kw2);if let Ok(v)=crate::services::api::get_path(&path).await{if let Some(arr)=v["data"]["list"].as_array(){data.set(arr.clone());}total.set(v["data"]["total"].as_i64().unwrap_or(0));}}else{if let Ok((l,t))=api::page(&m,pg(),10).await{data.set(l);total.set(t);}}loading.set(false);});}},"🔍 搜索"}
            button{class:"ant-btn text-sm",onclick:move|_|{keyword.set(String::new());let m=m4.clone();spawn(async move{loading.set(true);if let Ok((l,t))=api::page(&m,pg(),10).await{data.set(l);total.set(t);}loading.set(false);});},"↺ 重置"}
        }
        DataTable{columns:columns.clone(),data:dt,loading:ld,page:p,page_size:10,total:tot,on_page_change:EventHandler::new(move|p:i64|pg.set(p)),on_edit:Some(EventHandler::new(move|id:String|{if let Some(i)=data().iter().find(|v|gid(v)==id){form.set(i.clone());edit_id.set(Some(id));show.set(true);}})),on_delete:Some(EventHandler::new({let m=m.clone();move|id:String|{let m=m.clone();spawn(async move{let _=api::remove(&m,&id).await;toast("删除成功",ToastKind::Success);});}})),extra_row:None}
        Modal{title:if edit_id().is_some(){"编辑".to_string()}else{"新增".to_string()},visible:sh,width:None,loading:sv,on_close:EventHandler::new(move|_|show.set(false)),on_save:Some(EventHandler::new(on_save)),children:{let cl=columns.clone();let items:Vec<_>=cl.iter().map(|c|{let k=c.key.clone();let l=c.title.clone();let ft=if k=="status"||k.ends_with("Status"){0}else if k.contains("sort")||k.contains("order"){1}else if k.contains("remark")||k.contains("desc")||k.contains("content")||k.contains("note")||k.contains("address"){2}else{3};match ft{0=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{select{class:"ant-input",onchange:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);},option{value:"","请选择"}option{value:"0","启用"}option{value:"1","禁用"}}}}}},1=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{input{class:"ant-input",r#type:"number",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value().parse::<i64>().unwrap_or(0));form.set(f);}}}}}},2=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{textarea{class:"ant-input",rows:"3",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);}}}}}},_=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{input{class:"ant-input",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);}}}}}}}}).collect::<Vec<_>>();rsx!{for item in items{ {item} }}}}
    }
}
fn gid(row: &serde_json::Value) -> String { row.get("id").and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(String::from))).unwrap_or_default() }
