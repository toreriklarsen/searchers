use std::path::PathBuf;
use clap::Parser;
use anstyle::{AnsiColor, Color, Style};

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
}

/// CLI options for the indexer
#[derive(Parser, Debug)]
#[command(styles = get_styles())]
#[command(author, version, about)]
pub struct Cli {
    /// Input directory to index
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    pub inputdir: PathBuf,

    /// Enable watch mode
    #[arg(short, long, default_value_t = false)]
    pub watch: bool,

    /// Enable watch mode
    #[arg(short, long, default_value_t = false)]
    pub noindex: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_dir_and_watch_flag() {
        let args = vec![
            "testbin",
            "--inputdir",
            "/tmp/data",
            "--watch"
        ];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.inputdir, PathBuf::from("/tmp/data"));
        assert!(cli.watch);
    }

    #[test]
    fn test_input_dir_without_watch_flag() {
        let args = vec![
            "testbin",
            "--inputdir",
            "/tmp/data"
        ];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.inputdir, PathBuf::from("/tmp/data"));
        assert!(!cli.watch);
    }

    #[test]
    fn test_short_flags() {
        let args = vec![
            "testbin",
            "-i",
            "/tmp/data",
            "-w"
        ];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.inputdir, PathBuf::from("/tmp/data"));
        assert!(cli.watch);
    }
}