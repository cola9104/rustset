pub mod chat;
use dioxus::prelude::*;
use serde_json::{Value, json};
use crate::components::*;
use crate::services::api;

pub use chat::AiChat;

#[component] pub fn AiImage() -> Element { image_page() }
#[component] pub fn AiWrite() -> Element { crud("AI写作","ai-write",vec![Column{key:"title".to_string(),title:"标题".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiMusic() -> Element { crud("AI音乐","ai-music",vec![Column{key:"title".to_string(),title:"标题".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiModel() -> Element { crud("模型管理","ai-model",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"type".to_string(),title:"类型".to_string(),width:Some("100px".to_string())},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiChatRole() -> Element { crud("聊天角色","ai-chat-role",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"prompt".to_string(),title:"提示词".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiTool() -> Element { crud("工具管理","ai-tool",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"type".to_string(),title:"类型".to_string(),width:Some("100px".to_string())}]) }
#[component] pub fn AiKnowledge() -> Element { crud("知识库","ai-knowledge",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"docCount".to_string(),title:"文档数".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiChatManager() -> Element { readonly_page("对话管理","ai-chat-conversation",vec![Column{key:"title".to_string(),title:"标题".to_string(),width:None},Column{key:"userId".to_string(),title:"用户".to_string(),width:None},Column{key:"createTime".to_string(),title:"创建时间".to_string(),width:Some("160px".to_string())}]) }
#[component] pub fn AiImageManager() -> Element { readonly_page("绘图管理","ai-image",vec![Column{key:"prompt".to_string(),title:"提示词".to_string(),width:None},Column{key:"platform".to_string(),title:"平台".to_string(),width:Some("100px".to_string())},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiImageSquare() -> Element { readonly_page("绘图作品","ai-image-public",vec![Column{key:"prompt".to_string(),title:"提示词".to_string(),width:None},Column{key:"userId".to_string(),title:"作者".to_string(),width:None},Column{key:"createTime".to_string(),title:"时间".to_string(),width:Some("160px".to_string())}]) }
#[component] pub fn AiWriteManager() -> Element { readonly_page("写作管理","ai-write",vec![Column{key:"title".to_string(),title:"标题".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiMusicManager() -> Element { readonly_page("音乐管理","ai-music",vec![Column{key:"title".to_string(),title:"标题".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiModelChatRole() -> Element { crud("聊天角色","ai-chat-role",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"category".to_string(),title:"分类".to_string(),width:Some("100px".to_string())}]) }
#[component] pub fn AiModelTool() -> Element { crud("工具管理","ai-tool",vec![Column{key:"name".to_string(),title:"名称".to_string(),width:None},Column{key:"type".to_string(),title:"类型".to_string(),width:Some("100px".to_string())}]) }
#[component] pub fn AiKnowledgeDocument() -> Element { crud("知识库文档","ai-knowledge-document",vec![Column{key:"title".to_string(),title:"标题".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]) }
#[component] pub fn AiKnowledgeDocumentCreate() -> Element { panel("创建文档","📄","创建文档") }
#[component] pub fn AiKnowledgeDocumentUpdate() -> Element { panel("修改文档","📝","修改文档") }
#[component] pub fn AiKnowledgeRetrieval() -> Element { panel("文档召回测试","🔍","文档召回测试") }
#[component] pub fn AiKnowledgeSegment() -> Element { panel("知识库分段","🧩","知识库分段") }

fn panel(title:&str,icon:&str,desc:&str)->Element{
    let t=title.to_string();let i=icon.to_string();let d=desc.to_string();
    rsx!{div{class:"bg-white rounded-lg shadow-sm border",
        div{class:"px-6 py-4 border-b",h3{class:"text-base font-semibold","{t}"}}
        div{class:"p-6",div{class:"py-12 text-center text-gray-400",
            span{class:"text-4xl block mb-3","{i}"}p{class:"text-lg","{d}"}
        }}
    }}
}

