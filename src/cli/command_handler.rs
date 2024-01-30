// src/cli/command_handler.rs

use super::command::{Command, list_routes};
use super::command_type::CommandType;

pub fn register_commands() -> Vec<Command> {
    vec![
        Command::new(CommandType::ListRoutes, "List all routes", list_routes),
        // registre outros comandos aqui
    ]
}

pub fn execute_command(commands: &Vec<Command>, command_type: CommandType) {
    for command in commands {
        if command.command_type == command_type {
            (command.execute)();
            return;
        }
    }
    println!("Command not found");
}
