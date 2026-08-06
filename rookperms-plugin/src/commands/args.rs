use pumpkin_plugin_api::command::{Arg, ConsumedArgs};
use pumpkin_plugin_api::command_wit::Number;

pub fn text(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(value) | Arg::Msg(value) => {
            let value = value.trim().to_owned();
            (!value.is_empty()).then_some(value)
        }
        _ => None,
    }
}

pub fn flag(args: &ConsumedArgs, key: &str) -> Option<bool> {
    match args.get_value(key) {
        Arg::Bool(value) => Some(value),
        Arg::Simple(value) => match value.trim().to_lowercase().as_str() {
            "true" | "yes" | "allow" => Some(true),
            "false" | "no" | "deny" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

pub fn integer(args: &ConsumedArgs, key: &str) -> Option<i32> {
    match args.get_value(key) {
        Arg::Num(Ok(Number::Int32(value))) => Some(value),
        Arg::Num(Ok(Number::Int64(value))) => i32::try_from(value).ok(),
        Arg::Simple(value) => value.trim().parse().ok(),
        _ => None,
    }
}
