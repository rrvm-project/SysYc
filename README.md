# SysYc
compiler of Sysy2022, a programming language for ..what?


## 使用方法

TODO: 来个会英文的把这段改成英文

`cargo run -- <input_file>`，编译指定的文件。

#### 输出模式：

`--parse`: 输出文法解析的结果

`--llvm`: 输出 `llvm IR`。

`--riscv`: 输出最终的代码。

若不指定输出模式则会发生错误。

#### 参数

`-o`: 指定输出文件，未指定则在标准输出流输出。

`-Ox`：指定优化方式/等级（未实现）
