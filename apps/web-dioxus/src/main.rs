#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::router::*;

mod components; mod layouts; mod pages; mod services;

use layouts::{MainLayout, AuthLayout};
use pages::_core::*;
use pages::dashboard::*;
use pages::system::*;
use pages::infra::*;
use pages::infra::asset::*;
use pages::ai::*;
use pages::cloud::*;
use pages::provider::*;
use pages::room::*;
use pages::security::*;
use pages::zone::*;
use pages::ticket::*;
use pages::task::*;
use pages::risk::*;
use pages::business::*;
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AuthLayout)]
        #[route("/auth/login")]        Login {},
        #[route("/auth/register")]     Register {},
        #[route("/auth/forgot")]       ForgotPassword {},
    #[end_layout]
    #[layout(MainLayout)]
        #[route("/")]                       Dashboard {},
        #[route("/dashboard/workspace")]    DashboardWorkspace {},
        #[route("/dashboard/analytics")]   DashboardAnalytics {},
        // System (29 routes)
        #[route("/system/user")]            SystemUser {},
        #[route("/system/role")]            SystemRole {},
        #[route("/system/menu")]            SystemMenu {},
        #[route("/system/dept")]            SystemDept {},
        #[route("/system/post")]            SystemPost {},
        #[route("/system/dict")]            SystemDict {},
        #[route("/system/tenant")]          SystemTenant {},
        #[route("/system/tenant-package")]  SystemTenantPackage {},
        #[route("/system/operate-log")]     SystemOperateLog {},
        #[route("/system/login-log")]       SystemLoginLog {},
        #[route("/system/mail-account")]    SystemMailAccount {},
        #[route("/system/mail-template")]   SystemMailTemplate {},
        #[route("/system/mail-log")]        SystemMailLog {},
        #[route("/system/sms-channel")]     SystemSmsChannel {},
        #[route("/system/sms-template")]    SystemSmsTemplate {},
        #[route("/system/sms-log")]         SystemSmsLog {},
        #[route("/system/notice")]          SystemNotice {},
        #[route("/system/notify")]          SystemNotify {},
        #[route("/system/oauth2")]          SystemOauth2 {},
        #[route("/system/social")]          SystemSocial {},
        #[route("/system/profile")]         SystemProfile {},
        #[route("/system/area")]            SystemArea {},
        #[route("/system/social-client")]   SystemSocialClient {},
        #[route("/system/social-user")]     SystemSocialUser {},
        #[route("/system/oauth2-client")]   SystemOauth2Client {},
        #[route("/system/oauth2-token")]    SystemOauth2Token {},
        #[route("/system/notify-template")]    SystemNotifyTemplate {},
        #[route("/system/notify-message-list")] SystemNotifyMessageList {},
        #[route("/system/notify-message")]      SystemNotifyMessageId {},
        // Infra (22 routes)
        #[route("/infra/config")]           InfraConfig {},
        #[route("/infra/datasource")]       InfraDatasource {},
        #[route("/infra/file-config")]      InfraFileConfig {},
        #[route("/infra/file")]             InfraFile {},
        #[route("/infra/job")]              InfraJob {},
        #[route("/infra/job-log")]          InfraJobLog {},
        #[route("/infra/codegen")]          InfraCodegen {},
        #[route("/infra/api-access-log")]   InfraApiAccessLog {},
        #[route("/infra/api-error-log")]    InfraApiErrorLog {},
        #[route("/infra/redis-monitor")]    InfraRedisMonitor {},
        #[route("/infra/server-monitor")]   InfraServerMonitor {},
        #[route("/infra/demo01")]           InfraDemo01 {},
        #[route("/infra/demo02")]           InfraDemo02 {},
        #[route("/infra/demo03")]           InfraDemo03 {},
        #[route("/infra/build")]            InfraBuild {},
        #[route("/infra/rust")]             InfraRust {},
        #[route("/infra/postgresql")]       InfraPostgreSql {},
        #[route("/infra/traces")]           InfraTraces {},
        #[route("/infra/swagger")]          InfraSwagger {},
        #[route("/infra/websocket")]        InfraWebSocket {},
        #[route("/infra/job/log")]          InfraJobLogDetail {},
        #[route("/infra/codegen/edit")]     InfraCodegenEdit {},
        // Asset (11 routes)
        #[route("/asset/provider")]          AssetProvider {},
        #[route("/asset/room")]              AssetRoom {},
        #[route("/asset/cloud")]             AssetCloud {},
        #[route("/asset/zone")]              AssetZone {},
        #[route("/asset/security")]          AssetSecurity {},
        #[route("/asset/list")]              AssetList {},
        #[route("/asset/business-app")]      AssetBusinessApp {},
        #[route("/asset/business-resource")] AssetBusinessResource {},
        #[route("/asset/ticket")]            AssetTicket {},
        #[route("/asset/task")]              AssetTask {},
        #[route("/asset/risk")]              AssetRisk {},
        // Cloud Platform Management
        #[route("/cloud")]                 CloudPlatformPage {},
        #[route("/provider")]              ProviderPage {},
        #[route("/room")]                  RoomPage {},
        #[route("/security")]              SecurityPage {},
        #[route("/zone")]                  ZonePage {},
        #[route("/ticket")]                TicketPage {},
        #[route("/task")]                  TaskPage {},
        #[route("/risk")]                  RiskPage {},
        #[route("/business")]              BusinessAppPage {},
        // AI (10 main nav routes)
        #[route("/ai/chat")]        AiChat {},
        #[route("/ai/image")]       AiImage {},
        #[route("/ai/write")]       AiWrite {},
        #[route("/ai/music")]       AiMusic {},
        #[route("/ai/model")]       AiModel {},
        #[route("/ai/knowledge")]   AiKnowledge {},
        #[route("/ai/tool")]        AiTool {},
        #[route("/ai/chat-role")]   AiChatRole {},
        #[route("/ai/chat/manager")]      AiChatManager {},
        #[route("/ai/image/manager")]     AiImageManager {},
    #[end_layout]
    #[route("/:..route")] NotFound { route: Vec<String> },
}

fn main() { console_log::init_with_level(log::Level::Debug).ok(); dioxus::launch(App); }
#[component] fn App() -> Element { rsx! { Router::<Route> {} } }
#[component] fn NotFound(route: Vec<String>) -> Element { rsx! { div { class: "flex items-center justify-center min-h-screen bg-gray-100", div { class: "text-center", h1 { class: "text-6xl font-bold text-gray-300 mb-4", "404" } p { class: "text-lg text-gray-500", "页面未找到" } } } } }
