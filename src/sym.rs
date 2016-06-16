#![allow(dead_code)]

use builtins;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path;

use opts;

#[derive(PartialEq, Copy, Clone)]
pub enum SymType {
    Binary,
    Builtin,
    Var,
    Environment,
    Fn
}

pub enum Sym {
    Binary(path::PathBuf),
    Builtin(builtins::Builtin),
    Var(String),
    Environment(String),
    Fn(Fn)
}

#[derive(Clone)]
pub struct Fn {
    pub name: String,
    pub inline: bool,
    pub args: Vec<String>,
    pub vararg: Option<String>,
    pub postargs: Option<Vec<String>>,
    pub lines: Vec<String>
}

enum Val {
    Var(String),
    Fn(Fn)
}

// the 'is_fn' flag enables us to check
// whether the current scope is the top-level scope of
// a function -- aside from global ops, we don't want to act
// through the function barrier.
struct Scope {
    vars: HashMap<String, Val>,  // contains vars and also functions
    is_fn: bool,
    is_gl: bool                  // why do we need this?
}

#[derive(PartialEq, Copy, Clone)]
pub enum ScopeSpec {
    Local,
    Global,
    Environment,
    Default
}


// We can use 'static for builtins because that's what builtins are: static.
pub struct Symtable {
    bins:     HashMap<String, path::PathBuf>,
    builtins: HashMap<&'static str, builtins::Builtin>,
    scopes:   Vec<Scope>
}

impl Symtable {
    pub fn new() -> Symtable {
        let mut st = Symtable {
            bins:     HashMap::new(),
            builtins: builtins::Builtin::map(),
            scopes:   Vec::new()
        };

        st.scopes.push(Scope {
            vars: HashMap::new(),
            is_fn: false,
            is_gl: true
        });

        st.hash_bins();

        st
    }
   
    pub fn set(&mut self, key: &str, val: String)
            -> Result<&mut Symtable, opts::OptError> {
        self.set_scope(key, val, ScopeSpec::Default)
    }

