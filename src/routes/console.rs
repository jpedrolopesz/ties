pub fn process_comand(command: String) {


    if command.starts_with("/msg") {
        chat_controller::send_message(parsead_message);
    }
}