/* /src/kernel/console/console_base.rs */
use limine::framebuffer::Framebuffer;

// 8x8-as font (bár a tömb 2048, a logikád 8 bájtos indexelést használ)
const FONT: &[u8; 2048] = include_bytes!("../../font.bin");

static mut INTERNAL_DEBUG_TICK: u64 = 0;

pub struct ConsoleBase<'a> {
    pub fb: &'a Framebuffer<'a>,
    pub cursor_x: u64,
    pub cursor_y: u64,
    pub current_fg: u32,
    pub current_bg: u32,
    pub line_height: u64, // Ez tárolja az Y irányú ugrást
}

impl<'a> ConsoleBase<'a> {
    pub fn new(fb: &'a Framebuffer<'a>) -> Self {
        Self {
            fb,
            cursor_x: 20,
            cursor_y: 20,
            current_fg: 0xFFFFFF,
            current_bg: 0x000000,
            line_height: 9, // Alapértelmezett: 8 pixel betű + 1 pixel szünet
        }
    }

    // ---RENDERERS---

    pub fn render_buffer(&mut self, buffer: &crate::kernel::console::ring_buffer::RingBuffer) {

        unsafe {
            if crate::kernel::console::display_manager::NEEDS_FULL_REDRAW {
                self.clear();
                self.cursor_x = 20;
                self.cursor_y = 20;
                crate::kernel::console::display_manager::NEEDS_FULL_REDRAW = false;
            }
        }

        let buf = buffer.get_buf();
        let pos = buffer.get_pos();

        self.cursor_x = 20;
        self.cursor_y = 20;

        if buffer.is_wrapped() {
            self.render_bytes(&buf[pos..]);
            self.render_bytes(&buf[..pos]);
        } else {
            self.render_bytes(&buf[..pos]);
        }
    }

    fn render_bytes(&mut self, data: &[u8]) {
        let mut i = 0;
        while i < data.len() {
            // Minecraft színek kezelése
            if data[i] == b'^' && i + 2 < data.len() && data[i + 1] == b'&' {
                self.current_fg = self.parse_color(data[i + 2]);
                i += 3;
                continue;
            }

            let b = data[i];
            // Ha a pufferben 0-ás bájt van (üres rész), ne rajzoljunk semmit
            if b == 0 { 
                i += 1; 
                continue; 
            }

            match b {
                b'\n' => self.newline(),
                b'\r' => self.cursor_x = 20,
                0x08 => { // <--- EZT IDE TEDD (Backspace)
                    if self.cursor_x > 20 {
                        self.cursor_x -= 8;
                        // Letöröljük a karaktert (szóközzel)
                        let old_fg = self.current_fg;
                        self.current_fg = self.current_bg;
                        self.draw_char(b' '); 
                        self.current_fg = old_fg;
                    }
                },
                32..=126 => {
                    // ... (Rajzoló kód marad) ...
                    self.draw_char(b);
                    self.cursor_x += 8;
                    // ... (Automata sorváltás marad) ...
                }
                _ => {}
            }
            i += 1;
        }
    }

    // ---DRAWERS---

    pub fn draw_char(&self, c: u8) {

        let index = c as usize;
        if index >= 256 { return; }

        let fb_ptr = self.fb.addr() as *mut u32;
        let pitch = (self.fb.pitch() / 4) as usize;

        // A rajzolási ciklus hossza igazodik a sormagassághoz, 
        // de nem rajzolunk többet, mint amit a font bír (max 10 sor)
        let rows_to_draw = if self.line_height < 10 { self.line_height } else { 10 };

        for row in 0..rows_to_draw {
            let curr_y = self.cursor_y + row;
            if curr_y >= self.fb.height() { break; }

            // Csak az első 8 sorban van adat a fontfájlból
            let row_data = if row < 8 { FONT[index * 8 + row as usize] } else { 0 };
            
            let row_addr = unsafe { fb_ptr.add(curr_y as usize * pitch + self.cursor_x as usize) };
            
            for col in 0..8 {
                if self.cursor_x + col as u64 >= self.fb.width() { break; }
                
                let bit = (row_data >> (7 - col)) & 1;
                let color = if bit == 1 { self.current_fg } else { self.current_bg };

                unsafe {
                    row_addr.add(col).write_volatile(color);
                }
            }
        }
    }

