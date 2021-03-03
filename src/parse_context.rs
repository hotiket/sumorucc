use super::ctype::CType;
use super::node::{Node, NodeKind};

pub struct LVar {
    // 変数の名前
    pub name: String,
    // 変数の型
    pub ctype: CType,
    // RBPからのオフセット
    pub offset: usize,
}

pub struct GVar {
    // 変数の名前
    pub name: String,
    // 変数の型
    pub ctype: CType,
    // 初期値
    pub val: Option<Vec<Node>>,
}

struct Scope {
    child: Option<Box<Self>>,
    lvars: Vec<LVar>,
}

impl Scope {
    fn new() -> Self {
        Self {
            child: None,
            lvars: Vec::new(),
        }
    }

    fn add_var(&mut self, name: &str, ctype: CType, offset: usize) -> Result<(), &str> {
        if let Some(ref mut child) = self.child {
            child.add_var(name, ctype, offset)
        } else if self.find_current_var(name).is_some() {
            Err("すでに定義されています")
        } else {
            self.lvars.push(LVar {
                name: name.to_string(),
                ctype,
                offset,
            });

            Ok(())
        }
    }

    fn find_var(&self, name: &str) -> Option<NodeKind> {
        if let Some(ref child) = self.child {
            let lvar = child.find_var(name);
            if lvar.is_some() {
                return lvar;
            }
        }

        self.find_current_var(name)
    }

    fn find_current_var(&self, name: &str) -> Option<NodeKind> {
        self.lvars
            .iter()
            .find(|v| v.name == name)
            .map(|v| NodeKind::LVar(v.name.clone(), v.ctype.clone(), v.offset))
    }

    fn enter(&mut self) {
        if let Some(ref mut child) = self.child {
            child.enter();
        } else {
            self.child = Some(Box::new(Self::new()));
        }
    }

    fn exit(&mut self) -> Result<(), &str> {
        if self.child.is_none() {
            return Err("対応するスコープがありません");
        }

        self.exit_impl();
        Ok(())
    }

    // 戻り値: スコープ削除の可否
    fn exit_impl(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            if !child.exit_impl() {
                self.child = None;
            }
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn debug_print_lvars(&self) {
        self.debug_print_lvars_impl(0);
    }

    #[allow(dead_code)]
    fn debug_print_lvars_impl(&self, depth: usize) {
        eprintln!("{}DEPTH={}", " ".repeat(depth), depth);
        for lvar in self.lvars.iter() {
            eprintln!("{}{} {}", " ".repeat(depth), &lvar.ctype, &lvar.name);
        }

        if let Some(ref child) = self.child {
            child.debug_print_lvars_impl(depth + 1);
        }
    }
}

pub struct Function {
    name: String,
    stack_size: usize,
    scope: Scope,
}

impl Function {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stack_size: 0,
            scope: Scope::new(),
        }
    }

    fn add_var(&mut self, name: &str, ctype: CType) -> Result<(), &str> {
        let offset = self.stack_size + ctype.size();
        let result = self.scope.add_var(name, ctype, offset);

        // 変数の追加に成功したらスタックサイズを更新する
        if result.is_ok() {
            self.stack_size = offset;
        }

        result
    }

    fn find_var(&self, name: &str) -> Option<NodeKind> {
        self.scope.find_var(name)
    }

    fn enter(&mut self) {
        self.scope.enter();
    }

    fn exit(&mut self) -> Result<(), &str> {
        self.scope.exit()
    }

    #[allow(dead_code)]
    fn debug_print_lvars(&self) {
        self.scope.debug_print_lvars();
    }
}

pub struct ParseContext {
    pub funcs: Vec<Function>,
    pub gvars: Vec<GVar>,
    current_fn: Option<String>,
}

impl ParseContext {
    pub fn new() -> Self {
        Self {
            funcs: Vec::new(),
            gvars: Vec::new(),
            current_fn: None,
        }
    }

    pub fn stack_size(&self, name: &str) -> Option<usize> {
        self.find_fn(name).map(|func| func.stack_size)
    }

    pub fn enter_fn(&mut self, name: &str) -> Result<(), &str> {
        if self.current_fn.is_some() {
            return Err("関数内での関数定義です");
        }

        if self.find_fn(name).is_none() && self.find_gvar(name).is_none() {
            self.funcs.push(Function::new(name));
            self.current_fn = Some(name.to_string());
            Ok(())
        } else {
            Err("すでに定義されています")
        }
    }

    pub fn exit_fn(&mut self) -> Result<(), &str> {
        if self.current_fn.is_none() {
            return Err("関数定義がされていません");
        }

        self.current_fn = None;
        Ok(())
    }

    pub fn find_fn(&self, name: &str) -> Option<&Function> {
        self.funcs.iter().find(|f| f.name == name)
    }

    pub fn find_fn_mut(&mut self, name: &str) -> Option<&mut Function> {
        self.funcs.iter_mut().find(|f| f.name == name)
    }

    pub fn add_var(&mut self, name: &str, ctype: CType) -> Result<(), &str> {
        if self.current_fn.is_some() {
            // selfの再借用にならないよう処理中の関数名をクローンを作成する
            let fn_name = self.current_fn.as_ref().unwrap().clone();
            let func = self.find_fn_mut(&fn_name).unwrap();
            func.add_var(name, ctype)
        } else if self.find_gvar(name).is_some() || self.find_fn(name).is_some() {
            Err("すでに定義されています")
        } else {
            self.gvars.push(GVar {
                name: name.to_string(),
                ctype,
                val: None,
            });
            Ok(())
        }
    }

    pub fn find_var(&self, name: &str) -> Option<NodeKind> {
        self.find_lvar(name).or_else(|| self.find_gvar(name))
    }

    pub fn find_lvar(&self, name: &str) -> Option<NodeKind> {
        if let Some(ref fn_name) = self.current_fn {
            let func = self.find_fn(fn_name).unwrap();
            func.find_var(name)
        } else {
            None
        }
    }

    pub fn find_gvar(&self, name: &str) -> Option<NodeKind> {
        self.gvars
            .iter()
            .find(|v| v.name == name)
            .map(|v| NodeKind::GVar(v.name.clone(), v.ctype.clone()))
    }

    pub fn set_val(&mut self, name: &str, val: Vec<Node>) -> Result<(), &str> {
        if let Some(gvar) = self.gvars.iter_mut().find(|v| v.name == name) {
            gvar.val = Some(val);
            Ok(())
        } else {
            Err("変数定義がされていません")
        }
    }

    pub fn enter_scope(&mut self) -> Result<(), &str> {
        if self.current_fn.is_none() {
            return Err("関数定義がされていません");
        }

        let fn_name = self.current_fn.as_ref().unwrap().clone();
        let func = self.find_fn_mut(&fn_name).unwrap();
        func.enter();
        Ok(())
    }

    pub fn exit_scope(&mut self) -> Result<(), &str> {
        if self.current_fn.is_none() {
            return Err("関数定義がされていません");
        }

        let fn_name = self.current_fn.as_ref().unwrap().clone();
        let func = self.find_fn_mut(&fn_name).unwrap();
        func.exit()
    }

    #[allow(dead_code)]
    fn debug_print_lvars(&self) {
        self.find_fn(&self.current_fn.as_ref().unwrap())
            .unwrap()
            .debug_print_lvars();
    }
}
