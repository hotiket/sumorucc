use std::mem::swap;

use super::ctype::{CType, Integer};
use super::node::{Node, NodeKind};

#[derive(Clone)]
pub struct LVar {
    // 変数の名前
    pub name: String,
    // 変数の型
    pub ctype: CType,
    // RBPからのオフセット
    pub offset: usize,
}

#[derive(Clone)]
pub struct GVar {
    // 変数の名前
    pub name: String,
    // 変数の型
    pub ctype: CType,
    // 初期値
    pub val: Option<Vec<Node>>,
}

#[derive(Clone)]
pub struct Str {
    // 文字列を指すラベル
    pub label: String,
    // 文字列の中身
    pub val: Vec<u8>,
}

#[derive(Clone)]
struct Scope {
    child: Option<Box<Self>>,
    lvars: Vec<LVar>,
    structs: Vec<CType>,
}

fn find_struct(structs: &[CType], name: &str) -> Option<CType> {
    structs
        .iter()
        .find(|c| matches!(c, CType::Struct(n, ..) if n.as_ref().map_or(false, |n| n == name)))
        .cloned()
}

fn get_struct_name<'a>(ctype: &'a CType) -> Result<Option<&'a str>, &'static str> {
    if let CType::Struct(name, ..) = ctype {
        if let Some(name) = name.as_ref() {
            Ok(Some(name))
        } else {
            Ok(None)
        }
    } else {
        Err("構造体ではありません")
    }
}

impl Scope {
    fn new() -> Self {
        Self {
            child: None,
            lvars: Vec::new(),
            structs: Vec::new(),
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

    fn add_struct(&mut self, name: &str, ctype: CType) -> Result<(), &str> {
        if let Some(ref mut child) = self.child {
            child.add_struct(name, ctype)
        } else if self.find_current_struct(name).is_some() {
            Err("すでに定義されています")
        } else {
            self.structs.push(ctype);

            Ok(())
        }
    }

    fn find_struct(&self, name: &str) -> Option<CType> {
        if let Some(ref child) = self.child {
            let ctype = child.find_struct(name);
            if ctype.is_some() {
                return ctype;
            }
        }

        self.find_current_struct(name)
    }

    fn find_current_struct(&self, name: &str) -> Option<CType> {
        find_struct(&self.structs, name)
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

#[derive(Clone)]
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

    fn add_struct(&mut self, name: &str, ctype: CType) -> Result<(), &str> {
        self.scope.add_struct(name, ctype)
    }

    fn find_struct(&self, name: &str) -> Option<CType> {
        self.scope.find_struct(name)
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

#[derive(Clone)]
pub struct ParseContext {
    pub funcs: Vec<Function>,
    pub gvars: Vec<GVar>,
    pub structs: Vec<CType>,
    pub strs: Vec<Str>,
    current_fn: Option<String>,
    str_n: usize,
}

impl ParseContext {
    pub fn new() -> Self {
        Self {
            funcs: Vec::new(),
            gvars: Vec::new(),
            structs: Vec::new(),
            strs: Vec::new(),
            current_fn: None,
            str_n: 0,
        }
    }

    // NOTE: selfをまるごとcloneするので
    //       性能上のボトルネックになるかもしれない。
    pub fn save(&self) -> Self {
        self.clone()
    }

    pub fn restore(&mut self, mut ctx: Self) {
        swap(self, &mut ctx);
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

    pub fn add_struct(&mut self, ctype: CType) -> Result<(), &str> {
        let name = match get_struct_name(&ctype) {
            Ok(Some(name)) => name,
            // タグをつけていない構造体は後から
            // 参照できないので登録せずにOkを返す。
            Ok(_) => return Ok(()),
            Err(msg) => return Err(msg),
        };

        if self.current_fn.is_some() {
            let fn_name = self.current_fn.as_ref().unwrap().clone();
            let func = self.find_fn_mut(&fn_name).unwrap();
            func.add_struct(&name.to_string(), ctype)
        } else if self.find_struct(name).is_some() {
            Err("すでに定義されています")
        } else {
            self.structs.push(ctype);
            Ok(())
        }
    }

    pub fn add_str(&mut self, val: Vec<u8>) -> (String, CType) {
        let label = format!(".L__String{}", self.str_n);
        let ret = label.clone();
        let base = Box::new(CType::Integer(Integer::Char));
        let ctype = CType::Array(base, val.len());
        self.strs.push(Str { label, val });

        self.str_n += 1;

        (ret, ctype)
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

    pub fn find_struct(&self, name: &str) -> Option<CType> {
        self.find_lstruct(name).or_else(|| self.find_gstruct(name))
    }

    pub fn find_lstruct(&self, name: &str) -> Option<CType> {
        if let Some(ref fn_name) = self.current_fn {
            let func = self.find_fn(fn_name).unwrap();
            func.find_struct(name)
        } else {
            None
        }
    }

    pub fn find_gstruct(&self, name: &str) -> Option<CType> {
        find_struct(&self.structs, name)
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
