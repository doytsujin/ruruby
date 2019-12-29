use crate::util::*;
use crate::vm::*;
use std::collections::HashMap;

pub type ClassRef = Ref<ClassInfo>;

impl ClassRef {
    pub fn from_no_superclass(id: IdentId) -> Self {
        ClassRef::new(ClassInfo::new(id, None))
    }

    pub fn from(id: IdentId, superclass: ClassRef) -> Self {
        ClassRef::new(ClassInfo::new(id, Some(superclass)))
    }

    pub fn get_class_method(&self, id: IdentId) -> Option<&MethodRef> {
        self.class_method.get(&id)
    }

    pub fn get_instance_method(&self, id: IdentId) -> Option<&MethodRef> {
        self.instance_method.get(&id)
    }
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub id: IdentId,
    pub instance_method: MethodTable,
    pub class_method: MethodTable,
    pub constants: ValueTable,
    pub superclass: Option<ClassRef>,
    //pub subclass: HashMap<IdentId, ClassRef>,
}

impl ClassInfo {
    pub fn new(id: IdentId, superclass: Option<ClassRef>) -> Self {
        ClassInfo {
            id,
            instance_method: HashMap::new(),
            class_method: HashMap::new(),
            constants: HashMap::new(),
            superclass,
            //subclass: HashMap::new(),
        }
    }
}

pub fn init_class(globals: &mut Globals) -> ClassRef {
    let class_id = globals.get_ident_id("Class");
    let class = ClassRef::from(class_id, globals.module_class);
    globals.add_builtin_instance_method(class, "new", class_new);
    class
}

/// Built-in function "new".
fn class_new(
    vm: &mut VM,
    receiver: PackedValue,
    args: VecArray,
    _block: Option<MethodRef>,
) -> VMResult {
    match receiver.as_class() {
        Some(class_ref) => {
            let instance = ObjectRef::from(class_ref);
            let new_instance = PackedValue::object(instance);
            // call initialize method.
            if let Some(methodref) = class_ref.get_instance_method(IdentId::INITIALIZE) {
                let iseq = vm.globals.get_method_info(*methodref).as_iseq(&vm)?;
                vm.vm_run(new_instance, iseq, None, args, None)?;
                vm.exec_stack.pop().unwrap();
            };
            Ok(new_instance)
        }
        None => Err(vm.error_unimplemented(format!("Receiver must be a class! {:?}", receiver))),
    }
}
