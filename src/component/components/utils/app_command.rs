use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType};


pub struct ApplicationCommand<'a>(&'a ApplicationCommandInteraction);

impl<'a> ApplicationCommand<'a> {
    pub fn new(interaction: &'a ApplicationCommandInteraction) -> Self {
        ApplicationCommand(interaction)
    }
    pub fn fullname(&self) -> String {
        let mut names = vec![self.0.data.name.as_str()];
        let mut cmd = self.0.data.options.first();
        // s'inspirer de la fonction get_command pour produire le nom
        while let Some(opt) = cmd {
            names.push(opt.name.as_str());
            if opt.kind == ApplicationCommandOptionType::SubCommand {
                return names.join(".");
            }
            cmd = opt.options.first();
        }
        names.join(".")
    }
    pub fn get_command(&'a self) -> &'a ApplicationCommandInteractionDataOption {
        let mut cmd = self.0.data.options.first();
        while let Some(&ApplicationCommandInteractionDataOption{ref kind, ref options, ..}) = cmd {
            if kind == &ApplicationCommandOptionType::SubCommand {
                return cmd.unwrap();
            }
            if options.is_empty() {
                panic!("No subcommand found.\n{:?}", self.0);
            }
            cmd = options.first();
        }
        unreachable!()
    }
    pub fn get_argument(&'a self, name: &str) -> Option<&'a ApplicationCommandInteractionDataOption> {
        let command = self.get_command();
        command.options.iter().find(|opt| opt.name.as_str() == name)
    }
}