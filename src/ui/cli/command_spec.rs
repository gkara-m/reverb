use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{Command, failure::failure::Failure};

pub(super) struct CommandSpec {
    nodes: HashMap<String, CommandSpecNode>,
    transmit: Sender<Command>,
}

struct CommandSpecNode {
    valid_aliases: Vec<String>,
    help: String,
    children: Vec<String>,
    handler: Option<fn(&str, &Sender<Command>) -> Result<(), Failure>>,
    call_type: CommandCallType,
}

#[derive(PartialEq)]
pub enum CommandCallType {
    NoArgs,
    Args,
    NotCallable,
}

impl CommandSpecNode {
    fn new(valid_aliases: Vec<&str>, help: String, handler: Option<fn(&str, &Sender<Command>) -> Result<(), Failure>>, call_type: CommandCallType) -> CommandSpecNode {
        CommandSpecNode {
            valid_aliases: valid_aliases.into_iter().map(|s| s.to_string()).collect(),
            help,
            children: Vec::new(),
            handler,
            call_type,
        }
    }

    fn call(&self, input: Vec<&str>, position: usize, transmit: &Sender<Command>, command_spec: &CommandSpec) -> Result<(), Failure> {
        // global help handling, if the current node is help, handle it here since it is a special case.
        if self.valid_aliases.contains(&"help".to_string()) {
            command_spec.print_help(1);
            return Ok(());
        }

        // normal command handling
        for child in &self.children {
            let node = command_spec.get(child).unwrap();
            for alias in &node.valid_aliases {
                if alias == input.get(position).unwrap_or(&"") {
                    return node.call(input, position + 1, transmit, command_spec);
                }
            }
        }
        self.handle(input, position, transmit, command_spec)?;
        Ok(())
    }

    fn handle(&self, input: Vec<&str>, position: usize, transmit: &Sender<Command>, command_spec: &CommandSpec) -> Result<(), Failure> {
        println!("handling: {}", input.join(","));
        let mut valid= true;
        let args;
        match self.call_type {
            CommandCallType::NoArgs => {if input.len() > position {
                    println!("Command does not take arguments");
                    valid = false;
                }
                args = String::new();
            },
            CommandCallType::Args => {if input.len() <= position {
                    println!("Command requires arguments");
                    valid = false;
                }
                args = input[position..input.len()].join(" ");
            },
            CommandCallType::NotCallable => {
                valid = false;
                args = String::new(); // this is just to satisfy the borrow checker, this value will never be used since valid is false
            },
        }
        if valid && let Some(handler) = self.handler {
            handler(args.as_str(), transmit)?;
        } else {
            self.print_help(command_spec, format!("{}", input[0..position.saturating_sub(1)].join(" ")), 3);
        }
        Ok(())
    }

    fn print_help(&self, command_spec: &CommandSpec, prefix: String, num_layers: usize) {
        println!("{} {}{}", prefix, self.valid_aliases.join(" | "), self.help);
        if num_layers > 0 {
            let prefix = format!("{} {}", prefix, self.valid_aliases.get(0).unwrap_or(&"".to_string()));
            for child in &self.children {
                let node = command_spec.get(child).unwrap();
                node.print_help(command_spec, format!("{} ", prefix), num_layers - 1);
            }
        } else {
            if self.children.len() > 0 {
                println!("{} {} <args> | help (for more information on this command and its subcommands)", prefix, self.valid_aliases.join(" | "));
            }
        }
    }
}

impl CommandSpec {
    pub fn new (transmit: Sender<Command>) -> CommandSpec {
        let mut command_spec = CommandSpec {
            nodes: HashMap::new(),
            transmit,
        };
        command_spec.nodes.insert("root".to_string(), CommandSpecNode::new(vec![], "REVERB commands:".to_string(), None, CommandCallType::NotCallable));
        command_spec
    }

    pub fn add (mut self, name: &str, valid_aliases: Vec<&str>, help: &str, handler: Option<fn(&str, &Sender<Command>) -> Result<(), Failure>>, call_type: CommandCallType, parent: Option<&str>) -> CommandSpec {
        let name = name.to_string();
        if self.nodes.contains_key(&name) {
            unreachable!("Command spec node with name {} already exists, this should not be possible please report this bug", name);
        }
        let node = CommandSpecNode::new(valid_aliases, help.to_string(), handler, call_type);
        self.nodes.insert(name.clone(), node);
        match parent {
            Some(parent) => {
                match self.nodes.get(&parent.to_string()) {
                    Some(parent_node) => {
                        for child in &parent_node.children {
                            for alias in self.get(&name).unwrap().valid_aliases.iter() {
                                if self.nodes.get(child).unwrap().valid_aliases.contains(alias) {
                                    unreachable!("Parent node {} already has a child with alias {}, this should not be possible please report this bug", parent, alias);
                                }
                            }
                        }
                        self.nodes.get_mut(&parent.to_string()).unwrap().children.push(name);
                    },
                    None => {unreachable!("Parent node {} not found when adding command spec node, this should not be possible please report this bug", parent)},
                }
            }
            None => self.root_mut().children.push(name),
        }     
        self
    }

    pub fn call(&self, input: &str) -> Result<(), Failure> {
        let parts: Vec<&str> = input.split(' ').collect();
        self.root().call(parts, 0, &self.transmit, &self)
    }

    fn get(&self, name: &str) -> Option<&CommandSpecNode> {
        self.nodes.get(name)
    }

    fn root_mut(&mut self) -> &mut CommandSpecNode {
        self.nodes.get_mut("root").unwrap()
    }

    fn root(&self) -> &CommandSpecNode {
        self.nodes.get("root").unwrap()
    }
    
    pub fn print_help(&self, num_layers: usize) {
        self.root().print_help(&self, String::new(), num_layers);
    }
}