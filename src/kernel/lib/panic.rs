/* /src/kernel/lib/panic.rs */

use crate::kernel::console::{CONSOLE, ring_buffer::LOG_BUFFER};
use limine::framebuffer::Framebuffer;
use core::panic::PanicInfo;

pub fn kernel_panic(
    _fb: &Framebuffer,
    message: &str, 
    details: &[&str], 
    info: Option<&PanicInfo>
) -> ! {
    if let Some(mut console_lock) = CONSOLE.try_lock() {
        if let Some(console) = console_lock.as_mut() {

            console.newline();
            console.current_fg = 0x880808;
            let panic_msg = "PANIC: ";
            for &b in panic_msg.as_bytes() { console.draw_char(b); console.cursor_x += 8; }

            console.current_fg = 0x555555;
            for &b in message.as_bytes() { console.draw_char(b); console.cursor_x += 8; }

            if let Some(info) = info {
                if let Some(loc) = info.location() {
                    console.newline();
                    console.current_fg = 0x555555;
                    let file_info = "AT: ";
                    for &b in file_info.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
                    
                    let loc_str = loc.file();
                    for &b in loc_str.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
                }
            }
            
            console.newline();
            console.current_fg = 0x555555;
            let halt = "System halted.";
            for &b in halt.as_bytes() { console.draw_char(b); console.cursor_x += 8; }



    // if let Some(mut console_lock) = CONSOLE.try_lock().or_else(|| {
    //     // Ha foglalt, de pánik van, nincs idő várni: 
    //     // egy valódi OS-ben itt "feltörnénk" a lock-ot.
    //     None 
    // }) {
    //     if let Some(console) = console_lock.as_mut() {
    //         // 1. Teljes takarítás (közvetlenül a framebufferre)
    //         console.clear();
    //         console.cursor_x = 20;
    //         console.cursor_y = 20;

    //         // 2. ASCII Art rajzolása
    //         let art = [
    //             "           ######",
    //             "        ##########      #####",
    //             "    #########/####\\###########",
    //             "  ####     ###########     ####",
    //             " ##      ####  #####/@@      ###",
    //             "#      ###    ,-.##/`.#\\##     ##",
    //             "      ##     /  |$/  |,-. ##    #",
    //             "             \\_,'$\\_,'|  \\ ###",
    //             "               \\_$$$$$`._/  ##",
    //             "                 $$$$$_/    ##",
    //             "                 $$$$$       #",
    //             "                 $$$$$",
    //             "                 $$$$$",
    //             "                $$$$$",
    //             "                $$$$$",
    //             "               $$$$$$$",
    //             "              $$$$$$$$$",
    //         ];

            

    //         for line in art.iter() {
    //             console.current_fg = 0x555555; // ^&8 szürke
    //             // A render_bytes helyett használhatjuk a belső rajzolót
    //             for &b in line.as_bytes() {
    //                 console.draw_char(b);
    //                 console.cursor_x += 8;
    //             }
    //             console.newline();
    //             console.cursor_x = 20;
    //         }

    //         // 3. Szöveges üzenetek
    //         console.newline();
    //         console.current_fg = 0xFF0000; // ^&4 piros
    //         let banner = "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!";
    //         for &b in banner.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
            
    //         console.newline();
    //         console.cursor_x = 20;
    //         console.current_fg = 0xFF5555; // ^&c világospiros
    //         let title = "             KERNEL PANIC             ";
    //         for &b in title.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
            
    //         console.newline();
    //         console.cursor_x = 20;
    //         console.current_fg = 0xFF0000;
    //         for &b in banner.as_bytes() { console.draw_char(b); console.cursor_x += 8; }

    //         console.newline();
    //         console.newline();
    //         console.cursor_x = 20;
    //         console.current_fg = 0xFFFFFF; // ^&f fehér
    //         let reason_label = "REASON: ";
    //         for &b in reason_label.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
            
    //         console.current_fg = 0xFF5555;
    //         for &b in message.as_bytes() { console.draw_char(b); console.cursor_x += 8; }

    //         // Location kiírása
    //         if let Some(info) = info {
    //             if let Some(loc) = info.location() {
    //                 console.newline();
    //                 console.cursor_x = 20;
    //                 console.current_fg = 0xAAAAAA; // ^&7 szürke
    //                 let file_label = "FILE: ";
    //                 for &b in file_label.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
    //                 console.current_fg = 0xFFFFFF;
    //                 for &b in loc.file().as_bytes() { console.draw_char(b); console.cursor_x += 8; }
    //             }
    //         }

    //         console.newline();
    //         console.newline();
    //         console.cursor_x = 20;
    //         console.current_fg = 0x555555;
    //         let halt_msg = "System halted. Please reboot manually.";
    //         for &b in halt_msg.as_bytes() { console.draw_char(b); console.cursor_x += 8; }
        }
    }

    // CPU megállítása
    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}