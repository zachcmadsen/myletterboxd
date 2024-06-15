use std::{env, fs, thread};

use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Settings {
    tmdb_api_key: String,
}

fn get_settings() -> anyhow::Result<Settings> {
    let config = Config::builder()
        .add_source(config::File::with_name("local"))
        .build()?;
    let settings = config.try_deserialize()?;
    Ok(settings)
}

#[derive(Deserialize)]
struct SearchMovieResult {
    id: u64,
    release_date: String,
    title: String,
}

#[derive(Deserialize)]
struct SearchMovieResponse {
    results: Vec<SearchMovieResult>,
}

#[derive(Deserialize)]
struct MovieDetailsResponse {
    id: u64,
    release_date: String,
    runtime: u64,
    title: String,
}

#[derive(Deserialize)]
struct DiaryRecord {
    #[serde(rename = "Name")]
    title: String,
    #[serde(rename = "Year")]
    year: String,
    #[serde(rename = "Watched Date")]
    watched_date: String,
}

fn search_movie(
    agent: &ureq::Agent,
    settings: &Settings,
    title: &str,
    year: &str,
) -> anyhow::Result<Vec<SearchMovieResult>> {
    let response = agent
        .get("https://api.themoviedb.org/3/search/movie")
        .query("api_key", &settings.tmdb_api_key)
        .query("query", title)
        .query("year", year)
        .call()?
        .into_json::<SearchMovieResponse>()?;
    Ok(response.results)
}

fn get_movie_details(
    agent: &ureq::Agent,
    settings: &Settings,
    id: u64,
) -> anyhow::Result<MovieDetailsResponse> {
    let response = agent
        .get(&format!("https://api.themoviedb.org/3/movie/{}", id))
        .query("api_key", &settings.tmdb_api_key)
        .call()?
        .into_json::<MovieDetailsResponse>()?;
    Ok(response)
}

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    args.next();
    let Some(filename) = args.next() else {
        eprintln!("error: no input file");
        std::process::exit(1);
    };

    let settings = get_settings()?;
    let agent = ureq::Agent::new();

    let contents = fs::read(filename)?;
    let mut reader = csv::Reader::from_reader(contents.as_slice());

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for record in reader.deserialize().take(5) {
        let record: DiaryRecord = record?;

        thread::sleep(std::time::Duration::from_secs(1));

        eprintln!("info: fetching {} ({})", record.title, record.year);

        let results = search_movie(&agent, &settings, &record.title, &record.year)?;
        // TODO: Could try searching the list for a single exact match.
        if results.len() != 1 {
            eprintln!(
                "warning: skipping {} ({}), found {} results",
                record.title,
                record.year,
                results.len()
            );
            continue;
        }

        let result = &results[0];
        if result.title != record.title || !result.release_date.starts_with(&record.year) {
            eprintln!(
                "warning: skipping {} ({}), unexpected result: {} ({})",
                record.title, record.year, result.title, result.release_date
            );
            continue;
        }

        let details = get_movie_details(&agent, &settings, result.id)?;

        let new_record = NewDiaryRecord {
            id: details.id,
            release_date: details.release_date,
            runtime: details.runtime,
            title: details.title,
            watched_date: record.watched_date,
        };

        writer.serialize(new_record)?;
    }

    Ok(())
}

#[derive(Serialize)]
struct NewDiaryRecord {
    id: u64,
    release_date: String,
    runtime: u64,
    title: String,
    watched_date: String,
}
