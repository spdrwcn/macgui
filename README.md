## macgui

 - 批量采集SN MAC地址客户端，数据展示为二维码并写入Redis。
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

