use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::LevelFilter;

pub fn init(level_filter: LevelFilter) -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .trace(Color::Cyan)
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] \x1b[0m{}",
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(level_filter)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
