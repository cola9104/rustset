pub mod asset;
use dioxus::prelude::*;
use serde_json::{Value, json};
use crate::components::*;

#[component] pub fn InfraConfig() -> Element { crud("参数配置","config",vec![Column{key:"category".to_string(),title:"分类".to_string(),width:None},Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"configKey".to_string(),title:"键".to_string(),width:None},Column{key:"value".to_string(),title:"值".to_string(),width:None},Column{key:"visible".to_string(),title:"可见".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraDatasource() -> Element { crud("数据源","data-source-config",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"url".to_string(),title:"URL".to_string(),width:None},Column{key:"username".to_string(),title:"用户名".to_string(),width:None}]) }
#[component] pub fn InfraFileConfig() -> Element { crud("文件配置","file-config",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"storage".to_string(),title:"存储".to_string(),width:Some("80px".to_string())},Column{key:"master".to_string(),title:"主配置".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraFile() -> Element { readonly("文件管理",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"path".to_string(),title:"路径".to_string(),width:None},Column{key:"url".to_string(),title:"URL".to_string(),width:None},Column{key:"size".to_string(),title:"大小".to_string(),width:Some("100px".to_string())}]) }
#[component] pub fn InfraJob() -> Element { crud("定时任务","job",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"handlerName".to_string(),title:"处理器".to_string(),width:None},Column{key:"cronExpression".to_string(),title:"Cron".to_string(),width:Some("140px".to_string())},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraJobLog() -> Element { readonly("任务日志",vec![Column{key:"handlerName".to_string(),title:"处理器".to_string(),width:None},Column{key:"beginTime".to_string(),title:"开始".to_string(),width:Some("160px".to_string())},Column{key:"duration".to_string(),title:"耗时ms".to_string(),width:Some("80px".to_string())},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraCodegen() -> Element { readonly("代码生成",vec![Column{key:"tableName".to_string(),title:"表名".to_string(),width:None},Column{key:"className".to_string(),title:"类名".to_string(),width:None},Column{key:"businessName".to_string(),title:"业务名".to_string(),width:None}]) }
#[component] pub fn InfraApiAccessLog() -> Element { readonly("API访问日志",vec![Column{key:"requestMethod".to_string(),title:"方法".to_string(),width:Some("80px".to_string())},Column{key:"requestUrl".to_string(),title:"URL".to_string(),width:None},Column{key:"userIp".to_string(),title:"IP".to_string(),width:Some("130px".to_string())},Column{key:"duration".to_string(),title:"耗时ms".to_string(),width:Some("80px".to_string())},Column{key:"resultCode".to_string(),title:"结果".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraApiErrorLog() -> Element { readonly("API错误日志",vec![Column{key:"exceptionName".to_string(),title:"异常".to_string(),width:None},Column{key:"exceptionMessage".to_string(),title:"消息".to_string(),width:None},Column{key:"processStatus".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraRedisMonitor() -> Element { monitor("Redis监控","/infra/redis/monitor",vec!["version","uptimeInDays","connectedClients","usedMemoryHuman"]) }
#[component] pub fn InfraServerMonitor() -> Element { monitor("服务器监控","/infra/rust/monitor",vec!["cpu.cpuNum","jvm.total","sys.osName"]) }
#[component] pub fn InfraRust() -> Element { monitor("Rust监控","/infra/rust/monitor",vec!["cpu.cpuNum","jvm.max","sys.computerName"]) }
#[component] pub fn InfraPostgreSql() -> Element { monitor("PostgreSQL监控","/infra/postgresql/monitor",vec!["version","databaseName","maxConnections","numbackends"]) }
#[component] pub fn InfraDemo01() -> Element { crud("Demo01联系人","demo01-contact",vec![Column{key:"name".to_string(),title:"姓名".to_string(),width:None},Column{key:"contact".to_string(),title:"联系方式".to_string(),width:None}]) }
#[component] pub fn InfraDemo02() -> Element { crud("Demo02分类","demo02-category",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None}]) }
#[component] pub fn InfraDemo03() -> Element { crud("Demo03学生","demo03-student-normal",vec![Column{key:"name".to_string(),title:"姓名".to_string(),width:None},Column{key:"age".to_string(),title:"年龄".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraBuild() -> Element { info_page("表单构建","📋","在此构建自定义表单，拖拽组件生成页面") }
#[component] pub fn InfraTraces() -> Element { info_page("请求链路","🔗","查看分布式请求追踪和调用链") }
#[component] pub fn InfraSwagger() -> Element { info_page("接口文档","📖",r"API 文档地址: /openapi.json") }
#[component] pub fn InfraWebSocket() -> Element { info_page("WebSocket","📡","WebSocket 连接测试工具") }
#[component] pub fn InfraJobLogDetail() -> Element { readonly("调度日志",vec![Column{key:"jobName".to_string(),title:"任务".to_string(),width:None},Column{key:"startTime".to_string(),title:"开始时间".to_string(),width:Some("160px".to_string())},Column{key:"duration".to_string(),title:"耗时ms".to_string(),width:Some("80px".to_string())},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn InfraCodegenEdit() -> Element { info_page("生成配置修改","💻","编辑代码生成配置，选择表和字段") }

