/* src/kernel/debug/debug_panel.rs */
use crate::kernel::console::CONSOLE;
use crate::kernel::process::SCHEDULER;
use crate::kernel::console::console_base::ConsoleBase;
use core::sync::atomic::Ordering;

use alloc::format;


pub fn debug_panel_main() {
    unsafe { core::arch::asm!("sti"); }

    let mut last_update_tick = 0;

    loop {
        let current_ticks = unsafe { crate::arch::x86::idt::get_timer_ticks() };

        if current_ticks >= last_update_tick + 10 {
            last_update_tick = current_ticks;

            let stats = if let Some(sched) = SCHEDULER.try_lock() {
                let count = sched.get_task_count();
                let running_task_name = sched.get_tasks().iter()
                    .find(|t| t.state == crate::kernel::process::task::TaskState::Running)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "Unknown".into());
                Some((count, running_task_name))
            } else {
                None
            };

            if let Some((task_count, current_name)) = stats {
                if let Some(mut console_lock) = CONSOLE.try_lock() {
                    if let Some(console) = console_lock.as_mut() {
                        render_debug_overlay(console, task_count, &current_name);
                    }
                }
            }
        }

        crate::kernel::process::Scheduler::yield_now();
    }
}

fn render_debug_overlay(console: &mut ConsoleBase, task_count: usize, current_name: &str) {
    let (old_x, old_y) = (console.cursor_x, console.cursor_y);
    let (old_fg, old_bg) = (console.current_fg, console.current_bg);
    
    let ticks = unsafe { crate::arch::x86::idt::get_timer_ticks() };
    let start_x = (console.fb.width() as u64).saturating_sub(200);
    let mut current_row_y = 10;
    let line_spacing = 16;
    let label_width = 48;

    let mut next_row = |c: &mut ConsoleBase, fg: u32| {
        c.cursor_x = start_x;
        c.cursor_y = current_row_y;
        c.current_fg = fg;
        c.current_bg = 0x000000;
        current_row_y += line_spacing;
    };

    #[cfg(feature = "dev")]
    {
    next_row(console, 0xaa0a0a);
    draw_label_internal(console, b"DEV BUILD - UNSTABLE"); 
    }

    // --- KERNEL VERSION ---
    next_row(console, 0xFFFFFF);
    draw_label_internal(console, crate::KERNEL_VERSION.as_bytes());

    // --- UPTIME ---
    next_row(console, 0xAAAAAA);
    draw_label_internal(console, b"UP:   ");

    let (sec, frac_4) = crate::arch::x86::timer::tsc::get_uptime();
    
    let frac = frac_4 / 100;

    let base_x = start_x + label_width;
    
    console.cursor_x = base_x;
    draw_number_fixed_internal(console, sec, 4); 
    console.cursor_x = base_x + (4 * 8); 
    console.draw_char(b'.');
    console.cursor_x = base_x + (5 * 8);
    draw_number_fixed_internal(console, frac, 2);

    // --- LIVE SPINNER ---
    next_row(console, 0xAAAAAA);
    draw_label_internal(console, b"LIVE: ");
    console.cursor_x = start_x + label_width;
    
    let anim = [b'|', b'/', b'-', b'\\'];
    
    let freq = crate::arch::x86::timer::tsc::get_tsc_frequency();
    let tsc_now = crate::arch::x86::timer::tsc::read_tsc();
    
    let char_to_draw = if freq > 0 {
        anim[((tsc_now / (freq / 8)) % 4) as usize]
    } else {
        b'?'
    };

    for _ in 0..5 {
        console.draw_char(char_to_draw);
        console.cursor_x += 8;
    }

    // --- TASK COUNT ---
    next_row(console, 0xAAAAAA);
    draw_label_internal(console, b"TSK:  ");
    console.cursor_x = start_x + label_width;
    draw_number_internal(console, task_count as u64);

    // --- LAST KEY ---
    next_row(console, 0xAAAAAA);
        draw_label_internal(console, b"KEY:  ");
        console.cursor_x = start_x + label_width;

        unsafe {
            let scancode = crate::arch::x86::idt::LAST_SCANCODE;
            if scancode != 0 && scancode < 0x80 {
                draw_label_internal(console, b"0x");
                let mut hex_buf = [0u8; 2];
                let hex_chars = b"0123456789ABCDEF";
                hex_buf[0] = hex_chars[(scancode >> 4) as usize];
                hex_buf[1] = hex_chars[(scancode & 0x0F) as usize];
                draw_label_internal(console, &hex_buf);

                if let Some(c) = crate::kernel::drivers::keyboard::Keyboard::scancode_to_char(scancode) {
                    draw_label_internal(console, b" ("); 
                    console.draw_char(c as u8);
                    console.cursor_x += 8;     
                    console.draw_char(b')');
                    console.cursor_x += 8; 
                } else {
                    draw_label_internal(console, b" (?)");
                }
            } else {
                draw_label_internal(console, b"None");
            }
        }

    // --- MODE (DisplayMode) ---
    next_row(console, 0xAAAAAA);
    draw_label_internal(console, b"MODE: ");
    console.cursor_x = start_x + label_width;
    let mode_str = unsafe { crate::kernel::console::display_manager::CURRENT_MODE.as_str() };
    draw_label_internal(console, mode_str.as_bytes());

    console.current_fg = old_fg;
    console.current_bg = old_bg;
    console.cursor_x = old_x;
    console.cursor_y = old_y;
}

fn draw_number_fixed_internal(c: &mut ConsoleBase, num: u64, width: usize) {
    let mut divisor = 1;
    for _ in 0..(width - 1) { divisor *= 10; }
    
    for _ in 0..width {
        let digit = (num / divisor) % 10;
        c.draw_char(digit as u8 + b'0');
        c.cursor_x += 8;
        divisor /= 10;
    }
}

fn draw_label_internal(c: &mut ConsoleBase, label: &[u8]) {
    for &b in label {
        c.draw_char(b);
        c.cursor_x += 8;
    }
}

fn draw_number_internal(c: &mut ConsoleBase, mut num: u64) {
    if num == 0 {
        c.draw_char(b'0');
        return;
    }
    let mut divisor = 1;
    let mut temp = num;
    while temp >= 10 {
        temp /= 10;
        divisor *= 10;
    }
    while divisor > 0 {
        let digit = (num / divisor) % 10;
        c.draw_char(digit as u8 + b'0');
        c.cursor_x += 8;
        divisor /= 10;
    }
}