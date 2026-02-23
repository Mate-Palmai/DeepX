pub mod command_manager;
pub mod system;
pub mod utils;

pub struct CommandArgs<'a> {
    pub args: &'a [&'a str],
}