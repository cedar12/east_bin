# east

高性能、低占用的一款TCP端口转发工具。可以方便地将本地端口转发到远程服务器上。

![](https://img.shields.io/github/stars/cedar12/east_bin)
![](https://img.shields.io/github/forks/cedar12/east_bin)
![](https://img.shields.io/github/watchers/cedar12/east_bin)
![](https://img.shields.io/github/languages/code-size/cedar12/east_bin)
![](https://img.shields.io/badge/license-Apache%202-blue)
![](https://img.shields.io/github/downloads/cedar12/east_bin/total)

## 使用技术

*   开发语言：rust
*   异步框架：tokio

## 安装说明

### 编译环境要求

Rust 版本：1.67.1 或更高版本

### 下载和编译

你可以从 Github 上下载最新版的 east 源代码：

```sh
git clone https://github.com/cedar12/east_bin.git
# 编译
cargo build --release
```

编译完成后，可执行文件位于 target/release 目录中。
将easts文件拷贝到你的服务器中运行。
将eastc文件拷贝到你需要转发的内网机器上运行。

> 服务端和代理端还需根据需求修改配置文件 easts.yml、eastc.yml [参考](#配置文件)

### 运行

要运行east，只需在终端中执行以下命令：

```sh
# 运行服务端 easts [配置文件路径,默认easts.yml]
./easts
# 运行代理端 eastc [配置文件路径,默认eastc.yml]
./eastc
```

## 配置文件

> 配置文件只支持yml格式，服务端、代理端默认为easts.yml和eastc.yml。可通过以下方式运行指定的配置文件

```sh
# 运行服务端
./easts easts.yml
# 运行代理端
./eastc eastc.yml
```

### 服务端配置

```yml
# easts.yml
server:
  # 服务器绑定端口
  bind: 0.0.0.0:3555
  # 可选配置 私钥文件路径 运行时如果不存在则会生成私钥公钥文件 公钥以pub_开头文件
  key: ./key.pem
agent:
  # agent连接的id
  test:
      # 在服务器上绑定的端口号
    - bind_port: 8089
      # agent上转发的目标ip或域名
      target_host: 127.0.0.1
      # agent上转发的目标端口号
      target_port: 42880
      # 白名单 ip规则
      whitelist: 
        - 192.168.0.0/16
        - 127.0.0.1
    - bind_port: 8989
      target_host: 127.0.0.1
      target_port: 5880
      # 限制最大速率64kb/s 默认不限速
      max_rate: 64
```

### 代理端配置

```yml
# eastc.yml
# 服务端ip端口
server: 127.0.0.1:3555
# 对应服务端配置的agent.test。如服务端未配置的id，服务端会拒绝连接
id: test
# 可选配置 公钥文件路径
key: ./pub_key.pem
```

### 下载使用

*   [下载地址](https://github.com/cedar12/east_bin/releases/latest)

下载完成解压后分为easts（服务端）、eastc（代理端）

### 帮助
#### windows系统运行时提示丢失``VCRUNTIME140.dll``
> 下载[Microsoft Visual C++ 2015 Redistributable](https://www.microsoft.com/en-us/download/details.aspx?id=53840)安装即可
#### windwos系统注册成系统服务
> 下载[nssm](http://www.nssm.cc/download)注册成系统服务

### 许可证

本项目基于Apache2.0许可证发布。

### 注意

1.  该开源项目仅供学习，使用者应自行承担责任。
2.  该开源项目不保证无错误或无缺陷。使用者应在自己的风险下使用该项目。
3.  该开源项目作者不承担任何由于使用该项目而引起的直接或间接损失，包括但不限于利润损失、数据丢失等。

