use config_parser::Config;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Загрузка файла
    let config = Config::load_from_file("config.json")?;

    // Извлечение значения
    if let Ok(value) = config.get("username") {
        println!("Username: {}", value);
    }
    Ok(())
}
