#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SystunnelFrame {
    pub user_ds: u64,    // [rsp] - utolsó push
    pub rax: u64,        // [rsp + 8]
    pub rbx: u64,        // [rsp + 16]
    pub rcx: u64,        // [rsp + 24]
    pub rdx: u64,        // [rsp + 32]
    pub rsi: u64,        // [rsp + 40] - EZ LEGYEN ELŐBB
    pub rdi: u64,        // [rsp + 48] - EZ LEGYEN UTÁNA
    pub rbp: u64,        // ...
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,        // legmagasabb cím - első push
}