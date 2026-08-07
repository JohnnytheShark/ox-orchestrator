use ox_tools::ToolDispatcher;

pub fn handle_tools_command() {
    let dispatcher = ToolDispatcher::with_defaults();
    let defs = dispatcher.definitions();

    println!("Registered Tools ({}):\n", defs.len());
    for d in defs {
        let mutating_tag = if d.is_mutating {
            "[MUTATING - REQUIRES HITL APPROVAL]"
        } else {
            "[READ-ONLY / SAFE]"
        };

        println!("  * {:<16} {:<36}", d.name, mutating_tag);
        println!("    Description: {}", d.description);
        println!("    Parameters : {}\n", d.input_schema);
    }
}
