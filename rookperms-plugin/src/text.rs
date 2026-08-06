use pumpkin_plugin_api::command::CommandSender;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::text::TextComponent;

const PREFIX: &str = "[RookPerms] ";

pub fn colored(message: &str, color: NamedColor) -> TextComponent {
    let component = TextComponent::text(message);
    component.color_named(color);
    component
}

pub fn success(message: &str) -> TextComponent {
    prefixed(message, NamedColor::Green)
}

pub fn info(message: &str) -> TextComponent {
    prefixed(message, NamedColor::Gray)
}

pub fn failure(message: &str) -> TextComponent {
    prefixed(message, NamedColor::Red)
}

pub fn entry(label: &str, value: &str) -> TextComponent {
    let component = colored(&format!("  {label}: "), NamedColor::Gray);
    component.add_child(colored(value, NamedColor::White));
    component
}

pub fn heading(title: &str) -> TextComponent {
    let component = colored(title, NamedColor::Aqua);
    component.bold(true);
    component
}

pub fn usage(command: &str, description: &str) -> TextComponent {
    let component = colored(&format!("  {command}"), NamedColor::Yellow);
    component.click_suggest_command(command);
    component.add_child(colored(&format!(" - {description}"), NamedColor::Gray));
    component
}

pub fn send(sender: &CommandSender, component: TextComponent) {
    sender.send_message(component);
}

pub fn send_success(sender: &CommandSender, message: &str) {
    send(sender, success(message));
}

pub fn send_info(sender: &CommandSender, message: &str) {
    send(sender, info(message));
}

fn prefixed(message: &str, color: NamedColor) -> TextComponent {
    let component = colored(PREFIX, NamedColor::DarkAqua);
    component.add_child(colored(message, color));
    component
}
