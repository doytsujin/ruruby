# ruruby ![](https://github.com/sisshiki1969/ruruby/workflows/Rust/badge.svg)
A toy Ruby implementation by Rust.

## Related article
[Qiita: Rustでつくる（つくれるかもしれない）Ruby](https://qiita.com/sisshiki1969/items/3d25aa81a376eee2e7c2)

## Implementation status
- Literals
    - [x] Bool
    - [x] Integer
    - [x] Float
    - [x] String literal
    - [x] String literal with interpolation
    - [x] Array literal
    - [x] Hash literal
    - [x] Lambda literal
    - [x] Block literal
- Objects
    - [x] Array
    - [x] Hash
    - [x] Proc
- Variables
    - [x] Local variable
    - [x] Instance variable
    - [ ] Class variable
    - [ ] Global variable
- Constants
    - [x] Constant
- Branch and Loop
    - [x] If-then-elsif-else
    - [x] Unless-then-else
    - [x] Postfix if / unless
    - [x] For-in
    - [x] Break / Continue
    - [x] While
    - [x] Until
    - [x] Postfix while / until
    - [x] Return
- Methods
    - [x] Instance Method
    - [x] Class Method
    - [ ] Singleton Method
- Class and Module
    - [x] Subclass / Inheritance
    - [x] Initializer
    - [x] Attribute accessor
    - [x] Monkey patch (Ad-hoc class definition)
    - [x] Module

## How to run ruby
To build ruruby, You'll need installation of Rust.

To run ruby program file on ruruby,
```
$ cargo run tests/sample.rb
```
or
```
$ cargo run --release -- tests/sample.rb
```
You can launch irb-like interactive shell, omitting file name.
```
$ cargo run
```

### Option: Bytecode Trace execution
```
$ cargo run --features trace -- tests/sample.rb
    Finished dev [unoptimized + debuginfo] target(s) in 1.83s
     Running `target/debug/ruruby tests/sample.rb`
PUSH_STRING stack:0
SET_LOCAL stack:1
PUSH_STRING stack:0
PUSH_STRING stack:1
CONCAT_STR stack:2
GET_LOCAL stack:1
TO_S stack:2
CONCAT_STR stack:2
PUSH_STRING stack:1
CONCAT_STR stack:2
SEND_SELF stack:1
Hello world!
END stack:1
```

### Option: Emit ByteCode
```
$ cargo run --features emit-iseq -- tests/sample.rb
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/ruruby tests/sample.rb`
-----------------------------------------
MethodRef(15)
  00000 PUSH_STRING 
  00005 SET_LOCAL 0 '0'
  00014 PUSH_STRING 
  00019 PUSH_STRING 
  00024 CONCAT_STR
  00025 GET_LOCAL 0 '0'
  00034 TO_S
  00035 CONCAT_STR
  00036 PUSH_STRING 
  00041 CONCAT_STR
  00042 SEND_SELF 'puts' 1 items
  00051 END
Hello world!
```

### Option: Performance analysis per VM instruction
```
$ cargo run --features perf -- tests/sample.rb
    Finished dev [unoptimized + debuginfo] target(s) in 3.53s
     Running `target/debug/ruruby tests/sample.rb`
Hello world!
Performance analysis for Inst:
------------------------------------------
Inst name         count    %time     nsec
                                    /inst
------------------------------------------
END                   1     0.00      953
PUSH_STRING           4     1.65     2562
SET_LOCAL             1     0.33     2578
GET_LOCAL             1     0.00      454
SEND_SELF             1     0.99     6895
CONCAT_STR            3     1.65     3354
TO_S                  1     0.82     5193
CODEGEN               1    92.42   561408
EXTERN                1     1.48     9645
------------------------------------------
```