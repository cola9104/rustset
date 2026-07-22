use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn Dashboard() -> Element {
    rsx!{ div{
        h1{ class:"text-2xl font-bold text-gray-800 mb-6","仪表板"}
        // Stats row
        div{ class:"grid grid-cols-4 gap-4 mb-6",
            StatCard{title:"总资产".to_string(),value:"1,234".to_string(),icon:"📦".to_string(),color:"border-blue-200 bg-blue-50".to_string()}
            StatCard{title:"云资产".to_string(),value:"856".to_string(),icon:"☁️".to_string(),color:"border-purple-200 bg-purple-50".to_string()}
            StatCard{title:"待处理工单".to_string(),value:"23".to_string(),icon:"📝".to_string(),color:"border-orange-200 bg-orange-50".to_string()}
            StatCard{title:"未解决风险".to_string(),value:"45".to_string(),icon:"⚠️".to_string(),color:"border-red-200 bg-red-50".to_string()}
        }
        // Main content: charts + quick nav
        div{ class:"grid grid-cols-3 gap-6",
            // Chart area (2/3)
            div{ class:"col-span-2",
                div{ class:"bg-white rounded-lg shadow-sm border p-6 mb-6",
                    h3{ class:"text-base font-semibold mb-4","资产增长趋势" }
                    div{ class:"flex items-end gap-2 h-[200px]",
                        for (h,label) in [(40,"1月"),(65,"2月"),(55,"3月"),(80,"4月"),(70,"5月"),(90,"6月"),(100,"7月"),(85,"8月"),(95,"9月"),(75,"10月"),(88,"11月"),(92,"12月")] {
                            div{ class:"flex-1 flex flex-col items-center gap-1",
                                div{ class:"w-full bg-primary-400 rounded-t opacity-70 hover:opacity-100 transition-opacity", style:"height:{h}px" }
                                span{ class:"text-xs text-gray-400","{label}" }
                            }
                        }
                    }
                }
                div{ class:"grid grid-cols-2 gap-4",
                    div{ class:"bg-white rounded-lg shadow-sm border p-6",
                        h3{ class:"text-sm font-medium text-gray-500 mb-3","近期工单" }
                        for (name,status,time) in RECENT_TICKETS {
                            div{ class:"flex items-center justify-between py-2 border-b last:border-0 text-sm",
                                span{ class:"text-gray-700","{name}" }
                                span{ class:"ant-tag {status}","{time}" }
                            }
                        }
                    }
                    div{ class:"bg-white rounded-lg shadow-sm border p-6",
                        h3{ class:"text-sm font-medium text-gray-500 mb-3","最新风险" }
                        for (name,severity,time) in RECENT_RISKS {
                            div{ class:"flex items-center justify-between py-2 border-b last:border-0 text-sm",
                                span{ class:"text-gray-700","{name}" }
                                span{ class:"ant-tag {severity}","{time}" }
                            }
                        }
                    }
                }
            }
            // Right sidebar
            div{ class:"col-span-1",
                div{ class:"bg-white rounded-lg shadow-sm border",
                    div{ class:"flex items-center justify-between px-6 py-4 border-b",
                        h3{ class:"text-base font-semibold","快速导航" }
                    }
                    div{ class:"p-4",
                        div{ class:"grid grid-cols-2 gap-2",
                            for (label,path,icon) in NAV {
                                a{ href:"{path}",class:"flex flex-col items-center gap-1.5 p-3 rounded-lg border hover:border-primary-400 hover:bg-primary-50 transition-colors text-center cursor-pointer no-underline",
                                    span{ class:"text-xl","{icon}"}
                                    span{ class:"text-xs text-gray-600","{label}"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }}
}

#[component] pub fn DashboardWorkspace() -> Element {
    rsx!{ div{
        h1{ class:"text-xl font-bold text-gray-800 mb-4","工作台"}
        div{ class:"grid grid-cols-3 gap-4",
            div{ class:"col-span-2 bg-white rounded-lg shadow-sm border p-6" }
            div{ class:"col-span-1 bg-white rounded-lg shadow-sm border p-6",
                h3{ class:"text-base font-semibold mb-3","待办事项"}
                for (i,title) in WORKSPACE_TODOS.iter().enumerate() {
                    div{ class:"flex items-center gap-2 py-2 text-sm text-gray-600 border-b last:border-0",
                        span{ class:"text-gray-400 text-xs","{i+1}"}
                        "{title}"
                    }
                }
            }
        }
    }}
}

#[component] pub fn DashboardAnalytics() -> Element {
    rsx!{ div{
        h1{ class:"text-xl font-bold text-gray-800 mb-4","分析页"}
        div{ class:"grid grid-cols-4 gap-4 mb-6",
            StatCard{title:"今日访问".to_string(),value:"2,458".to_string(),icon:"👁️".to_string(),color:"border-blue-200 bg-blue-50".to_string()}
            StatCard{title:"活跃用户".to_string(),value:"1,023".to_string(),icon:"👥".to_string(),color:"border-green-200 bg-green-50".to_string()}
            StatCard{title:"转化率".to_string(),value:"12.5%".to_string(),icon:"📈".to_string(),color:"border-purple-200 bg-purple-50".to_string()}
            StatCard{title:"平均时长".to_string(),value:"8m32s".to_string(),icon:"⏱️".to_string(),color:"border-orange-200 bg-orange-50".to_string()}
        }
        div{ class:"grid grid-cols-2 gap-4",
            div{ class:"bg-white rounded-lg shadow-sm border p-6",
                h3{ class:"text-base font-semibold mb-4","访问趋势" }
                {line_chart()}
            }
            div{ class:"bg-white rounded-lg shadow-sm border p-6",
                h3{ class:"text-base font-semibold mb-4","流量分布" }
                {donut_chart()}
            }
        }
    }}
}

const NAV: &[(&str,&str,&str)] = &[
    ("资产","/asset/list","🖥️"),("业务应用","/asset/business-app","📱"),
    ("工单","/asset/ticket","📝"),("任务","/asset/task","⚡"),
    ("风险","/asset/risk","⚠️"),("云平台","/asset/cloud","☁️"),
    ("服务商","/asset/provider","🏢"),("机房","/asset/room","🏗️"),
];

const RECENT_TICKETS: &[(&str,&str,&str)] = &[
    ("新服务器申请","ant-tag-blue","2h前"),("防火墙策略变更","ant-tag-orange","5h前"),
    ("域名备案更新","ant-tag-green","1天前"),("SSL证书续期","ant-tag-green","2天前"),
];

const RECENT_RISKS: &[(&str,&str,&str)] = &[
    ("192.168.1.1 端口暴露","ant-tag-red","刚刚"),("10.0.0.5 弱密码","ant-tag-orange","1h前"),
    ("数据库未授权访问","ant-tag-red","3h前"),
];

const WORKSPACE_TODOS: &[&str] = &[
    "审核新资产入库申请","处理安全扫描报告","更新网络拓扑图","完成月度资产盘点","备份配置文件",
];

// ── SVG Charts (zero dependencies) ──

fn line_chart() -> Element {
    let data = [(0,40),(1,65),(2,55),(3,80),(4,70),(5,90),(6,100),(7,85),(8,95),(9,75),(10,88),(11,92)];
    let w = 600; let h = 200; let pad = 30; let max_val = 110.0;
    let points: Vec<String> = data.iter().enumerate().map(|(i,(x,y))| {
        let px = pad + (x * (w - 2*pad)) / 11; let py = h as f64 - pad as f64 - (*y as f64 / max_val * (h - 2*pad) as f64);
        format!("{:.0},{:.0}", px, py)
    }).collect();
    let polyline = points.join(" ");
    let a=format!("{},{pad} {} {},{} {},{pad}", pad, pad, points.first().unwrap(), points.last().unwrap(), w-pad);
    let xlabels: Vec<_> = ["1月","3月","5月","7月","9月","11月"].iter().enumerate().map(|(i,l)| {
        let x = pad + (i * 2 * (w - 2*pad)) / 11; format!("{x},{}", h-pad+16)
    }).collect();
    rsx!{
        svg{ view_box:"0 0 {w} {h}", class:"w-full",
            // Grid lines
            line{ x1:"{pad}", y1:"{pad}", x2:"{pad}", y2:"{h-pad}", stroke:"#e5e7eb", stroke_width:"1" }
            line{ x1:"{pad}", y1:"{h-pad}", x2:"{w-pad}", y2:"{h-pad}", stroke:"#e5e7eb", stroke_width:"1" }
            for py in [40.0,76.0,113.0,149.0] {
                line{ x1:"{pad}", y1:"{py}", x2:"{w-pad}", y2:"{py}", stroke:"#f3f4f6", stroke_width:"1" }
            }
            // Area fill
            polygon{ points:"{polyline} {a}", fill:"url(#grad)", opacity:"0.15" }
            // Line
            polyline{ points:"{polyline}", fill:"none", stroke:"#4f46e5", stroke_width:"2.5", stroke_linejoin:"round" }
            // Dots
            for pt in &points {
                circle{ cx:"{pt.split(',').next().unwrap()}", cy:"{pt.split(',').last().unwrap()}", r:"3.5", fill:"#4f46e5", stroke:"white", stroke_width:"2" }
            }
            // Gradient
            defs{ linearGradient{ id:"grad", x1:"0", x2:"0", y1:"0", y2:"1",
                stop{ offset:"0%", stop_color:"#4f46e5", stop_opacity:"0.4" }
                stop{ offset:"100%", stop_color:"#4f46e5", stop_opacity:"0" }
            }}
        }
    }
}

fn donut_chart() -> Element {
    // Compute SVG arcs for donut segments
    let segments: Vec<(&str,f64,&str)> = vec![("服务器",35.0,"#4f46e5"),("云平台",25.0,"#818cf8"),("网络",20.0,"#c7d2fe"),("安全",12.0,"#e0e7ff"),("其他",8.0,"#f3f4f6")];
    let mut cumulative: f64 = 0.0; let r = 60; let cx = 100; let cy = 100;
    let arcs: Vec<_> = segments.iter().map(|(lbl,pct,color)| {
        let start_angle = (cumulative / 100.0) * 360.0 - 90.0; cumulative += pct;
        let end_angle = (cumulative / 100.0) * 360.0 - 90.0;
        let (sx,sy) = (cx+(r as f64*start_angle.to_radians().cos()) as i32, cy+(r as f64*start_angle.to_radians().sin()) as i32);
        let (ex,ey) = (cx+(r as f64*end_angle.to_radians().cos()) as i32, cy+(r as f64*end_angle.to_radians().sin()) as i32);
        let large = if *pct > 50.0 {1} else {0};
        let d = format!("M {cx} {cy} L {sx} {sy} A {r} {r} 0 {large} 1 {ex} {ey} Z");
        (*lbl,*pct,*color,d)
    }).collect();
    rsx!{
        div{ class:"flex flex-col items-center",
            svg{ view_box:"0 0 200 160", class:"w-48 h-36",
                for (_,_,color,d) in &arcs {
                    path{ d:"{d}", fill:"{color}", stroke:"white", stroke_width:"2" }
                }
                circle{ cx:"{cx}", cy:"{cy}", r:"35", fill:"white" }
                text{ x:"{cx}", y:"{cy+4}", text_anchor:"middle", class:"font-bold text-lg fill-gray-700","100%" }
            }
            div{ class:"grid grid-cols-2 gap-x-4 gap-y-1 mt-2 text-xs",
                for (lbl,pct,color,_) in &arcs {
                    div{ class:"flex items-center gap-1",
                        span{ class:"w-2.5 h-2.5 rounded-full inline-block", style:"background:{color}" }
                        span{ class:"text-gray-600","{lbl}" }
                        span{ class:"text-gray-400 ml-auto","{pct}%" }
                    }
                }
            }
        }
    }
}
