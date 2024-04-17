# dispatch

一个平衡网络接口间流量的SOCKS代理。

_应该适用于macOS、Windows和Linux。目前仅在macOS上进行了测试。_

这是对最初用CoffeeScript编写并针对Node.js的[dispatch-proxy](https://github.com/alexkirsz/dispatch-proxy)的Rust重写版本。

## 快速链接

- [安装](#安装)
- [原理](#原理)
- [使用案例](#使用案例)
- [使用方法](#使用方法)
- [示例](#示例)
- [工作原理](#工作原理)
- [许可证](#许可证)

## 安装

### 从预构建的二进制文件

您可以从[发布页面](https://github.com/alexkirsz/dispatch/releases)下载适用于macOS、Windows和Linux的预构建二进制文件。

### 从crates.io

您需要Rust版本1.51.0或更高版本。您可以使用[rustup](https://rustup.rs/)来安装最新版本的Rust编译器工具链。

```
cargo install dispatch-proxy
```

## 原理

您常常会发现自己有多个未使用的互联网连接——无论是5G移动热点还是免费Wi-Fi网络——您的系统不允许您与主要网络同时使用。

例如，我第一个学生宿舍提供了有线和无线互联网访问。它们分别有1,200kB/s的带宽上限。我的3G移动互联网访问又提供了额外的400kB/s。将所有这些通过dispatch和下载管理器结合起来，结果达到了2,800kB/s的有效带宽！

## 使用案例

可能性无限：

- 与下载管理器或BitTorrent客户端一起使用，合并多个连接的带宽以下载单个文件；
- 将您能访问的尽可能多的接口合并成一个单一的负载平衡接口；
- 在多个代理上运行不同的应用程序（例如，用于平衡下载/上传）；
- 在家中创建一个通过以太网和您的5G卡连接的热点代理，供所有移动设备使用；
- 等等。

## 使用方法

```
$ dispatch
一个平衡网络接口间流量的SOCKS代理。

使用方法：dispatch [选项] <命令>

命令：
  list   列出所有可用的网络接口
  start  启动SOCKS代理服务器
  help   打印此消息或给定子命令的帮助

选项：
  -d, --debug    将调试日志写入标准输出而不是文件
  -h, --help     打印帮助信息
  -V, --version  打印版本信息
```

```
$ dispatch start -h
启动SOCKS代理服务器

使用方法：dispatch start [选项] <地址>...

参数：
  <地址>...  以<地址>[/优先级]的形式分派到的网络接口IP地址

选项：
      --ip <IP>      接受连接的IP [默认值：127.0.0.1]
      --port <PORT>  用于监听连接的端口 [默认值：1080]
  -h, --help         打印帮助信息
```

## 示例

```
$ dispatch list
```

列出所有可用的网络接口。

```
$ dispatch start 10.0.0.0 fdaa:bbcc:ddee:0:1:2:3:4
```

将传入的连接分派到本地地址 `10.0.0.0` 和 `fdaa:bbcc:ddee:0:1:2:3:4`。

```
$ dispatch start 10.0.0.0/7 10.0.0.1/3
```

将传入的连接分派到 `10.0.0.0` 7

次中的10次，以及到 `10.0.0.1` 3次中的10次。

## 工作原理

每当SOCKS代理服务器接收到对地址或域的连接请求时，它将使用[加权轮询](https://en.wikipedia.org/wiki/Weighted_round_robin)算法选择一个提供的本地地址。然后，所有后续的连接流量都将通过对应的接口。

**注意：** 如果请求的地址或域解析为IPv4（或IPv6）地址，则必须提供IPv4（或IPv6）的本地地址。

#### 许可证

<sup>
根据您的选择，可根据<a href="LICENSE-APACHE">Apache License, Version
2.0</a>或<a href="LICENSE-MIT">MIT license</a>授权。
</sup>

<br>

<sub>
除非您明确声明，否则您为本库作出的任何故意提交的贡献，按照Apache-2.0许可证定义，将按上述方式双重许可，无需任何附加条款或条件。
</sub>
