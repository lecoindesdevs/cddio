mod bot;
mod component;
mod config;
#[macro_use]
mod util;

trait ResultLog {
    type OkType;
    fn expect_log(self, msg: &str) -> Self::OkType;
}
impl<T, S: AsRef<str>> ResultLog for Result<T, S> {
    type OkType=T;
    fn expect_log(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) if msg.is_empty() => panic!("{}", e.as_ref()),
            Err(e) => panic!("{}: {}", msg, e.as_ref()),
        } 
    }
}
#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    let config = config::Config::read_file("./config.json").expect_log("Could not load the configuration file");
    let mut bot = bot::Bot::new(&config).await.or_else(|e|Err(e.to_string())).expect_log("");
    bot.start().await.or_else(|e|Err(e.to_string())).expect_log("Client won't start");
}
