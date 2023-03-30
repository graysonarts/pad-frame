use once_cell::sync::OnceCell;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug)]
struct AppArgs {
    directory: std::path::PathBuf,
    width: usize,
}

static REGEX: OnceCell<Regex> = OnceCell::new();

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    let args = AppArgs {
        width: pargs
            .opt_value_from_fn("--width", parse_width)?
            .unwrap_or(5),
        directory: pargs.free_from_str()?,
    };

    Ok(args)
}

fn parse_width(s: &str) -> Result<usize, &'static str> {
    s.parse().map_err(|_| "not a number")
}

fn main() {
    REGEX.set(
        Regex::new(r#"^(?P<prefix>.*?)(?P<count>\d+)(?P<suffix>.*?)$"#).expect("Regex is invalid"),
    );
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    for entry in WalkDir::new(&args.directory) {
        let entry = entry.unwrap();
        if !entry.path().is_file() {
            continue;
        }

        if let Err(err) = fix_path_to_width(args.width, entry.path()) {
            eprintln!("Error: {} @ {}", err, entry.path().display());
        }
    }
}

fn fix_path_to_width(width: usize, path: &std::path::Path) -> Result<(), &'static str> {
    let os_filename = path.file_name().ok_or("No filename")?;
    let filename = os_filename.to_str().ok_or("can't convert to string")?;
    let captures = REGEX
        .get()
        .unwrap()
        .captures(filename)
        .ok_or("Doesn't match regex")?;
    if captures["count"].len() == width {
        return Ok(());
    }

    let count: usize = captures["count"]
        .parse()
        .map_err(|_| "Cannot parse count")?;

    let new_filename = format!(
        "{}{:0width$}{}",
        &captures["prefix"],
        count,
        &captures["suffix"],
        width = width
    );
    let new_path = path.parent().ok_or("Can't get parent")?.join(new_filename);

    println!("Will rename {} -> {}", path.display(), new_path.display());
    std::fs::rename(path, new_path).map_err(|_| "Cannot rename")?;
    Ok(())
}
