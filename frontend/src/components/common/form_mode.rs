/// 表单模式枚举 - 用于区分新增和编辑操作
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FormMode {
    /// 新增模式
    New,
    /// 编辑模式
    Edit,
}

impl FormMode {
    /// 获取表单标题
    pub fn title(&self, module_name: &str) -> String {
        match self {
            FormMode::New => format!("新增{}", module_name),
            FormMode::Edit => format!("编辑{}", module_name),
        }
    }

    /// 获取保存按钮文本
    pub fn save_text(&self) -> &'static str {
        match self {
            FormMode::New => "添加",
            FormMode::Edit => "保存",
        }
    }

    /// 判断是否为新增模式
    pub fn is_new(&self) -> bool {
        matches!(self, FormMode::New)
    }

    /// 判断是否为编辑模式
    pub fn is_edit(&self) -> bool {
        matches!(self, FormMode::Edit)
    }
}
