# tlsdemo

BORUIX 的用户态 TLS 验收程序：编译器级 `#[thread_local]` 的主线程读写与跨线程独立。

[English](README.en.md)

## 测什么

本程序是 3P4-1（用户态 TLS 完整支持）的 QEMU 验收载体，只依赖 `libsys`：

- **主线程**：在自己的 TLS 块上写入并回读，验证 `fs:` 相对访问可用
- **派生线程**：经 `thread_spawn` 起一个线程，该线程在**自己的**块上写不同的值并回读
- **独立性**：线程结束后主线程的值必须**未被影响**——两者若共享同一块即判定失败

通过时输出 `tls per-thread ok` 并以 0 退出。

## 构建与运行

本仓按 BORUIX 程序仓惯例自带 `build.rs` 与 `linker.ld`。**`linker.ld` 必须显式页对齐放置
`.tdata`/`.tbss`**：留作孤儿段时链接器会把 `.tdata` 插进页中间、产出 `p_vaddr` 非页对齐的
`PT_LOAD`，被内核加载器的整页映射限制拒绝（实测报 `not supported`）。

整树构建由 `tools` 仓统一驱动（本程序已登记进 `USER_PROGRAMS`）：

```
python tools/main.py build
python tools/main.py run --serial        # 或在 shell 中直接运行 /programs/tlsdemo.elf
```

## 背景

编译器 local-exec TLS 在 BORUIX 上落地依赖三个关键点（详见 `docs` 仓 `docs/TODO/3p.md` 的 3P4-1）：

1. 内核为**每个执行单元**建立独立 TLS 块 + TCB 并设 FS base（入队前就位）；
2. 进入用户态前必须装载**用户数据 FS 选择子**——空选择子下任何 `fs:` 访问立即 #GP，
   而 `rdfsbase` 只读 MSR、看起来完全正常，极具迷惑性；
3. TCB **首字段必须是线程指针自指针**（x86-64 TLS ABI）：编译器 local-exec 序列从
   `fs:[0]` 取线程指针再加负位移访问变量。

## 许可

MIT（见 `LICENSE`）。
