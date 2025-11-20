use applier::{email_sender, log};
use chrono::Utc;
use clap::Parser;
use json::JsonValue;
use reqwest::blocking::Client;
use serde_json::json;
use std::process;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "Jonkoping")]
    region: String,

    #[arg(short, long, default_value = "servering")]
    search: String,
}

fn main() {
    let arguments = Args::parse();

    let body = build_body(arguments);

    let response = web_push_to_text(body);
    let json = text_to_json(response.unwrap());

    let ids = get_list(&json);
    let urls = ids_to_url(ids);

    for i in 0..urls.len() {
        println!("\nProcessing {}.", &urls[i]);
        email_sender(&urls[i]);
    }
}

fn web_push_to_text(body: String) -> Result<String, reqwest::Error> {
    let client = Client::new();

    let res = client
        .post("https://platsbanken-api.arbetsformedlingen.se/jobs/v1/search")
        .header("Content-Type", "application/json")
        .body(body)
        .send()?;

    let text = res.text();
    return text;
}

fn text_to_json(text: String) -> JsonValue {
    let json: JsonValue = json::parse(&text).expect("Failure to parse text into json");
    return json;
}

fn get_list(json: &JsonValue) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();

    let ad_count = json["numberOfAds"].as_usize();

    for item in 0..ad_count.unwrap() {
        let id = &json["ads"][item]["id"].as_str().unwrap();
        list.push(id.to_string());
    }

    return list;
}

fn ids_to_url(list: Vec<String>) -> Vec<String> {
    let mut url_list: Vec<String> = Vec::new();
    for i in 0..list.len() {
        let full_url =
            "https://arbetsformedlingen.se/platsbanken/annonser/".to_owned() + list[i].as_str();
        url_list.push(full_url)
    }
    return url_list;
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
