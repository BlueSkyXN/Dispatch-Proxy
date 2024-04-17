# Dispatch-Proxy
一个平衡网络接口间流量的SOCKS代理。

# 调研说明
原作者英文文档和翻译我发在GitHub仓库了。
原作者说只用了Mac进行测试，我目前已成功在Windows 11 Pro环境下完成了测试

## 功能
通过纯软件实现多网卡带宽叠加和负载均衡
- 通过绑定多个本地IP接口，起一个本地S5 Proxy，通过该入口实现加权负载均衡控制流量
- 实现下行带宽多网卡（网络适配器）聚合带宽叠加
- 实现上行带宽多网卡（网络适配器）聚合带宽叠加

## 测试环境

Windows 11 Pro 23H2
``alexkirsz/dispatc`` [v0.1.3](https://github.com/alexkirsz/dispatch/releases/tag/v0.1.3)
``V2rayN`` [v6.4.3](https://github.com/2dust/v2rayN/releases/tag/6.43)
``Netch`` [v1.8.1](https://github.com/netchx/netch/releases/tag/1.8.1)

## 测试效果
在同路由器下，通过WindowsPro策略组同时接入Wi-Fi（802.11ax 2.4Gbps 5Ghz）和以太网（2.5G LAN）网线，IP分别是``192.168.110.2``和``192.168.110.1``
成功实现在测速网（Speedtest.net)等网站，实现WiFi+LAN有线网线接入的带宽叠加和加权负载均衡

启动命令
```
./dispatch start  --ip 127.0.0.1 --port 10806 192.168.110.101@7 192.168.110.102@3
```

原文档说用斜杠来设置权重，经过我实验发现需要用@而不是斜杠

### 如何套娃
- V2rayN作为出口，需要做好DNS等设置，如果有异常需要重启服务
- Netch作为进程代理控制程序，配置并使用Dispatch提供的S5代理，对V2rayN的二进制程序启用（包括内核，比如xray.exe、sing-box.exe）
- Dispatch自行选择端口和可用的本地IP，作为上联源，以及配置权重，默认是均等连接，然后默认端口是1080，不可冲突端口

### 预测可用场景
- 实现高速内网：叠加WiFi+LAN达到更大的理论速度，不影响互联网出口IP。
- 叠加光猫LAN口：常见的光猫通常为1Gbps LAN接口，可通过叠加多个G口或者光猫自带WiFi实现更大带宽接入，获取被G口限制的冗余带宽，不影响互联网出口IP。
- 多宽带/外网并联：叠加蜂窝流量卡（蜂窝网络类，包括MIFI/手机热点）实现更大带宽的外网上下行水平，但是这会导致出口IP不一致的问题，可能有负面影响。


## 原说明文档使用方法

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

### 示例

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
