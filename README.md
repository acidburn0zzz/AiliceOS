# AiliceOS

# 注意⚠️

### 该项目正在进行大规模重构,包括 多CPU架构支持,UEFI支持、完整文件系统、内存等


### 进度报告
这是UEFI启动内核的一个基础代码，该版本实现了UFEI启动内核

#### 其他：
#### 未来：
* I/O支持
* 进程管理
* 基础驱动
* IPC通信
* 模块化
* ....

### 项目仍在进行

## Simplified Chinese(简体中文)

### 关于系统
这是AiliceOS，基于Rust开发的操作系统，目前并不是一个OS

* 系统名称：AiliceOS

### 未来
希望可以当做一个正常的Unix使用

### 多平台
* 架构：x64(mips,aarch64下一个版本将会支持)。

### 编译
平台：Windows10 MacOS Linux* Unix*

需要的工具：
* Rust QEMU cargo-xbuild


### 组件安装
安装Rust
```
$  curl https://sh.rustup.rs -sSf | sh
```
如果您安装了Rust但没有nightly版，可以使用这个命令：
```
$  rustup install nightly
```
设置nightly为默认

```
$  rustup default nightly
```

安装拓展工具
```
$  cargo install cargo-xbuild

$  rustup component add rust-src --toolchain=nightly

# 如果为MacOS，则使用brew安装QMEU
$  /usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"

$  brew install qemu
# 如果为Ubuntu，则使用apt安装QMEU
$  sudo apt update && sudo apt install qemu-system # or sudo apt install qemu

# 如果为Windows，推荐使用Windows10的WSL(Linux子系统)

$  rustup component add rust-src
```

您可以使用以下命令进行编译：

运行：

```
$  make all
```
