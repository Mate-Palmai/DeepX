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

        if current_ticks >= last_update_tick + 5 { 
            last_update_tick = current_ticks;

            let (count, cpu_stats) = {
                let sched = SCHEDULER.lock();
                let c = sched.get_task_count();
                let stats = sched.get_cpu_tasks(); 
                (c, stats)
            };

            if let Some(mut console_lock) = CONSOLE.try_lock() {
                if let Some(console) = console_lock.as_mut() {
                    render_debug_overlay(console, count, &cpu_stats);
                }
            }
        }

        core::hint::spin_loop();
        crate::kernel::process::Scheduler::yield_now();
    }
}

fn render_debug_overlay(console: &mut ConsoleBase, task_count: usize, cpu_tasks: &[u64; 8]) {
    let (old_x, old_y) = (console.cursor_x, console.cursor_y);
    let (old_fg, old_bg) = (console.current_fg, console.current_bg);
    
    let start_x = (console.fb.width() as u64).saturating_sub(200);
    let mut current_row_y = 10;
    let line_spacing = 16;
    let label_width = 48;

    
    #[cfg(feature = "dev")]
    {
        console.cursor_x = start_x;
        console.cursor_y = current_row_y;
        console.current_fg = 0xaa0a0a;
        draw_label_internal(console, b"DEV BUILD - UNSTABLE"); 
        current_row_y += line_spacing;
    }

    // --- KERNEL VERSION ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
    console.current_fg = 0xFFFFFF;
    draw_label_internal(console, crate::KERNEL_VERSION.as_bytes());
    current_row_y += line_spacing;

    // --- UPTIME ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
    console.current_fg = 0xAAAAAA;
    draw_label_internal(console, b"UP:   ");
    let (sec, frac_4) = crate::arch::x86::timer::tsc::get_uptime();
    console.cursor_x = start_x + label_width;
    draw_number_fixed_internal(console, sec, 4); 
    console.draw_char(b'.');
    console.cursor_x += 8;
    draw_number_fixed_internal(console, frac_4 / 100, 2);
    current_row_y += line_spacing;

    // --- LIVE SPINNER ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
    draw_label_internal(console, b"LIVE: ");
    console.cursor_x = start_x + label_width;
    let anim = [b'|', b'/', b'-', b'\\'];
    let tsc_now = crate::arch::x86::timer::tsc::read_tsc();
    let freq = crate::arch::x86::timer::tsc::get_tsc_frequency();
    let char_to_draw = if freq > 0 { anim[((tsc_now / (freq / 8)) % 4) as usize] } else { b'?' };
    for _ in 0..5 { console.draw_char(char_to_draw); console.cursor_x += 8; }
    current_row_y += line_spacing;

    // --- TASK COUNT ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
    draw_label_internal(console, b"TSK:  ");
    console.cursor_x = start_x + label_width;
    draw_number_internal(console, task_count as u64);
    current_row_y += line_spacing;

    // --- CPU CORES & RUNNING TASKS ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
    console.current_fg = 0x00FF00;
    draw_label_internal(console, b"CPU CORES:");
    current_row_y += line_spacing;
    
    for (cpu_id, &task_id) in cpu_tasks.iter().enumerate() {
        if cpu_id >= 4 { break; } 

        console.cursor_x = start_x + 8;
        console.cursor_y = current_row_y;
        console.current_fg = 0xAAAAAA;

        draw_label_internal(console, b"CPU ");
        draw_number_internal(console, cpu_id as u64);
        draw_label_internal(console, b": ");

        if task_id == u64::MAX {
            console.current_fg = 0x555555;
            draw_label_internal(console, b"IDLE");
        } else {
            console.current_fg = 0x00FFFF;
            draw_label_internal(console, b"TASK #");
            draw_number_internal(console, task_id);
        }
        
        current_row_y += line_spacing;
    }
    // --- LAST KEY ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
    console.current_fg = 0xAAAAAA;
    draw_label_internal(console, b"KEY:  ");
    console.cursor_x = start_x + label_width;
    unsafe {
        let scancode = crate::arch::x86::idt::LAST_SCANCODE;
        if scancode != 0 && scancode < 0x80 {
            draw_label_internal(console, b"0x");
            let hex_chars = b"0123456789ABCDEF";
            console.draw_char(hex_chars[(scancode >> 4) as usize]);
            console.cursor_x += 8;
            console.draw_char(hex_chars[(scancode & 0x0F) as usize]);
            console.cursor_x += 8;
            if let Some(c) = crate::kernel::drivers::keyboard::Keyboard::scancode_to_char(scancode) {
                draw_label_internal(console, b" ("); console.draw_char(c as u8); console.cursor_x += 8; draw_label_internal(console, b")");
            }
        } else { draw_label_internal(console, b"None"); }
    }
    current_row_y += line_spacing;

    // --- MODE ---
    console.cursor_x = start_x;
    console.cursor_y = current_row_y;
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