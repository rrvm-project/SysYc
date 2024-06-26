## 寄存器处理

### 模块介绍

此模块处理寄存器的分配，以及函数调用时的寄存器使用：

1. 处理函数的传入参数。
2. 给指令分配寄存器，将溢出的寄存器存入虚拟内存（未指定位置的栈上空间）。
3. 计算虚拟内存在栈上的具体位置。
4. 处理 `caller_saved` 寄存器。
5. 处理 `callee_saved` 寄存器。

### 实现细节

#### 图染色

1. 贪心染色时：依次尝试未所有点染色，将所有无法染色的点批量溢出。
    > 如果一个点在前面的溢出点未染色的情况下还是不能染色，那这个点也必须要被溢出。
2. 只需进行一次数据流分析，将溢出的节点从 `live_out` 中删除即可得到溢出后基本块的数据流信息。
3. 虚拟内存的 `live_out` 就是原本的 `live_out` 中被溢出的寄存器。
    > 这里实现的比较脏，用 `live_in` 保存了溢出前的 `live_out`。

#### caller_saved

1. 在冲突图计算时，函数调用后存活的变量都是需要保存的。
