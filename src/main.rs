#![no_std]
#![no_main]
#![feature(thread_local)]

use libsys::{cmdline, mmap, thread_exit, thread_join, thread_spawn, write, STDOUT};

/// 必须**外部可见**且**先写后读**，否则 LLVM 会消除/常量折叠它——产物既无 PT_TLS
/// 也无 fs: 访问，"TLS 测试"退化成打印常量（本项目连续踩中两次）。
#[thread_local]
#[unsafe(no_mangle)]
pub static mut SLOT: u32 = 0;

/// 线程回填给它自己的观察值（join 提供顺序保证）。
static mut THREAD_SAW: u32 = 0;

const STACK_SIZE: usize = 64 * 1024;

fn puts(s: &[u8]) {
    let _ = write(STDOUT, s);
}

fn put_hex(mut v: u64) {
    let mut buf = [0u8; 16];
    let mut i = 16usize;
    loop {
        i -= 1;
        let d = (v & 0xf) as u8;
        buf[i] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
        v >>= 4;
        if v == 0 {
            break;
        }
    }
    puts(b"0x");
    let _ = write(STDOUT, &buf[i..]);
}

#[unsafe(naked)]
unsafe extern "C" fn thread_entry() {
    core::arch::naked_asm!("and rsp, -16", "call {b}", b = sym thread_body);
}

/// 派生线程体：在**自己的** TLS 块上写 0x22222222 并回读。
extern "C" fn thread_body() {
    unsafe { SLOT = 0x2222_2222 };
    let v = unsafe { SLOT };
    unsafe { THREAD_SAW = v };
    thread_exit(0);
}

#[unsafe(no_mangle)]
pub extern "C" fn user_main(argc: isize, argv: *const *const u8) -> i32 {
    // 3P4-2 验收：报出**内核/loader 实际交付**的命令行（长度 + 尾部 16 字节）。
    // 旧上限 511 字节一旦回归，长命令行会被 E2BIG 拒绝（程序根本不会跑到这里）；
    // 若被静默截断，尾部字节就对不上——故尾部是"完整送达"的行为锚点。
    match unsafe { cmdline(argc, argv) } {
        Some(c) => {
            puts(b"cmd_len=");
            put_hex(c.len() as u64);
            puts(b"\ncmd_tail=");
            let n = c.len().min(16);
            let _ = write(STDOUT, &c[c.len() - n..]);
            puts(b"\n");
        }
        None => puts(b"cmd_len=none\n"),
    }

    // 主线程：在自己的块上写 0x11111111。
    unsafe { SLOT = 0x1111_1111 };
    let main_before = unsafe { SLOT };
    puts(b"main tls=");
    put_hex(main_before as u64);
    puts(b"\n");

    let Ok(stack) = mmap(STACK_SIZE as u64) else {
        puts(b"mmap failed\n");
        return 2;
    };
    let stack_top = stack + STACK_SIZE as u64;
    let Ok(tid) = thread_spawn(thread_entry as usize as u64, stack_top) else {
        puts(b"thread_spawn failed\n");
        return 3;
    };
    let _ = thread_join(tid);

    let thread_saw = unsafe { THREAD_SAW };
    let main_after = unsafe { SLOT };
    puts(b"thread saw=");
    put_hex(thread_saw as u64);
    puts(b"\nmain after=");
    put_hex(main_after as u64);
    puts(b"\n");

    if thread_saw == 0x2222_2222 && main_after == 0x1111_1111 {
        puts(b"tls per-thread ok\n");
        0
    } else {
        puts(b"tls per-thread bad\n");
        1
    }
}
