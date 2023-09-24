use fluent::{FluentArgs, FluentValue};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use fluent_resmgr::ResourceManager;
use std::str::FromStr;
use std::{env, fs, io, path::PathBuf};
use unic_langid::LanguageIdentifier;

fn get_available_locales() -> Result<Vec<LanguageIdentifier>, io::Error> {
    let mut locales = vec![];

    let res_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("res")
        .join("translations");
    let res_dir = fs::read_dir(res_path)?;
    for entry in res_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                    let langid: LanguageIdentifier = name.parse().expect("Parsing failed.");
                    locales.push(langid);
                }
            }
        }
    }
    Ok(locales)
}

fn main() {
    let resources = vec!["simple.ftl".into(), "errors.ftl".into()];

    // 1. Get the command line arguments.
    let args: Vec<String> = env::args().collect();

    let res_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("res")
        .join("translations");
    let mgr = ResourceManager::new(format!(
        "{}/{{locale}}/{{res_id}}",
        res_path.to_str().unwrap()
    ));

    // 2. If the argument length is more than 1,
    //    take the second argument as a comma-separated
    //    list of requested locales.
    let requested: Vec<LanguageIdentifier> = args.get(2).map_or(vec![], |arg| {
        arg.split(',')
            .map(|s| s.parse().expect("Parsing locale failed."))
            .collect()
    });

    // 3. Negotiate it against the available ones
    let default_locale: LanguageIdentifier = "en-US".parse().expect("Parsing failed.");
    let available = get_available_locales().expect("Retrieving available locales failed.");
    let resolved_locales = negotiate_languages(
        &requested,
        &available,
        Some(&default_locale),
        NegotiationStrategy::Filtering,
    );

    // 5. Get a bundle for given paths and locales.
    let bundle = mgr.get_bundle(
        resolved_locales.into_iter().map(|s| s.to_owned()).collect(),
        resources,
    );

    // 6. Check if the input is provided.
    match args.get(1) {
        Some(input) => {
            // 6.1. Cast it to a number.
            match isize::from_str(input) {
                Ok(i) => {
                    // 6.2. Construct a map of arguments
                    //      to format the message.
                    let mut args = FluentArgs::new();
                    args.set("input", FluentValue::from(i));
                    args.set("value", FluentValue::from(collatz(i)));
                    // 6.3. Format the message.
                    let mut errors = vec![];
                    let msg = bundle.get_message("response-msg").expect("Message exists");
                    let pattern = msg.value().expect("Message has a value");
                    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
                    println!("{}", value);
                }
                Err(err) => {
                    let mut args = FluentArgs::new();
                    args.set("input", FluentValue::from(input.to_string()));
                    args.set("reason", FluentValue::from(err.to_string()));
                    let mut errors = vec![];
                    let msg = bundle
                        .get_message("input-parse-error-msg")
                        .expect("Message exists");
                    let pattern = msg.value().expect("Message has a value");
                    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
                    println!("{}", value);
                }
            }
        }
        None => {
            let mut errors = vec![];
            let msg = bundle
                .get_message("missing-arg-error")
                .expect("Message exists");
            let pattern = msg.value().expect("Message has a value");
            let value = bundle.format_pattern(pattern, None, &mut errors);
            println!("{}", value);
        }
    }
}

fn collatz(n: isize) -> isize {
    match n {
        1 => 0,
        _ => match n % 2 {
            0 => 1 + collatz(n / 2),
            _ => 1 + collatz(n * 3 + 1),
        },
    }
}
