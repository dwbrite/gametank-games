use npm_rs::*;

fn main() {
    NpmEnv::default()
        .set_path("ui")
        .init_env()
        .run("build:coffee")
        .run("build:public")
        .run("build:sass")
        .exec().unwrap();
}