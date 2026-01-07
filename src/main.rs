use applier::{
    af_structs::{self, Ad, Advert},
    email_sender,
};
use chrono::Utc;
use clap::Parser;
use reqwest::Client;
use serde_json::json;
use std::process;
use tracing::{Level, debug, error, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "Jonkoping")]
    region: String,

    #[arg(short, long, default_value = "servering")]
    search: String,

    #[arg(long, default_value = "INFO")]
    logging: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Args::parse();

    let ll = match arguments.logging.as_ref() {
        "INFO" => Level::INFO,
        "TRACE" => Level::TRACE,
        "ERROR" => Level::ERROR,
        "DEBUG" => Level::DEBUG,
        "WARN" => Level::WARN,
        _ => {
            println!("Invalid logging level");
            Level::INFO
        }
    };

    let _sub = tracing_subscriber::fmt().with_max_level(ll).init();

    let body = build_body(arguments);
    debug!("made body");

    let response = web_push_to_text(body).await?;
    debug!("got response");

    let ads = fetch_ads(&response.ads).await?;
    let ads = ads
        .iter()
        .filter(|ad| <Advert as Clone>::clone(&ad).email().is_some());
    debug!("got and filtered ads");

    for ad in ads {
        info!("Processing {}", ad.id);
        email_sender(&ad).await;
    }

    Ok(())
}

async fn fetch_ads(list: &Vec<Ad>) -> Result<Vec<Advert>, Box<dyn std::error::Error>> {
    let client = Client::new();

    let mut adverts: Vec<Advert> = Vec::new();
    for a in list {
        let res = client
            .get(format!(
                "https://platsbanken-api.arbetsformedlingen.se/jobs/v1/job/{}",
                a.id
            ))
            .send()
            .await?;

        let txt = res.text().await?;

        match serde_json::from_str(&txt) {
            Ok(v) => adverts.push(v),
            Err(e) => {
                error!("Cannot deserialize response.\n{}\n{}", e, a.id);
            }
        };
    }

    return Ok(adverts);
}

async fn web_push_to_text(body: String) -> Result<af_structs::Search, Box<dyn std::error::Error>> {
    let client = Client::new();

    let res = client
        .post("https://platsbanken-api.arbetsformedlingen.se/jobs/v1/search")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    let text = res.text().await?;
    let search: af_structs::Search = serde_json::from_str(&text)?;

    Ok(search)
}

fn build_body(arguments: Args) -> String {
    let search = arguments.search.as_str();
    let region = match arguments.region.as_str() {
        "Jonkoping" => "KURg_KJF_Lwc",
        "Skovde" => "fqAy_4ji_Lz2",
        _ => {
            println!("ERROR: invalid region.");
            process::exit(1);
        }
    };

    let now = Utc::now();
    let date = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let body = json!({
        "filters":[{
            "type":"freetext",
            "value": search
            },
            {
                "type":"municipality",
                "value": region
            }],
        "fromDate":null,
        "order":"relevance",
        "maxRecords":25,
        "startIndex":0,
        "toDate":date,
        "source":"pb"});

    return body.to_string();
}
