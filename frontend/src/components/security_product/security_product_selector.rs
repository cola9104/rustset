use dioxus::prelude::*;
use std::collections::HashMap;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaShieldHalved, FaXmark};

use crate::state::security_product::{
    SecurityProduct, SecurityProductCategory, SecurityProductStatus,
};
use crate::app::SECURITY_PRODUCTS_STATE;

/// 已选择的安全产品（每个分类一个）
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectedSecurityProducts {
    pub products: HashMap<SecurityProductCategory, i32>,
}

impl SelectedSecurityProducts {
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
        }
    }

    pub fn set(&mut self, category: SecurityProductCategory, product_id: i32) {
        self.products.insert(category, product_id);
    }

    pub fn get(&self, category: &SecurityProductCategory) -> Option<i32> {
        self.products.get(category).copied()
    }

    pub fn remove(&mut self, category: &SecurityProductCategory) {
        self.products.remove(category);
    }

    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }
}

/// 安全产品选择器组件属性
#[component]
pub fn SecurityProductSelector(
    /// 已选择的产品
    selected: Signal<SelectedSecurityProducts>,
    /// 可选的分类筛选（为None时显示全部分类）
    categories: Option<Vec<SecurityProductCategory>>,
    /// 是否只显示运行中的产品
    active_only: Option<bool>,
) -> Element {
    let active_only = active_only.unwrap_or(true);
    let display_categories = categories.unwrap_or_else(|| SecurityProductCategory::all_categories());

    let products = SECURITY_PRODUCTS_STATE.read();
    let selected_products = selected.read();

    rsx! {
        div { class: "space-y-4",
            // 已选标签区域
            if !selected_products.is_empty() {
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "已选择的安全产品" }
                    div { class: "flex flex-wrap gap-2",
                        for (category, product_id) in selected_products.products.iter() {
                            {
                                let product_name = products.iter()
                                    .find(|p| p.id == *product_id)
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| format!("产品{}", product_id));
                                let category_name = category.display_name();
                                let cat = *category;
                                rsx! {
                                    span { class: "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-indigo-100 text-indigo-800",
                                        Icon { icon: FaShieldHalved, width: 14, height: 14, class: "mr-1.5" }
                                        "{category_name}: {product_name}"
                                        button {
                                            class: "ml-2 text-indigo-600 hover:text-indigo-900",
                                            onclick: move |_| {
                                                selected.write().remove(&cat);
                                            },
                                            Icon { icon: FaXmark, width: 12, height: 12 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 分类选择区域
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                for category in display_categories.iter() {
                    {
                        let category_products: Vec<&SecurityProduct> = if active_only {
                            products.iter()
                                .filter(|p| p.category == *category && p.status == SecurityProductStatus::Active)
                                .collect()
                        } else {
                            products.iter()
                                .filter(|p| p.category == *category)
                                .collect()
                        };

                        let selected_id = selected_products.get(category);
                        let cat = *category;
                        // 当前选中的值字符串，用于控制select的value
                        let selected_value = selected_id.map(|id| id.to_string()).unwrap_or_default();

                        rsx! {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1",
                                    Icon { icon: FaShieldHalved, width: 14, height: 14, class: "inline mr-1 text-indigo-500" }
                                    "{category.display_name()}"
                                }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 text-sm",
                                    value: "{selected_value}",
                                    onchange: move |e| {
                                        let value = e.value();
                                        if value.is_empty() {
                                            selected.write().remove(&cat);
                                        } else if let Ok(id) = value.parse::<i32>() {
                                            selected.write().set(cat, id);
                                        }
                                    },
                                    option { value: "", "-- 请选择 --" }
                                    for product in category_products.iter() {
                                        {
                                            let is_selected = selected_id == Some(product.id);
                                            rsx! {
                                                option {
                                                    value: "{product.id}",
                                                    selected: is_selected,
                                                    "{product.name} ({product.vendor})"
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

            // 提示信息
            p { class: "text-xs text-gray-500 mt-2",
                "每个分类可选择一个安全产品，已选产品将以标签形式展示在上方"
            }
        }
    }
}