    //TODO: Nem a console_base-ben kell torolni a sorokat hanel a ringbufferbol.
    pub fn scroll(&mut self) {
        let fb_ptr = self.fb.addr() as *mut u32;
        let pitch = (self.fb.pitch() / 4) as usize;
        let height = self.fb.height() as usize;
        let width = self.fb.width() as usize;
        let line_h = self.line_height as usize;

        unsafe {
            // 1. Pixelek másolása felfelé
            // A (line_h) sortól kezdve mindent az (0) sorba másolunk
            for y in line_h..height {
                let src_row = fb_ptr.add(y * pitch);
                let dst_row = fb_ptr.add((y - line_h) * pitch);
                // Memória másolása soronként
                core::ptr::copy_nonoverlapping(src_row, dst_row, width);
            }

            // 2. Az utolsó sor kitakarítása a háttérszínnel
            for y in (height - line_h)..height {
                let row_addr = fb_ptr.add(y * pitch);
                for x in 0..width {
                    row_addr.add(x).write_volatile(self.current_bg);
                }
            }
        }
        // Visszahelyezzük a kurzort az utolsó sor elejére
        self.cursor_y -= self.line_height;
    }

    // ---UTILITY---
    pub fn newline(&mut self) {
        self.cursor_x = 20;
        self.cursor_y += self.line_height;
        
        // Ha túlmentünk az alján, görgetünk
        if self.cursor_y + self.line_height >= self.fb.height() - 10 {
            self.scroll();
        }
    }

    pub fn clear(&mut self) {
        let fb_ptr = self.fb.addr() as *mut u32;
        let pitch = (self.fb.pitch() / 4) as usize;
        let width = self.fb.width() as usize;
        let height = self.fb.height() as usize;

        for y in 0..height {
            let row_addr = unsafe { fb_ptr.add(y * pitch) };
            for x in 0..width {
                unsafe {
                    row_addr.add(x).write_volatile(self.current_bg);
                }
            }
        }
        
        // Kezdőpozíció visszaállítása
        self.cursor_x = 20;
        self.cursor_y = 20;
    }

    fn parse_color(&self, code: u8) -> u32 {
        match code {
            b'0' => 0x000000, b'1' => 0x0000AA, b'2' => 0x00AA00, b'3' => 0x00AAAA,
            b'4' => 0xAA0000, b'5' => 0xAA00AA, b'6' => 0xFFAA00, b'7' => 0xAAAAAA,
            b'8' => 0x555555, b'9' => 0x5555FF, b'a' => 0x55FF55, b'b' => 0x55FFFF,
            b'c' => 0xFF5555, b'd' => 0xFF55FF, b'e' => 0xFFFF55, b'f' => 0xFFFFFF,
            _ => 0xFFFFFF,
        }
    }

