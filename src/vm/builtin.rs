use super::value::*;
use crate::loader::*;
use crate::vm::*;
use rand;

pub struct Builtin {}

impl Builtin {
    pub fn init_builtin(globals: &mut Globals) {
        globals.add_builtin_method("puts", builtin_puts);
        globals.add_builtin_method("p", builtin_p);
        globals.add_builtin_method("print", builtin_print);
        globals.add_builtin_method("assert", builtin_assert);
        globals.add_builtin_method("require", builtin_require);
        globals.add_builtin_method("require_relative", builtin_require_relative);
        globals.add_builtin_method("block_given?", builtin_block_given);
        globals.add_builtin_method("method", builtin_method);
        globals.add_builtin_method("is_a?", builtin_isa);
        globals.add_builtin_method("to_s", builtin_tos);
        globals.add_builtin_method("Integer", builtin_integer);
        globals.add_builtin_method("__dir__", builtin_dir);
        globals.add_builtin_method("raise", builtin_raise);
        globals.add_builtin_method("rand", builtin_rand);

        /// Built-in function "puts".
        fn builtin_puts(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            fn flatten(vm: &VM, val: PackedValue) {
                match val.as_array() {
                    None => println!("{}", vm.val_to_s(val)),
                    Some(aref) => {
                        for val in &aref.elements {
                            flatten(vm, val.clone());
                        }
                    }
                }
            }
            for i in 0..args.len() {
                flatten(vm, args[i]);
            }
            Ok(PackedValue::nil())
        }

        fn builtin_p(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            for i in 0..args.len() {
                println!("{}", vm.val_pp(args[i]));
            }
            if args.len() == 1 {
                Ok(args[0])
            } else {
                Ok(PackedValue::array_from(
                    &vm.globals,
                    args.get_slice(0, args.len()).to_vec(),
                ))
            }
        }

        /// Built-in function "print".
        fn builtin_print(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            for i in 0..args.len() {
                if let Value::Char(ch) = args[i].unpack() {
                    let v = [ch];
                    use std::io::{self, Write};
                    io::stdout().write(&v).unwrap();
                } else {
                    print!("{}", vm.val_to_s(args[i].clone()));
                }
            }
            Ok(PackedValue::nil())
        }

        /// Built-in function "assert".
        fn builtin_assert(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 2, 2)?;
            if !args[0].equal(args[1]) {
                panic!(
                    "Assertion error: Expected: {} Actual: {}",
                    vm.val_pp(args[0]),
                    vm.val_pp(args[1]),
                );
            } else {
                println!("Assert OK: {:?}", vm.val_pp(args[0]));
                Ok(PackedValue::nil())
            }
        }

        fn builtin_require(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 1, 1)?;
            let file_name = match args[0].as_string() {
                Some(string) => string,
                None => return Err(vm.error_argument("file name must be a string.")),
            };
            let mut path = std::env::current_dir().unwrap();
            path.push(file_name);
            require(vm, path)?;
            Ok(PackedValue::bool(true))
        }

        fn builtin_require_relative(
            vm: &mut VM,
            args: &Args,
            _block: Option<MethodRef>,
        ) -> VMResult {
            vm.check_args_num(args.len(), 1, 1)?;
            let mut path = vm.root_path.last().unwrap().clone();

            let file_name = match args[0].as_string() {
                Some(string) => PathBuf::from(string),
                None => return Err(vm.error_argument("file name must be a string.")),
            };
            path.pop();
            //path.push(file_name);
            for p in file_name.iter() {
                if p == ".." {
                    path.pop();
                } else {
                    path.push(p);
                }
            }
            path.set_extension("rb");
            require(vm, path)?;
            Ok(PackedValue::bool(true))
        }

        fn require(vm: &mut VM, path: PathBuf) -> Result<(), RubyError> {
            let file_name = path.to_string_lossy().to_string();
            let (absolute_path, program) = match load_file(file_name.clone()) {
                Ok((path, program)) => (path, program),
                Err(err) => {
                    match err {
                        LoadError::NotFound(msg) => {
                            eprintln!("No such file or directory --- {} (LoadError)", &file_name);
                            eprintln!("{}", msg);
                        }
                        LoadError::CouldntOpen(msg) => {
                            eprintln!("Cannot open file. '{}'", &file_name);
                            eprintln!("{}", msg);
                        }
                    }
                    return Err(vm.error_internal("LoadError"));
                }
            };
            #[cfg(feature = "verbose")]
            eprintln!("reading:{}", absolute_path.to_string_lossy());
            vm.root_path.push(path);
            vm.class_push(vm.globals.object);
            vm.run(absolute_path.to_str().unwrap(), program)?;
            vm.class_pop();
            vm.root_path.pop().unwrap();
            Ok(())
        }

        /// Built-in function "block_given?".
        fn builtin_block_given(vm: &mut VM, _args: &Args, _block: Option<MethodRef>) -> VMResult {
            Ok(PackedValue::bool(vm.context().block.is_some()))
        }

        fn builtin_method(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 1, 1)?;
            let name = match args[0].as_symbol() {
                Some(id) => id,
                None => return Err(vm.error_type("An argument must be a Symbol.")),
            };
            let recv_class = args.self_value.get_class_object_for_method(&vm.globals);
            let method = vm.get_instance_method(recv_class, name)?;
            let val = PackedValue::method(&vm.globals, name, args.self_value, method);
            Ok(val)
        }

        fn builtin_isa(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 1, 1)?;
            let mut recv_class = args.self_value.get_class_object(&vm.globals);
            loop {
                if recv_class.id() == args[0].id() {
                    return Ok(PackedValue::true_val());
                }
                recv_class = recv_class.as_class().superclass;
                if recv_class.is_nil() {
                    return Ok(PackedValue::false_val());
                }
            }
        }

        fn builtin_tos(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 0, 0)?;
            let s = vm.val_to_s(args.self_value);
            Ok(PackedValue::string(s))
        }

        fn builtin_integer(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 1, 1)?;
            let self_ = args[0];
            let val = if self_.is_packed_value() {
                if self_.is_packed_fixnum() {
                    self_.as_packed_fixnum()
                } else if self_.is_packed_num() {
                    self_.as_packed_flonum().trunc() as i64
                } else {
                    return Err(vm.error_type(format!(
                        "Can not convert {} into Integer.",
                        vm.val_pp(self_)
                    )));
                }
            } else {
                match self_.unpack() {
                    Value::FixNum(num) => num,
                    Value::FloatNum(num) => num as i64,
                    Value::String(s) => match s.parse::<i64>() {
                        Ok(num) => num,
                        Err(_) => {
                            return Err(vm.error_type(format!(
                                "Invalid value for Integer(): {}",
                                vm.val_pp(self_)
                            )))
                        }
                    },
                    _ => {
                        return Err(vm.error_type(format!(
                            "Can not convert {} into Integer.",
                            vm.val_pp(self_)
                        )))
                    }
                }
            };
            Ok(PackedValue::fixnum(val))
        }

        fn builtin_dir(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 0, 0)?;
            let mut path = vm.root_path.last().unwrap().clone();
            path.pop();
            Ok(PackedValue::string(path.to_string_lossy().to_string()))
        }

        fn builtin_raise(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
            vm.check_args_num(args.len(), 0, 2)?;
            for i in 0..args.len() {
                eprintln!("{}", vm.val_pp(args[i]));
            }
            Err(vm.error_unimplemented("error"))
        }

        fn builtin_rand(_vm: &mut VM, _args: &Args, _block: Option<MethodRef>) -> VMResult {
            let num = rand::random();
            Ok(PackedValue::flonum(num))
        }
    }
}
