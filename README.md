## macgui

 - 批量采集SN MAC地址客户端，数据展示为二维码并以`json`格式写入Redis。
 - 可配合 <https://github.com/spdrwcn/macjson> 导入Execl表格

 ![macgui](macgui.png)

## 用法

```
./macgui -h
macgui 1.2.1
h13317136163@163.com
MAC地址采集程序

USAGE:
    macgui.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bluetooth <Value>...    蓝牙匹配参数 [default: bluetooth true]
    -i, --ip <IP_ADDRESS>         Redis数据库地址 [default: redis://127.0.0.1:6379/0]
    -w, --wired <Value>...        有线网卡匹配参数 [default: gbe true]
    -l, --wireless <Value>...     无线网卡匹配参数 [default: wi-fi true]
```

## 默认参数 多组参数逗号分隔
```
vec!["gbe", "true"]
```

## 示例 
```
macgui -i redis://127.0.0.1:6379/0 -w gbe true realtek -l ax211 true wi-fi -b blue true 
```

## 编译 

- 推荐使用`cross`

```
git clone https://github.com/spdrwcn/macgui.git
cd macgui 
cross build --release --target=x86_64-pc-windows-gnu
```
