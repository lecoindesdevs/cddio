use serenity::framework::StandardFramework;

mod test;

pub fn set_commands(framework: StandardFramework) -> StandardFramework {
    framework
        .group(&test::TEST_GROUP)
}