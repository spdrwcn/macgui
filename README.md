## macgui

 - 批量采集SN MAC地址客户端，数据展示为二维码并以`json`格式写入Redis。
 - 可配合 <https://github.com/spdrwcn/macjson> 导入Execl表格

 ![macgui](macgui.png)

## 用法

```
./macgui -h
macgui 1.4.0
h13317136163@163.com
MAC地址采集程序

USAGE:
    macgui.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --ip <IP_ADDRESS>         Redis数据库地址 [default: redis://127.0.0.1:6379/0]
```

## conditions.yaml 配置文件，与`exe`文件同目录
```
conditions:  
  - adapter_type: wired  
    keywords:  
      - ["gbe", "true"]  
  - adapter_type: wireless  
    keywords:  
      - ["wi-fi", "true"]  
      - ["wi-fi", "ax"]  
      - ["wireless", "true"]  
  - adapter_type: bluetooth  
    keywords:  
      - ["bluetooth", "true"]
```

## 示例 
```
macgui -i redis://127.0.0.1:6379/0 
```

## 编译 

- 推荐使用`cross`

```
git clone https://github.com/spdrwcn/macgui.git
cd macgui 
cross build --release --target=x86_64-pc-windows-gnu
```
