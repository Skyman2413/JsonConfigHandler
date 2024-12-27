use config_parser::Config;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Загрузка файла
    let mut config = Config::load_from_file("config.json");

    // Извлечение значения
    if let Some(value) = config.get_value("username")? {
        println!("Username: {}", value);
    }

    // Изменение значения
    config.set_value("username", "new_user")?;

    // Сохранение в файл
    config.save_to_file("config.json")?;

    Ok(())
}
