# SysYc
Compiler of Sysy2022, a programming language for ..what?

## 使用方法

`cargo run -- <input_file>`，编译指定的文件。

#### 输出模式：

`--parse`: 输出文法解析的结果。

`--llvm`: 输出 `llvm IR`。

`--riscv`: 输出最终的代码。

若不指定输出模式则会发生错误。

#### 参数

`-o`: 指定输出文件，未指定则在标准输出流输出。

`-Ox`：指定优化方式/等级（支持 `-O0`，`-O1`，`-O2` 三种优化等级，其中 `-O2` 可能产生错误）。

#### Features

`simu`: 以中端代码模拟器运行。示例 `cargo run --features "simu"  -- <源代码> [<输入文件>]` 。
`debug`: 用于调试。

## Usage

`cargo run -- <input_file>` to complie the file inputed.

#### Output Mode：

`--parse`: Output the result of grammar prasing.

`--llvm`: Output in `llvm IR`.

`--riscv`: Output riscv asm code as the final output.

An error occurs in case of no output mode is specified.

#### Arguments

`-o`: Specify the output file. Will output into standard output if not specified.

`-Ox`: Specify the optimization level. supports three levels of optimization: `-O0`, `-O1`, `-O2`, where `-O2` may produce errors.


#### Features

`simu`: Run as an simulator for our llvm IR. Usage `cargo run --features "simu"  -- <source> [<input>]` .
`debug`: For debugging.