use super::msnp18::command::MSNP18Command;

pub enum NotificationCommand {
    VER(),
    MSNP18(MSNP18Command)
}