fn crud(title:&str,module:&str,columns:Vec<Column>)->Element{
    let m=module.to_string();let mut data=use_signal(Vec::<Value>::new);let mut total=use_signal(||0i64);
    let mut pg=use_signal(||1i64);let mut loading=use_signal(||true);let mut show=use_signal(||false);
    let mut edit_id=use_signal(||None::<String>);let mut form=use_signal(||json!({}));let mut saving=use_signal(||false);
    let load={let m=m.clone();move||{let m=m.clone();spawn(async move{loading.set(true);if let Ok((l,t))=api::page(&m,pg(),10).await{data.set(l);total.set(t);}loading.set(false);});}};
    use_effect(load);
    let on_save={let m=m.clone();move|_|{let f=form();let m=m.clone();saving.set(true);spawn(async move{let r=if edit_id().is_some(){api::update(&m,f).await}else{api::create(&m,f).await.map(|_|())};saving.set(false);if r.is_ok(){toast("保存成功",ToastKind::Success);show.set(false);pg.set(1);}else{toast(&format!("失败:{}",r.unwrap_err()),ToastKind::Error);}});}};
    rsx!{
        PageHeader{title:title.to_string(),actions:rsx!{button{class:"ant-btn ant-btn-primary",onclick:move|_|{form.set(json!({}));edit_id.set(None);show.set(true);},"+ 新增"}}}
        DataTable{columns:columns.clone(),data:data(),loading:loading(),page:pg(),page_size:10,total:total(),on_page_change:EventHandler::new(move|p:i64|pg.set(p)),on_edit:Some(EventHandler::new(move|id:String|{if let Some(i)=data().iter().find(|v|get_id(v)==id){form.set(i.clone());edit_id.set(Some(id));show.set(true);}})),on_delete:Some(EventHandler::new({let m=m.clone();move|id:String|{let m=m.clone();spawn(async move{let _=api::remove(&m,&id).await;toast("删除成功",ToastKind::Success);});}})),extra_row:None}
        Modal{title:if edit_id().is_some(){"编辑".to_string()}else{"新增".to_string()},visible:show(),width:None,loading:saving(),on_close:EventHandler::new(move|_|show.set(false)),on_save:Some(EventHandler::new(on_save)),children:{let cols=columns.clone();let items:Vec<_>=cols.iter().map(|c|{let k=c.key.clone();let l=c.title.clone();rsx!{FormItem{label:l.clone(),required:false,children:{let k=k.clone();rsx!{input{class:"ant-input",placeholder:"请输入{l}",oninput:move|e|{let mut f=form();f[k.clone()]=json!(e.value());form.set(f);}}}}}}}).collect::<Vec<_>>();rsx!{for item in items{{item}}}}}
    }
}
fn rid(row:&Value)->String{row.get("id").and_then(|v|v.as_i64().map(|n|n.to_string()).or_else(||v.as_str().map(String::from))).unwrap_or_default()}

fn readonly_page(title:&str,module:&str,columns:Vec<Column>)->Element{let m=module.to_string();let mut data=use_signal(Vec::<Value>::new);let mut total=use_signal(||0i64);let mut pg=use_signal(||1i64);let mut loading=use_signal(||true);let load={let m=m.clone();move||{let m=m.clone();spawn(async move{loading.set(true);if let Ok((l,t))=api::page(&m,pg(),10).await{data.set(l);total.set(t);}loading.set(false);});}};use_effect(load);rsx!{PageHeader{title:title.to_string(),actions:None}DataTable{columns,data:data(),loading:loading(),page:pg(),page_size:10,total:total(),on_page_change:EventHandler::new(move|p:i64|pg.set(p)),on_edit:None,on_delete:None,extra_row:None}}}

fn image_page() -> Element {
    let mut prompt=use_signal(String::new);let mut images_sig=use_signal(Vec::<Value>::new);
    let mut generating=use_signal(||false);let mut model_id=use_signal(String::new);
    let mut models=use_signal(Vec::<Value>::new);
    use_effect(move||{spawn(async move{if let Ok(v)=api::action("/ai/model/simple-list",json!({"type":"image"})).await{if let Some(arr)=v["data"].as_array(){models.set(arr.clone());}}});});
    let ml=models();let pv=prompt();let gv=generating();let mv=model_id();let imgs=images_sig();
    let gallery:Vec<Element>=imgs.iter().map(|img|{
        let url=img.get("picUrl").or(img.get("url")).and_then(|v|v.as_str()).unwrap_or("").to_string();
        let pt=img.get("prompt").and_then(|v|v.as_str()).unwrap_or("").to_string();
        rsx!{div{class:"bg-white rounded-lg shadow-sm border overflow-hidden",img{src:"{url}",class:"w-full h-48 object-cover"},div{class:"p-2 text-xs text-gray-500 truncate","{pt}"}}}
    }).collect();
    let has_images=!imgs.is_empty();
    rsx!{div{
        div{class:"bg-white rounded-lg shadow-sm border p-6 mb-6",
            h3{class:"text-base font-semibold mb-4","AI 绘图"}
            div{class:"flex gap-3 mb-3",
                input{class:"ant-input flex-1",placeholder:"描述你想生成的图片...",value:"{pv}",oninput:move|e|prompt.set(e.value())}
                select{class:"ant-input w-[200px]",value:"{mv}",onchange:move|e|model_id.set(e.value()),option{value:"","选择模型"}
                    for m in &ml{{let id=m["id"].as_i64().unwrap_or(0).to_string();let n=m["name"].as_str().unwrap_or("").to_string();rsx!{option{value:"{id}","{n}"}}}}
                }
                button{class:"ant-btn ant-btn-primary",disabled:gv,onclick:move|_|{
                    let p=prompt();if p.is_empty(){return;}generating.set(true);let mid=model_id().parse::<i64>().ok();let mut im=images_sig;
                    spawn(async move{if let Ok(v)=api::action("/ai/model/test",json!({"modelId":mid,"prompt":p,"type":"image","width":1024,"height":1024})).await{if let Some(d)=v["data"].as_object(){im.write().push(json!(d));}}generating.set(false);});
                },if gv{"生成中..."}else{"生成"}}
            }
        }
        if has_images{div{class:"grid grid-cols-3 gap-4",for item in &gallery{{item.clone()}}}}else{div{class:"grid grid-cols-4 gap-4",
            for _ in 0..8{div{class:"bg-white rounded-lg shadow-sm border aspect-square flex items-center justify-center text-gray-200",span{class:"text-4xl","🖼️"}}}
        }}
    }}
}
