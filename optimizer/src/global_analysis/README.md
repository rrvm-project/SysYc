# global analysis

全局内容分析：

1. 全局变量常量检测。
2. 副作用分析。
3. 读入内容分析。
   
## 全局变量常量检测

直接看这个全局变量有没有被写。

## 副作用分析

写入全局变量或函数参数的函数是有副作用的，这部分副作用可以被当做 store 在 mem2reg 过程中被处理。

## 读入内容分析

检查函数读入了哪些全局变量或者参数，这部分副作用可以被当做 load 在 mem2reg 过程中被处理。