    fn scope_idx<'a>(&'a mut self, key: &str, sc: ScopeSpec) -> usize {
        match sc {
            ScopeSpec::Global => 0,
            ScopeSpec::Local => self.scopes.len() - 1,
            ScopeSpec::Environment => { unreachable!() },
            ScopeSpec::Default => {
                let len = self.scopes.len();
                for (idx, scope) in self.scopes.iter_mut().rev().enumerate() {
                    if scope.vars.get(key).is_some() {
                        return len - idx - 1;
                    }
                }
                len - 1
            }
        }
    }

    pub fn set_fn(&mut self, key: &str, val: Fn, sc: ScopeSpec) -> &mut Symtable {
        {
            let idx = self.scope_idx(key, sc);
            let ref mut scope = self.scopes[idx];
            scope.vars.insert(key.to_string(), Val::Fn(val));
        }
        self
    }

    pub fn set_scope(&mut self, key: &str, val: String, sc: ScopeSpec)
            -> Result<&mut Symtable, opts::OptError> {
        if opts::is_opt(key) {
            try!(opts::set(key, val));
            return Ok(self);
        }

        if sc == ScopeSpec::Environment {
            env::set_var(key, val);
            return Ok(self);
        }

        // need to scope this for borrowck
        {
            let idx = self.scope_idx(key, sc);
            let ref mut scope = self.scopes[idx];

            if val == "" {
                scope.vars.remove(key);
            } else {
                scope.vars.insert(key.to_string(), Val::Var(val));
            }
        }

        Ok(self)
    }

    pub fn new_scope(&mut self, is_fn: bool) -> &mut Symtable {
        self.scopes.push(Scope {
            vars: HashMap::new(),
            is_fn: is_fn,
            is_gl: false
        });

        self
    }

    pub fn del_scope(&mut self) -> &mut Symtable {
        // error handling re: a bogus '}' is elsewhere
        self.scopes.pop();

        self
    }

    fn hash_bins(&mut self) -> &mut Symtable {
        self.bins.clear();

        let path_str = env::var("PATH").unwrap_or("/bin:/usr/bin".to_string());
        for path_dir in path_str.split(":") {
            if let Ok(path_dir) = fs::read_dir(path_dir) {
                for path_f in path_dir {
                    if let Err(e) = path_f {
                        println!("Error: {}", e);
                        break;
                    }
                    let path_f = path_f.unwrap();
                    let is_ok_f = match path_f.metadata() {
                        Ok(ent) => !ent.is_dir(),   // FIXME: should only be executable files
                        Err(_) => false
                    };

                    if is_ok_f {
                        if let Some(os_fname) = path_f.path().file_name() {
                            if let Ok(fname) = os_fname.to_os_string().into_string() {
                                self.bins.insert(fname, path_f.path());
                            }
                        }
                    }
                }
            }
        }

        self
    }

    pub fn prefix_resolve(&self, sym_n: &str) -> Vec<String> {
        self.prefix_resolve_types(sym_n, None)
    }

    // simply resolves a vector of matching strings.
    // TODO: rewrite this. it's bad; also we need to add fns to this
    pub fn prefix_resolve_types(&self, sym_n: &str, types: Option<Vec<SymType>>) 
                              -> Vec<String> {
        let types = match types {
            Some(x) => { x },
            None    => vec![SymType::Var,
                            SymType::Binary,
                            SymType::Builtin, 
                            SymType::Environment]
        };

        let mut res: Vec<String> = Vec::new();

        if types.contains(&SymType::Var) {
            // check for Var symbol
            for scope in self.scopes.iter().rev() {
                for v in scope.vars.iter().filter(|&(x, _)| x.starts_with(sym_n)) {
                    if let (_, &Val::Var(ref v)) = v {
                        res.push(v.clone());
                    }
                }
            }
        }

        if types.contains(&SymType::Builtin) {
            // check for Builtin symbol
            for v in self.builtins.iter().filter(|&(x, _)| x.starts_with(sym_n)) {
                res.push(v.1.name.to_string());
            }
        }

        if types.contains(&SymType::Binary) {
            // check for Binary symbol by filename
            for v in self.bins.iter().filter(|&(x, _)| x.starts_with(sym_n)) {
                res.push(v.1.clone().file_name().unwrap()
                            .to_os_string().into_string().unwrap());
            }
        }

        res
    }


    pub fn resolve(&mut self, sym_n: &str) -> Option<Sym> {
        self.resolve_types(sym_n, None)
    }

    pub fn resolve_types(&mut self, sym_n: &str, types: Option<Vec<SymType>>) 
                        -> Option<Sym> {

        let types = match types {
            Some(x) => { x },
            None    => vec![SymType::Var,
                            SymType::Binary,
                            SymType::Builtin, 
                            SymType::Environment,
                            SymType::Fn]
        };

        if types.contains(&SymType::Var) || types.contains(&SymType::Fn) {
            // check for opt
            if opts::is_opt(sym_n) {
                match opts::get(sym_n) {
                    Some(s) => return Some(Sym::Var(s)),
                    None    => return None
                }
            }

            // check for Var symbol
            for scope in self.scopes.iter().rev() {
                if let Some(v) = scope.vars.get(sym_n) {
                    match *v {
                        Val::Var(ref v) => {
                            if types.contains(&SymType::Var) {
                                return Some(Sym::Var(v.clone()));
                            }
                        },
                        Val::Fn(ref f)  => {
                            if types.contains(&SymType::Fn) {
                                return Some(Sym::Fn(f.clone()));
                            }
                        }
                    }
                }
            }
        }

        if types.contains(&SymType::Environment) {
            if let Ok(e) = env::var(sym_n) {
                return Some(Sym::Environment(e));
            }
        }

        if types.contains(&SymType::Builtin) {
            // check for Builtin symbol
            if let Some(bi) = self.builtins.get(sym_n) {
                return Some(Sym::Builtin(bi.clone()));
            }
        }

        if types.contains(&SymType::Binary) {
            // check for Binary symbol by filename
            if let Some(bin_path) = self.bins.get(sym_n) {
                return Some(Sym::Binary(bin_path.clone()));
            }

            // Check for executable file by full path
            if let Ok(_) = fs::metadata(sym_n) {
                // FIXME: needs more sanity checking for good files
                return Some(Sym::Binary(path::PathBuf::from(sym_n)));
            }

            // Re-hash bins and check again
            // TODO: make re-hash optional, since it has a noticeable runtime.
            if let Some(bin_path) = self.hash_bins().bins.get(sym_n) {
                return Some(Sym::Binary(bin_path.clone()));
            }
        }

        None
    }
}