fn crud(title:&str,module:&str,columns:Vec<Column>)->Element{
    let m=module.to_string();let mut data=use_signal(Vec::<Value>::new);let mut total=use_signal(||0i64);let mut pg=use_signal(||1i64);let mut loading=use_signal(||true);let mut show=use_signal(||false);let mut edit_id=use_signal(||None::<String>);let mut form=use_signal(||json!({}));let mut saving=use_signal(||false);let mut keyword=use_signal(String::new);
    let load={let m=m.clone();let cols=columns.clone();move||{let m=m.clone();let kw=keyword();let search_key=cols.first().map(|c|c.key.clone()).unwrap_or_else(||"name".to_string());spawn(async move{loading.set(true);if !kw.is_empty(){let path=format!("/infra/{}/page?pageNo={}&pageSize=10&{}={}",m,pg(),search_key,kw);if let Ok(v)=crate::services::api::get_path(&path).await{if let Some(arr)=v["data"]["list"].as_array(){data.set(arr.clone());}total.set(v["data"]["total"].as_i64().unwrap_or(0));}}else{if let Ok((l,t))=crate::services::api::page(&m,pg(),10).await{data.set(l);total.set(t);}}loading.set(false);});}};
    use_effect(load);
    let on_save={let m=m.clone();move|_|{let f=form();let m=m.clone();saving.set(true);spawn(async move{let r=if edit_id().is_some(){crate::services::api::update(&m,f).await}else{crate::services::api::create(&m,f).await.map(|_|())};saving.set(false);if r.is_ok(){toast("保存成功",ToastKind::Success);show.set(false);pg.set(1);}else{toast(&format!("失败:{}",r.unwrap_err()),ToastKind::Error);}});}};
    let t=title.to_string();let dt=data();let tot=total();let p=pg();let ld=loading();let sv=saving();let sh=show();let kw=keyword();let m2=m.clone();let cols2=columns.clone();let m3=m.clone();let cols3=columns.clone();let m4=m.clone();
    rsx!{
        PageHeader{title:t,actions:rsx!{button{class:"ant-btn ant-btn-primary",onclick:move|_|{form.set(json!({}));edit_id.set(None);show.set(true);},"+ 新增"}}}
        div{class:"flex gap-2 mb-4",
            input{class:"ant-input w-[260px]",placeholder:"搜索...",value:"{kw}",oninput:move|e|keyword.set(e.value()),onkeydown:{let sk=cols2.first().map(|c|c.key.clone()).unwrap_or_else(||"name".to_string());move|e|{if e.key()==Key::Enter{let m=m2.clone();let sk=sk.clone();let kw2=keyword();spawn(async move{loading.set(true);if!kw2.is_empty(){let path=format!("/infra/{}/page?pageNo={}&pageSize=10&{}={}",m,pg(),sk,kw2);if let Ok(v)=crate::services::api::get_path(&path).await{if let Some(arr)=v["data"]["list"].as_array(){data.set(arr.clone());}total.set(v["data"]["total"].as_i64().unwrap_or(0));}}else{if let Ok((l,t))=crate::services::api::page(&m,pg(),10).await{data.set(l);total.set(t);}}loading.set(false);});}}}}
            button{class:"ant-btn ant-btn-primary text-sm",onclick:{let sk=cols3.first().map(|c|c.key.clone()).unwrap_or_else(||"name".to_string());move|_|{let m=m3.clone();let sk=sk.clone();let kw2=keyword();spawn(async move{loading.set(true);if!kw2.is_empty(){let path=format!("/infra/{}/page?pageNo={}&pageSize=10&{}={}",m,pg(),sk,kw2);if let Ok(v)=crate::services::api::get_path(&path).await{if let Some(arr)=v["data"]["list"].as_array(){data.set(arr.clone());}total.set(v["data"]["total"].as_i64().unwrap_or(0));}}else{if let Ok((l,t))=crate::services::api::page(&m,pg(),10).await{data.set(l);total.set(t);}}loading.set(false);});}},"🔍 搜索"}
            button{class:"ant-btn text-sm",onclick:move|_|{keyword.set(String::new());let m=m4.clone();spawn(async move{loading.set(true);if let Ok((l,t))=crate::services::api::page(&m,pg(),10).await{data.set(l);total.set(t);}loading.set(false);});},"↺ 重置"}
        }
        DataTable{columns:columns.clone(),data:dt,loading:ld,page:p,page_size:10,total:tot,on_page_change:EventHandler::new(move|p:i64|pg.set(p)),on_edit:Some(EventHandler::new(move|id:String|{if let Some(i)=data().iter().find(|v|get_id(v)==id){form.set(i.clone());edit_id.set(Some(id));show.set(true);}})),on_delete:Some(EventHandler::new({let m=m.clone();move|id:String|{let m=m.clone();spawn(async move{let _=crate::services::api::remove(&m,&id).await;toast("删除成功",ToastKind::Success);});}})),extra_row:None}
        Modal{title:if edit_id().is_some(){"编辑".to_string()}else{"新增".to_string()},visible:sh,width:None,loading:sv,on_close:EventHandler::new(move|_|show.set(false)),on_save:Some(EventHandler::new(on_save)),children:{let cols=columns.clone();let items:Vec<_>=cols.iter().map(|c|{let k=c.key.clone();let l=c.title.clone();let ft=form_type(&k);match ft{0=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{select{class:"ant-input",onchange:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);},option{value:"","请选择"}option{value:"0","启用"}option{value:"1","禁用"}}}}}},1=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{input{class:"ant-input",r#type:"number",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value().parse::<i64>().unwrap_or(0));form.set(f);}}}}}},2=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{textarea{class:"ant-input",rows:"3",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);}}}}}},_=>rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{input{class:"ant-input",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);}}}}}}}}).collect::<Vec<_>>();rsx!{for item in items{{item}}}}}
    }
}
fn form_type(key:&str)->u8{
    let k=key.to_lowercase();
    if k=="status"||k.ends_with("status"){0}
    else if k=="sort"||k.contains("sort")||k=="order"||k.contains("age")||k=="type"||k.contains("id")&&k!="id"{1}
    else if k.contains("remark")||k.contains("desc")||k.contains("content")||k.contains("note")||k.contains("address"){2}
    else{3}
}
fn readonly(title:&str,columns:Vec<Column>)->Element{
    let mut data=use_signal(Vec::<Value>::new);let mut total=use_signal(||0i64);let mut pg=use_signal(||1i64);let mut loading=use_signal(||false);
    rsx!{PageHeader{title:title.to_string(),actions:None}DataTable{columns,data:data(),loading:loading(),page:pg(),page_size:10,total:total(),on_page_change:EventHandler::new(move|p:i64|pg.set(p)),on_edit:None,on_delete:None,extra_row:None}}
}
fn info_page(title:&str,icon:&str,desc:&str)->Element{let t=title.to_string();let i=icon.to_string();let d=desc.to_string();rsx!{div{class:"bg-white rounded-lg shadow-sm border",div{class:"flex items-center justify-between px-6 py-4 border-b",h3{class:"text-base font-semibold","{t}"}}div{class:"p-6",div{class:"py-12 text-center text-gray-400",span{class:"text-4xl block mb-3","{i}"}p{class:"text-lg","{d}"}}}}}}
fn placeholder(title:&str)->Element{
    let t=title.to_string();
    rsx!{div{class:"bg-white rounded-lg shadow-sm border",
        div{class:"flex items-center justify-between px-6 py-4 border-b",
            h3{class:"text-base font-semibold","{t}"}
        }
        div{class:"p-6",
            div{class:"py-12 text-center text-gray-400",
                span{class:"text-4xl block mb-3","🚧"}
                p{class:"text-lg","页面建设中..."}
            }
        }
    }}
}
fn monitor(title:&str,api_path:&str,fields:Vec<&str>)->Element{
    let mut data=use_signal(||None::<Value>);let mut loading=use_signal(||false);
    let t=title.to_string();let path=api_path.to_string();
    let fetch={let p=path;move||{loading.set(true);let p2=p.clone();spawn(async move{if let Ok(v)=crate::services::api::action(&p2,json!({})).await{data.set(Some(v["data"].clone()));}loading.set(false);});}};
    use_effect(fetch);
    let info=data();let has=info.is_some();let loading_val=loading();
    let rows:Vec<(String,String)>=fields.iter().map(|k|{
        let parts:Vec<&str>=k.splitn(2,'.').collect();
        let val=if parts.len()==2{info.as_ref().and_then(|x|x[parts[0]][parts[1]].as_str())}else{info.as_ref().and_then(|x|x[parts[0]].as_str())};
        let lbl=if parts.len()==2{parts[1].to_string()}else{parts[0].to_string()};
        (lbl,val.unwrap_or("-").to_string())
    }).collect();
    rsx!{div{
        PageHeader{title:t,actions:None}
        if loading_val{Spinner{text:Some("加载中...".into())}}
        else if has{div{class:"bg-white rounded-lg shadow-sm border",
            div{class:"p-4",for(lbl,val) in &rows{div{class:"flex justify-between py-2 border-b text-sm",span{class:"text-gray-500","{lbl}"}span{class:"font-medium","{val}"}}}
        }}
    }else{EmptyState{text:Some("无数据".into())}}
    }}
}
