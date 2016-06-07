use std::collections::HashMap;
use std::process::exit;
use std::rc;
use std::fs;
use std::env;

use std::io;
use std::io::BufReader;
use std::io::BufRead;

use sym;
use exec;
use parser;
use opts;
use prompt::LineState;

use exec::Arg;
use shell::Shell;

#[derive(Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub desc: &'static str,
    pub run:  rc::Rc<Fn(Vec<Arg>, &mut Shell, Option<BufReader<fs::File>>) -> i32>
}

// TODO:
//  - elementary flow control
//  - hash
//  - __blank
//  - __fn_exec

fn blank_builtin() -> Builtin {
    Builtin {
        name: "__blank",
        desc: "The blank builtin",
        run: rc::Rc::new(|args: Vec<Arg>, sh: &mut Shell,
                            _in: Option<BufReader<fs::File>>| -> i32 {
            let mut c = 0;
            // do nothing
            for a in args {
                c = match a {
                    Arg::Bl(bv) => {
                        exec::block_exec(sh, bv);
                        opts::status_code()
                    }
                    _ => { c } // TODO: how to properly deal with this?
                };
            }

            c
        })
    }
}

impl Default for Builtin {
    fn default() -> Self {
        blank_builtin()
    }
}

impl Builtin {
    pub fn map() -> HashMap<&'static str, Self> {
        let mut bi_map = HashMap::new();

        bi_map.insert(
            "ifx",
            Builtin {
                name: "ifx",
                desc: "Execute a block depending on the status code of a given statement",
                run: rc::Rc::new(|mut args: Vec<Arg>, sh: &mut Shell,
                                 _in: Option<BufReader<fs::File>>| -> i32 {
                    if args.len() == 0 {
                        warn!("Invalid number of arguments");
                        return 3;
                    }

                    if let Arg::Bl(lv) = args.pop().unwrap() {
                        exec::line_exec(sh, args);
                        let sc = opts::status_code();

                        if sc == 0 {
                            exec::block_exec(sh, lv);
                            opts::status_code()
                        } else {
                            sc
                        }
                    } else {
                        warn!("Must provide an executable block");
                        3
                    }
                })
            });

        bi_map.insert(
            "eval",
            Builtin {
                name: "eval",
                desc: "Evaluate a passed-in statement",
                run: rc::Rc::new(|args: Vec<Arg>, sh: &mut Shell,
                                    _in: Option<BufReader<fs::File>>| -> i32 {
                    exec::line_exec(sh, args);
                    opts::status_code()
                })
            });

        bi_map.insert(
            "set",
            Builtin {
                name: "set",
                desc: "Set a variable binding",
                run: rc::Rc::new(|args: Vec<Arg>, sh: &mut Shell, 
                                 _in: Option<BufReader<fs::File>>| -> i32 {
                    if args.len() < 2 {
                        warn!("set: insufficient arguments.");
                        return 2;
                    }
                    let mut key = String::new();
                    let mut val = String::new();
                    let mut spec = sym::ScopeSpec::Default;
                    let mut phase: u8 = 0;

                    let args = exec::downconvert(args);

                    for arg in args {
                        if phase == 0 && arg.starts_with("-") {
                            for c in arg.chars().skip(1) {
                                match c {
                                    'l' | 'g' | 'e' => {
                                        if spec != sym::ScopeSpec::Default {
                                            warn!("set: Multiple settings for \
                                                       binding specified");
                                        }
                                        debug!("set: Using '{}' for var", c);
                                        spec = match c {
                                            'l' => sym::ScopeSpec::Local,
                                            'g' => sym::ScopeSpec::Global,
                                            'e' => sym::ScopeSpec::Environment,
                                            _   => { unreachable!() }
                                        };
                                    },
                                    _   => {
                                        warn!("set: Unrecognized \
                                                  argument '{}' found",
                                                  c);
                                    }
                                }
                            }
                            continue;
                        } else if phase == 0 {
                            phase = 1;
                        }

                        if phase == 1 && arg == "=" {
                            phase = 2;
                            continue;
                        } else if phase == 1 {
                            if !key.is_empty() { key.push(' '); }
                            key.push_str(&arg);
                        } else {
                            if !val.is_empty() { val.push(' '); }
                            val.push_str(&arg);
                        }
                    }

                    if phase != 2 {
                        warn!("set: Malformed syntax (no '=')");
                    } else {
                        sh.st.set_scope(&key, val, spec);
                    }

                    0
                })
            });

        bi_map.insert(
            "cd",
            Builtin {
                name: "cd",
                desc: "Change directory",
                run: rc::Rc::new(|args: Vec<Arg>, _sh: &mut Shell,
                                 _in: Option<BufReader<fs::File>>| -> i32 {
                    // TODO: more smartly handle the case HOME is nothing?
                    if args.len() == 0 {
                        let home = match env::var("HOME") {
                            Ok(hm)  => hm,
                            Err(_)  => {
                                warn!("cd: no HOME environment variable found.");
                                return 2; /* TODO: correct error code */
                            }
                        };
                        match env::set_current_dir (home.clone()) {
                            Ok(_) => env::set_var("PWD", home),
                            Err(e) => {
                                warn!("cd: {}", e);
                                return 2;
                            }
                        };
                    } else {
                        let args = exec::downconvert(args);
                        let dest = match fs::canonicalize(args[0].clone()) {
                            Ok(pt) => pt,
                            Err(e) => {
                                warn!("cd: {}", e);
                                return 2;
                            }
                        }.into_os_string().into_string().unwrap();
                        match env::set_current_dir (dest.clone()) {
                            Ok(_) => env::set_var("PWD", dest),
                            Err(e) => {
                                warn!("cd: {}", e);
                                return 2;
                            }
                        };
                    }

                    0
                })
            });

        bi_map.insert(
            "exit",
            Builtin {
                name: "exit",
                desc: "Exit the tin shell",
                run: rc::Rc::new(|args: Vec<Arg>, _sh: &mut Shell,
                                 _in: Option<BufReader<fs::File>>| -> i32 {
                    if args.len() == 0 {
                        exit(0);
                    }

                    let args = exec::downconvert(args);
                    
                    match args[0].parse::<i32>() {
                        Ok(i) => exit(i),
                        Err(_) => {
                            warn!("exit: numeric argument required.");
                            exit(2)
                        }
                    }
                })
            });

        bi_map.insert(
            "history",
            Builtin {
                name: "history",
                desc: "List/control history",
                run: rc::Rc::new(|_args: Vec<Arg>, sh: &mut Shell,
                                 _in: Option<BufReader<fs::File>>| -> i32 {
                    sh.ht.hist_print();
                    0
                })
            });
        bi_map.insert(
            "read",
            Builtin {
                name: "read",
                desc: "Read from stdin or a file and echo to stdout",
                run: rc::Rc::new(|_args: Vec<Arg>, _sh: &mut Shell,
                                 inp: Option<BufReader<fs::File>>| -> i32 {
                    let mut in_buf = String::new();
                    let res = if let Some(mut br) = inp {
                        br.read_line(&mut in_buf)
                    } else {
                        io::stdin().read_line(&mut in_buf)
                    };
                    if res.is_ok() {
                        print!("{}", in_buf);
                        0
                    } else {
                        2
                    }
                })
            });

        bi_map
    }
}
