use std::{env, fs};

use config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    pub tmdb_api_key: String,
}

fn get_settings() -> anyhow::Result<Settings> {
    let config = Config::builder()
        .add_source(config::File::with_name("local"))
        .build()?;
    let settings = config.try_deserialize()?;
    Ok(settings)
}

#[derive(Debug, Deserialize)]
struct Movie {
    id: u64,
    title: String,
    release_date: String,
}

#[derive(Debug, Deserialize)]
struct SearchMovieResponse {
    results: Vec<Movie>,
}

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    args.next();

    let Some(filename) = args.next() else {
        eprintln!("error: no input file");
        std::process::exit(1);
    };

    let settings = get_settings()?;

    let contents = fs::read_to_string(filename)?;
    for line in contents.lines().skip(1) {
        let mut fields = line.split(',');
        fields.next();

        let title = fields.next().unwrap();
        let year = fields.next().unwrap();

        let response = ureq::get("https://api.themoviedb.org/3/search/movie")
            .query("api_key", &settings.tmdb_api_key)
            .query("query", title)
            .query("year", year)
            .call()?
            .into_json::<SearchMovieResponse>()?;

        println!("{:#?}", response.results);

        break;
    }

    Ok(())
}
