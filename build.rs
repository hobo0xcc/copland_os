extern crate toml;

use toml::map::Map;

fn default_arch() -> String {
    use toml::Value::Table;
    let config_str = std::fs::read_to_string("./.cargo/config.toml").unwrap();
    let values = config_str.parse::<toml::Value>().unwrap();
    if let Table(table) = &values["build"] {
        let arch = table.get("target").unwrap().to_string();
        arch[1..arch.len() - 1].to_string()
    } else {
        panic!("default arch is not found");
    }
}

fn main() {
    use std::env;
    let board_env = env::var("BOARD").unwrap();
    let mut arch_env = env::var("ARCH").unwrap();

    if arch_env == "default" {
        arch_env = default_arch();
    }

    let target_board = match board_env.as_ref() {
        "default" => match arch_env.as_ref() {
            "riscv64gc-unknown-none-elf" => "virt",
            "aarch64-unknown-none-softfloat" => "raspi3b",
            arch => unimplemented!("{}", arch),
        },
        board => board,
    };
    println!("cargo:rustc-cfg=target_board=\"{}\"", target_board);

    let config_str = std::fs::read_to_string("kernel.toml").unwrap();
    let values = config_str.parse::<toml::Value>().unwrap();
    let map_dummy = Map::new();
    for (name, val) in values.as_table().unwrap_or(&map_dummy) {
        println!("cargo:rustc-cfg={}={}", name, val.to_string());
    }
}
