use anyhow::Result;
use json::JsonValue;
use lettre::SmtpTransport;
use lettre::Transport;
use lettre::message::{Attachment, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use reqwest::blocking::get;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, process};

struct Advert {
    url: String,
    web_type: String,
}

struct Email {
    subject: String,
    body: String,
    recipient: String,
}

impl Advert {
    fn get_id(&self) -> String {
        match self.web_type.as_str() {
            "AF" => {
                let mut url = self.url.clone();
                let job_id = url.split_off(51);
                return job_id;
            }
            _ => panic!("ERROR: function get_id called with invalid Advert.web_type"),
        }
    }

    fn get_api(&self) -> String {
        let id = self.get_id();
        return "https://platsbanken-api.arbetsformedlingen.se/jobs/v1/job/".to_owned()
            + id.as_str();
    }

    fn to_json_file_name(&self) -> String {
        let mut home = find_home();
        home.push(".config/JobApplier/Jobs/");
        let filename = self.get_id() + ".json";
        home.push(filename);

        return home.display().to_string();
    }
}

pub fn email_sender(url: &str) -> Option<&str> {
    let config = read_config(); //parse config file

    let job_advert = Advert {
        url: url.into(),
        web_type: "AF".into(),
    };

    //download the json from AF
    match download_json(&job_advert) {
        Ok(_) => println!("Saved advert as json file."),
        Err(_) => {
            println!("Email already sent.");
            let json: JsonValue = get_json(&job_advert);
            match log(&json) {
                Ok(_) => (),
                Err(r) => println!("logging error: {r}"),
            }
            return Some("past success");
        }
    }

    let json: JsonValue = get_json(&job_advert);

    let email_recipient = match find_email(&json) {
        Some(email) => email,
        None => {
            println!("The employer did not fill any email contact, cannot automatically apply.");
            match log(&json) {
                Ok(_) => (),
                Err(r) => println!("logging error: {r}"),
            }
            return None;
        }
    };

    let pl = get_personal_letter();

    //compose an email to send
    let email = Email {
        subject: String::from("Ansökan för '")
            + json["title"].as_str().unwrap_or("Ansökan") //parses the json ad and gets the title
            + "'",
        body: pl.into(),
        recipient: email_recipient.into(),
    };

    mail(&email.subject, &email.body, &email.recipient, &config);

    match log(&json) {
        Ok(_) => (),
        Err(r) => println!("logging error: {r}"),
    }
    return Some("success");
}

fn download_json(job_ad: &Advert) -> Result<()> {
    let url = job_ad.get_api();

    // Send GET request
    let response = get(url)?;

    // Get the response body as bytes
    let bytes = response.bytes()?;

    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(job_ad.to_json_file_name())?;

    file.write_all(&bytes).expect("Failed to write file");
    Ok(())
}

fn mail(subject: &str, body: &str, recipient: &str, config: &JsonValue) {
    // Your ProtonMail credentials
    let username = config["SMTP"]["username"].to_string();
    let password = config["SMTP"]["token"].to_string(); // Use the SMTP password from ProtonMail

    println!("Sending Email to {recipient}.");

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
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Could not send email: {:?}", e),
    }
}

fn get_json(ad: &Advert) -> JsonValue {
    let contents =
        fs::read_to_string(ad.to_json_file_name()).expect("ERROR: failed to read json file");

    let parsed: JsonValue = json::parse(&contents).expect("ERROR: failure to parse json file");
    return parsed;
}

fn find_email(json: &JsonValue) -> Option<String> {
    if json["application"]["email"].is_string() {
        Some(json["application"]["email"].to_string())
    } else if json["application"]["mail"].is_string() {
        Some(json["application"]["mail"].to_string())
    } else {
        None
    }
}

fn read_config() -> JsonValue {
    let mut home_path = match env::home_dir() {
        Some(path) => path,
        None => {
            println!("Failed to parse $HOME path, exiting...");
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

fn get_personal_letter() -> String {
    let mut home_path = match env::home_dir() {
        Some(path) => path,
        None => {
            println!("Failed to parse $HOME path, exiting...");
            process::exit(1);
        }
    };

    home_path.push(".config/JobApplier/personal_letter.txt");

    return fs::read_to_string(home_path.to_str().unwrap()).expect(
        "
        ERROR: failed to parse personal letter file.
        Please make sure you have a config file ($HOME/.config/JobApplier/personal_letter.txt)
        refer to the documentation for how to format the file\n",
    );
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

#[tokio::main]
pub async fn log(json: &JsonValue) -> Result<(), sqlx::Error> {
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

    let id = match json["id"].as_str() {
        Some(val) => val.parse::<u32>().unwrap_or(0),
        None => 0,
    };
    let title = match json["title"].as_str() {
        Some(val) => val,
        None => "empty",
    };
    let occupation = match json["occupation"].as_str() {
        Some(val) => val,
        None => "empty",
    };
    let work_time_extent = match json["workTimeExtent"].as_str() {
        Some(val) => val,
        None => "empty",
    };
    let company = match json["company"]["name"].as_str() {
        Some(val) => val,
        None => "empty",
    };
    let city = match json["workplace"]["region"].as_str() {
        Some(val) => val,
        None => "empty",
    };
    let email = find_email(&json).unwrap_or("none".to_string());

    sqlx::query(
        "INSERT INTO log (id, title, occupation, workTimeExtent, company, city, email)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING;",
    )
    .bind(id)
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
