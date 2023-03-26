# MicroBin

![Build](https://github.com/szabodanika/microbin/actions/workflows/rust.yml/badge.svg)
![crates.io](https://img.shields.io/crates/v/microbin.svg)
[![Docker Image](https://github.com/szabodanika/microbin/actions/workflows/docker.yml/badge.svg)](https://hub.docker.com/r/danielszabo99/microbin)

MicroBin是一个超小型、功能丰富、可配置、自包含和自托管的粘贴箱web应用程序。它非常易于设置和使用，只需要几兆字节的内存和磁盘存储。设置它只需要几分钟，为什么不现在就试试呢？

## microbin汉化
### 说明
简单部分汉化,仅供学习使用,随缘更新!
## 演示:
图片:
![](https://lsky.balabi.asia/i/2023/03/26/641ff7330d924.png)
测试demo:[microbin中文测试](https://bin.alldreams.top/)

## 使用方法:

zip包上传至服务器并解压
`unzip [包名]`
1. 进入项目Dockerfile所在目录
2. 构建镜像
```shell
docker build -t microbin-zh_cn:v1 .
```
3. 配置容器
```shell
nano docker-compose.yml
```

参考配置:(注意镜像名的修改!!!)
```yaml
version: '3.5'

services:
  microbin:
    image: microbin-zh_cn:v1
    container_name: microbin
    restart: unless-stopped
    environment:
      - TZ=Asia/Shanghai
      - MICROBIN_HIGHLIGHTSYNTAX=true
      - MICROBIN_HASH_IDS=true
      - MICROBIN_EDITABLE=true
      - MICROBIN_PRIVATE=true
      - MICROBIN_HIDE_FOOTER=false
      - MICROBIN_HELP=true
      - MICROBIN_FOOTER_TEXT=内容设置保存的最长时间只有一周,请及时将内容保存到本地!!!
      - MICROBIN_HIDE_HEADER=false
      - MICROBIN_HIDE_LOGO=false
      - MICROBIN_NO_ETERNAL_PASTA=true
      - MICROBIN_NO_FILE_UPLOAD=false
      - MICROBIN_NO_LISTING=false
      - MICROBIN_THREADS=2
      - MICROBIN_TITLE=free-bin
      - MICROBIN_PUBLIC_PATH=http://localhost:5423/ 
      - MICROBIN_QR=true
    ports:
      - 5423:8080
    volumes:
      - ./microbin-data:/app/pasta_data
```
