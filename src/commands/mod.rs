use serenity::framework::{Framework, StandardFramework};

pub mod test;

pub fn set_commands(framework: StandardFramework) -> StandardFramework {
    framework
        .group(&test::TEST_GROUP)
}