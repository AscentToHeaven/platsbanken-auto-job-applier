use serde::{Deserialize, Serialize};

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct Search {
    pub positions: i64,
    pub numberOfAds: i64,
    pub offsetLimit: i64,
    pub ads: Vec<Ad>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct Ad {
    pub id: String,
    pub publishedDate: Option<String>,
    pub lastApplicationDate: Option<String>,
    pub title: Option<String>,
    pub occupation: Option<String>,
    pub workplace: Option<String>,
    pub workplaceName: Option<String>,
    pub unspecifiedWorkplace: Option<bool>,
    pub published: Option<bool>,
    pub positions: Option<u16>,
    pub sourceLinks: Vec<String>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Advert {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub publishedDate: Option<String>,
    pub occupation: Option<String>,
    pub company: Company,
    pub logotype: Option<String>,
    pub conditions: Option<String>,
    pub salaryDescription: Option<String>,
    pub salaryType: Option<String>,
    pub workTimeExtent: Option<String>,
    pub employmentType: Option<String>,
    pub duration: Option<String>,
    pub lastApplicationDate: Option<String>,
    pub expirationDate: Option<String>,
    pub positions: Option<u8>,
    pub published: Option<bool>,
    pub ownCar: Option<bool>,
    pub requiresExperience: Option<bool>,
    pub education: Education,
    pub application: Application,
    pub workplace: Workplace,
    pub drivingLicense: Vec<String>,
    pub skills: Vec<Skills>,
    pub languages: Vec<Lang>,
    pub workExperiences: Vec<WorkExp>,
    pub contacts: Vec<Contact>,
    pub keywords: Vec<String>,
}

impl Advert {
    pub fn email(&self) -> Option<String> {
        if self.application.email.is_some() {
            return self.application.email.clone();
        } else {
            return self.contact_email().clone();
        }
    }

    fn contact_email(&self) -> Option<String> {
        if self.contacts.is_empty() {
            return None;
        }

        for c in &self.contacts {
            if c.email.is_some() {
                return Some(c.email.clone().unwrap());
            }
        }

        return None;
    }
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Lang {
    name: String,
    required: bool,
    level: Option<Level>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Skills {
    name: String,
    required: bool,
    level: Option<Level>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WorkExp {
    name: Option<String>,
    required: Option<bool>,
    level: Option<Level>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Level {
    id: String,
    name: String,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Contact {
    name: Option<String>,
    surname: Option<String>,
    position: Option<String>,
    mobileNumber: Option<String>,
    phoneNumber: Option<String>,
    email: Option<String>,
    union: Option<bool>,
    description: Option<String>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Workplace {
    pub name: Option<String>,
    pub street: Option<String>,
    pub postCode: Option<String>,
    pub city: Option<String>,
    pub unspecifiedWorkplace: Option<bool>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub municipality: Option<String>,
    pub longitude: Option<String>,
    pub latitude: Option<String>,
    pub showMap: Option<bool>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Company {
    pub name: Option<String>,
    streetAddress: Option<String>,
    postCode: Option<String>,
    city: Option<String>,
    phoneNumber: Option<String>,
    webAddress: Option<String>,
    email: Option<String>,
    organisationNumber: String,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Education {
    name: Option<String>,
    required: Option<bool>,
    level: Option<String>,
}

#[allow(unused, non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Application {
    mail: Option<String>,
    email: Option<String>,
    webAddress: Option<String>,
    other: Option<String>,
    reference: Option<String>,
    information: Option<String>,
}
