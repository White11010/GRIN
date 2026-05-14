use std::convert::TryFrom;

pub enum Command {
    Timeline
}
impl TryFrom<&str> for Command {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "timeline" => Ok(Command::Timeline),
            _ => Err("Invalid command"),
        }
    }
}

pub struct Config {
    pub command: Command,
}
impl Config {
    pub fn build(args: &[String]) -> Result<Self, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments");
        }
    
        let command = Command::try_from(args[1].as_str())?;
    
        Ok(Self { command })
    }
}


pub fn run(config: Config) {
    match config.command {
        Command::Timeline => {
            println!("Timeline");
        }
    }
}