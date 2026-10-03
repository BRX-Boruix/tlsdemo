# tlsdemo

A BORUIX user-space TLS acceptance program: compiler-level `#[thread_local]` access on the main
thread and independence across threads.

[简体中文](README.md)

## What it checks

This is the QEMU acceptance vehicle for 3P4-1 (full user-space TLS support). It depends only on
`libsys`:

- **Main thread**: writes and reads back its own TLS block, proving `fs:`-relative access works.
- **Spawned thread**: `thread_spawn` starts a thread that writes a different value into **its own**
  block and reads it back.
- **Independence**: after the thread exits, the main thread's value must be **unchanged** — sharing
  one block fails the check.

On success it prints `tls per-thread ok` and exits 0.

## Build and run

Like every BORUIX program repo this one ships `build.rs` and `linker.ld`. **`linker.ld` must place
`.tdata`/`.tbss` explicitly and page-aligned**: as orphan sections the linker inserts `.tdata`
mid-page, producing a `PT_LOAD` whose `p_vaddr` is not page-aligned, which the kernel loader's
whole-page mapping rejects (observed as `not supported`).

The whole tree is built from the `tools` repo (this program is listed in `USER_PROGRAMS`):

```
python tools/main.py build
python tools/main.py run --serial        # or run /programs/tlsdemo.elf from the shell
```

## Background

Landing compiler local-exec TLS on BORUIX required three things (see `docs/TODO/3p.md` item 3P4-1 in
the `docs` repo):

1. The kernel builds an independent TLS block + TCB for **every execution unit** and sets FS base
   (in place before the unit is enqueued);
2. Entering user mode requires a **user-data FS selector** — a null selector makes any `fs:` access
   raise #GP, while `rdfsbase` only reads the MSR and therefore looks perfectly fine;
3. The TCB's **first field must be the thread-pointer self pointer** (x86-64 TLS ABI): the compiler's
   local-exec sequence loads the thread pointer from `fs:[0]` and then applies a negative
   displacement.

## License

MIT (see `LICENSE`).
