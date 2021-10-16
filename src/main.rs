mod config;


trait ResultLog {
    type OkType;
    fn expect_log(self, msg: &str) -> Self::OkType;
}
impl<T, S: AsRef<str>> ResultLog for Result<T, S> {
    type OkType=T;
    fn expect_log(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                panic!("{}: {}", msg, e.as_ref());
            }
        } 
    }
}
// impl<T, S: ToString> ResultLog for Result<T, S> {
//     type OkType=T;
//     fn expect_log(self, msg: &str) -> T {
//         match self {
//             Ok(v) => v,
//             Err(e) => {
//                 panic!("{}: {}", msg, e.to_string());
//             }
//         } 
//     }
// }


#[tokio::main]
async fn main() {
    let config = config::Config::read_file("./config.json").expect_log("Could not load the configuration file");
    println!("{:?}", config);
}
