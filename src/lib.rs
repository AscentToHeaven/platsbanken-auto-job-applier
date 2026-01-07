use anyhow::Result;
use json::JsonValue;
use lettre::SmtpTransport;
use lettre::Transport;
use lettre::message::{Attachment, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use rand::prelude::*;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, process};
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::af_structs::Advert;
pub mod af_structs;

#[derive(Debug)]
struct Email {
    subject: String,
    body: String,
    recipient: String,
}

fn to_json_file_name(id: &str) -> String {
    let mut home = find_home();
    home.push(".config/JobApplier/Jobs/");
    let filename = String::from(id) + ".json";
    home.push(filename);

    return home.display().to_string();
}

pub async fn email_sender(ad: &Advert) -> Option<String> {
    let config = read_config(); //parse config file

    //download the json from AF
    match download_json(ad) {
        Ok(_) => {
            info!("downloaded {}", &ad.id)
        }
        Err(_) => {
            info!("Email already sent. to {}", &ad.id);
            match log(ad).await {
                Ok(_) => (),
                Err(r) => error!("Logging failure: {}", r),
            }
            return Some("past success".to_owned());
        }
    }

    let pl = get_personal_letter();
    debug!("got personal letter");

    //compose an email to send
    let email = Email {
        subject: String::from("Ansökan för '")
            + &ad.title.clone().unwrap_or("arbete".to_string())
            + "'",
        body: pl.into(),
        recipient: ad.email().unwrap(),
    };

    // info!("Email: \n{:?}", email);
    mail(email, &config);

    match log(ad).await {
        Ok(_) => (),
        Err(r) => error!("Logging failure: {}", r),
    }

    return Some("success".to_owned());
}

fn download_json(ad: &Advert) -> Result<()> {
    let json = serde_json::to_string(&ad)?;
    let bytes = json.as_bytes();

    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(to_json_file_name(&ad.id))?;

    file.write_all(&bytes).expect("Failed to write file");
    Ok(())
}

fn mail(email: Email, config: &JsonValue) {
    let subject = email.subject;
    let body = email.body;
    let recipient = email.recipient;
    // Your ProtonMail credentials
    let username = config["SMTP"]["username"].to_string();
    let password = config["SMTP"]["token"].to_string(); // Use the SMTP password from ProtonMail

    info!("Sending Email to {recipient}.");

    let file_path = config["resumePath"].to_string();
    let file_data = fs::read(&file_path).expect("Could not read file");

    let cv = Attachment::new("cv".to_string()).body(file_data, "application/pdf".parse().unwrap());

    // Build the email
    let email = Message::builder()
        .from(username.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(SinglePart::plain(body.to_string()))
                .singlepart(cv),
        )
        .unwrap();

    // Set up the SMTP transport with STARTTLS on port 587
    let creds = Credentials::new(username.to_string(), password.to_string());

    let mail_server = config["SMTP"]["server"].to_string();
    let mailer = SmtpTransport::starttls_relay(&mail_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => info!("Email sent successfully!"),
        Err(e) => error!("Could not send email: {:?}", e),
    }
}

fn read_config() -> JsonValue {
    let mut home_path = match env::home_dir() {
        Some(path) => path,
        None => {
            error!("Failed to parse $HOME path, exiting...");
            process::exit(1);
        }
    };

    home_path.push(".config/JobApplier/config.json");

    let contents = fs::read_to_string(home_path.to_str().unwrap()).expect(
        "
        ERROR: failed to parse config file.
        Please make sure you have a config file ($HOME/.config/JobApplier/config.json)
        refer to the documentation for how to format the file\n",
    );

    json::parse(&contents).expect("ERROR: json failed to parse file.")
}

pub fn get_personal_letter() -> String {
    let mut home_path = match env::home_dir() {
        Some(path) => path,
        None => {
            error!("Failed to parse $HOME path, exiting...");
            process::exit(1);
        }
    };

    home_path.push(".config/JobApplier/pl/");

    let p_letters: Vec<PathBuf> = fs::read_dir(home_path)
        .unwrap()
        .filter_map(|f| match f {
            Ok(file) => Some(file.path()),
            Err(_) => None,
        })
        .collect();

    let mut rng = rand::rng();
    let rand = rng.random_range(0..p_letters.len());

    let letter = fs::read_to_string(p_letters[rand].clone()).expect(
        "
        ERROR: failed to find a personal letter.
        please make sure that you have at least one personal letter inside of $HOME/.config/JobApplier/pl/\n",
    );

    // info!("{}", &letter);

    return letter;
}

fn find_home() -> PathBuf {
    if let Some(path) = env::home_dir() {
        return path;
    } else {
        println!("Failed to parse $HOME path, exiting...");
        process::exit(1);
    }
}

pub fn find_config() -> PathBuf {
    let mut home = find_home();
    home.push(".config/JobApplier");
    return home;
}

pub async fn log(ad: &Advert) -> Result<(), sqlx::Error> {
    let mut config = find_config();
    config.push("log.db");
    let log_path = format!("sqlite:file:{}?mode=rwc", config.display());

    let database = SqlitePool::connect(&log_path).await?;

    // make the table if it doesn't exist
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS log (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                occupation TEXT NOT NULL,
                workTimeExtent TEXT NOT NULL,
                company TEXT NOT NULL,
                city TEXT NOT NULL,
                email TEXT,
                date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
        "#,
    )
    .execute(&database)
    .await?;

    let title = match &ad.title {
        Some(val) => val,
        None => "null",
    };
    let occupation = match &ad.occupation {
        Some(val) => val,
        None => "null",
    };
    let work_time_extent = match &ad.workTimeExtent {
        Some(val) => val,
        None => "null",
    };
    let company = match &ad.company.name {
        Some(val) => val,
        None => "null",
    };
    let city = match &ad.workplace.region {
        Some(val) => val,
        None => "null",
    };
    let email = ad.email().unwrap();

    sqlx::query(
        "INSERT INTO log (id, title, occupation, workTimeExtent, company, city, email)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING;",
    )
    .bind(&ad.id)
    .bind(title)
    .bind(occupation)
    .bind(work_time_extent)
    .bind(company)
    .bind(city)
    .bind(email)
    .execute(&database)
    .await
    .expect("failed write to db");

    Ok(())
}
