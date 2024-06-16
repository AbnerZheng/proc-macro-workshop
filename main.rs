#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use derive_builder::Builder;
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    #[builder(each = "env")]
    env: Vec<String>,
    current_dir: Option<String>,
}
use std::error::Error;
pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}
impl CommandBuilder {
    pub fn executable(&mut self, executable: String) -> &mut Self {
        self.executable = Some(executable);
        self
    }
    pub fn args(&mut self, args: Vec<String>) -> &mut Self {
        self.args = Some(args);
        self
    }
    pub fn arg(&mut self, arg: String) -> &mut Self {
        self.args.get_or_insert(::alloc::vec::Vec::new()).push(arg);
        self
    }
    pub fn env(&mut self, env: String) -> &mut Self {
        self.env.get_or_insert(::alloc::vec::Vec::new()).push(env);
        self
    }
    pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
        self.current_dir = Some(current_dir);
        self
    }
    pub fn build(&mut self) -> Result<Command, Box<dyn Error>> {
        Ok(Command {
            executable: self
                .executable
                .as_ref()
                .ok_or("field `executable` is missing")?
                .clone(),
            args: self.args.as_ref().ok_or("field `args` is missing")?.clone(),
            env: self.env.as_ref().ok_or("field `env` is missing")?.clone(),
            current_dir: self.current_dir.clone(),
        })
    }
}
impl Command {
    pub fn builder() -> CommandBuilder {
        let builder = CommandBuilder {
            executable: None,
            args: None,
            env: None,
            current_dir: None,
        };
        builder
    }
}
fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();
    match (&command.executable, &"cargo") {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (
        &command.args,
        &<[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new(["build", "--release"])),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
