/*
 * DeepX OS Project
 * Copyright (C) 2024-2026 - Máté Pálmai
 *
 * File: /src/kernel/boot/welcome.rs
 * Description: Welcome message.
 */

const LOGO: &str = r#"^&7________                    ____  ___
\______ \   ____   ____ ______\   \/  /
 |    |  \_/ __ \_/ __ \\____ \\   / 
 |    `   \  ___/\\  ___/|  |_> >     \
/_______  /\___  >\___  >  __/___/\  \
        \/     \/     \/|__|        \_/"#;


pub fn show_welcome<'a, 'b>() {

    crate::kernel::console::LOGGER.raw(LOGO);
    crate::kernel::console::LOGGER.raw("\n\n");

    // let mut logger = Logger::new(console);
    // logger.info("Welcome to DeepX OS!");
}