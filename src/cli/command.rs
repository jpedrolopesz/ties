use super::command_type::CommandType;
use super::Route; // Importe Route se estiver definido em outro arquivo

pub struct Command {
    pub command_type: CommandType,
    pub description: String,
    pub execute: fn() -> (),
}

impl Command {
    pub fn new(command_type: CommandType, description: &str, execute: fn() -> ()) -> Command {
        Command {
            command_type,
            description: description.to_string(),
            execute,
        }
    }
}

pub const ROUTES: [Route; 1] = [
    Route { path: "/home", handler_name: "home_handler" },
    // outras rotas
];




pub fn list_routes() {
    println!("Listando todas as rotas...");

    // Algum lugar em src/cli/command.rs ou em outro arquivo que importe ROUTES
    for route in ROUTES.iter() {
        println!("Path: {}, Handler: {}", route.path, route.handler_name);
    }

}