    /////////////////////////////////////////////////////////////////////
    //// ----------------------- DEBUG PANEL ----------------------- ////
    /////////////////////////////////////////////////////////////////////
    pub fn debug_panel(&mut self) {
        let (old_x, old_y) = (self.cursor_x, self.cursor_y);
        let (old_fg, old_bg) = (self.current_fg, self.current_bg);
        
        let ticks = unsafe { crate::arch::idt::get_timer_ticks() };
        let start_x = (self.fb.width() as u64).saturating_sub(200);
        let mut current_row_y = 10;
        let line_spacing = 16;
        let label_width = 48;

        // Kibővített segédfüggvény: pozíció + színek
        let mut next_row = |base: &mut ConsoleBase, fg: u32, bg: u32| {
            base.cursor_x = start_x;
            base.cursor_y = current_row_y;
            base.current_fg = fg;
            base.current_bg = bg;
            current_row_y += line_spacing;
        };

        // --- KERNEL VERSION ---
        next_row(self, 0xFFFFFF, 0x000000);
        self.draw_label(crate::KERNEL_VERSION.as_bytes());

        // --- TIME ---
        use crate::kernel::drivers::rtc::read_rtc_time;
        let (h, m, s) = read_rtc_time();


        next_row(self, 0xAAAAAA, 0x000000);
        self.draw_label(b"TIME: "); 
        self.cursor_x = start_x + label_width;
        self.draw_two_digits(h);
        self.draw_char(b':'); self.cursor_x += 8;
        self.draw_two_digits(m);
        self.draw_char(b':'); self.cursor_x += 8;
        self.draw_two_digits(s);

        // --- UPTIME ---
        next_row(self, 0xAAAAAA, 0x000000);
        self.draw_label(b"UP:   "); 
        self.cursor_x = start_x + label_width;
        self.draw_number(ticks / 100);

        // --- RAW TICKS ---
        next_row(self, 0xAAAAAA, 0x000000);
        self.draw_label(b"TICK: ");
        self.cursor_x = start_x + label_width;
        self.draw_number(ticks);

        // --- LIVE SPINNER ---
        next_row(self, 0xAAAAAA, 0x000000);
        self.draw_label(b"LIVE: ");
        self.cursor_x = start_x + label_width;
        let anim = [b'|', b'/', b'-', b'\\'];
        self.draw_char(anim[((ticks / 10) % 4) as usize]);

        self.cursor_x = start_x + label_width + 8;
        self.draw_char(anim[((ticks / 10) % 4) as usize]);

        self.cursor_x = start_x + label_width + 16;
        self.draw_char(anim[((ticks / 10) % 4) as usize]);

        self.cursor_x = start_x + label_width + 24;
        self.draw_char(anim[((ticks / 10) % 4) as usize]);

        self.cursor_x = start_x + label_width + 32;
        self.draw_char(anim[((ticks / 10) % 4) as usize]);

        // --- LAST KEY ---
        next_row(self, 0xAAAAAA, 0x000000);
        self.draw_label(b"KEY:  ");
        self.cursor_x = start_x + label_width;

        unsafe {
            let scancode = crate::arch::idt::LAST_SCANCODE;
            if scancode != 0 && scancode < 0x80 {
                // Hexa kiírás (pl. 0x24)
                self.draw_label(b"0x");
                let mut hex_buf = [0u8; 2];
                let hex_chars = b"0123456789ABCDEF";
                hex_buf[0] = hex_chars[(scancode >> 4) as usize];
                hex_buf[1] = hex_chars[(scancode & 0x0F) as usize];
                self.draw_label(&hex_buf);

                // Karakter kiírás fixálása
                if let Some(c) = crate::kernel::drivers::keyboard::Keyboard::scancode_to_char(scancode) {
                    self.draw_label(b" ("); // Ez lépteti a kurzort
                    self.draw_char(c as u8);
                    self.cursor_x += 8;      // <--- EZ HIÁNYZOTT: léptetjük a betű után
                    self.draw_char(b')');
                    self.cursor_x += 8;      // Biztonság kedvéért itt is
                } else {
                    self.draw_label(b" (?)");
                }
            } else {
                self.draw_label(b"None");
            }
        }

        // --- MODE ---
        next_row(self, 0xAAAAAA, 0x000000);
        self.draw_label(b"MODE: ");
        self.cursor_x = start_x + label_width;
        let mode_str = unsafe { crate::kernel::console::display_manager::CURRENT_MODE.as_str() };
        self.draw_label(mode_str.as_bytes());

        // --- WAIT FOR INPUT ---
        unsafe {
            if crate::kernel::lib::debug::IS_WAITING_FOR_INPUT {
                next_row(self, 0xFFFFFF, 0xFF0000);
                self.draw_label(b"WAITING FOR INPUT...");
            }
        }

        // --- VISSZAÁLLÍTÁS ---
        self.current_fg = old_fg;
        self.current_bg = old_bg;
        self.cursor_x = old_x;
        self.cursor_y = old_y;
    }

    fn draw_two_digits(&mut self, value: u8) {
        self.draw_char((value / 10) + b'0');
        self.cursor_x += 8;
        self.draw_char((value % 10) + b'0');
        self.cursor_x += 8;
    }
    // Segédfüggvények a tisztább kódért (ha a ConsoleBase-ben vagy):
    fn draw_label(&mut self, label: &[u8]) {
        for &c in label {
            self.draw_char(c);
            self.cursor_x += 8;
        }
    }

    fn draw_number(&mut self, mut num: u64) {
        if num == 0 {
            self.draw_char(b'0');
            return;
        }
        let mut divisor = 10000;
        let mut leading_zeros = true;
        while divisor > 0 {
            let digit = (num / divisor) % 10;
            if digit != 0 || !leading_zeros || divisor == 1 {
                self.draw_char(digit as u8 + b'0');
                self.cursor_x += 8;
                leading_zeros = false;
            }
            divisor /= 10;
        }
    }

}