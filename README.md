## 使用说明

### 图形界面模式
直接运行可执行文件即可打开图形界面。

### 命令行模式
```bash
# 生成固件版本号
./ssd_sn_tool firmware generate 2025 12 1 1 1024 A 4

# 解析固件版本号
./ssd_sn_tool firmware parse S01E1A4

# 查看配置
./ssd_sn_tool firmware config
```

## To Do
- [ ] SN码生成与解码功能
- [ ] 产品型号生成与解码功能
