use std::{error::Error, fmt};

use crate::{I18n, Translations, ACCEPT_LANG};

use actix_web::{FromRequest, HttpRequest, ResponseError};

#[derive(Debug)]
pub struct MissingTranslationsError(String);

impl fmt::Display for MissingTranslationsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find translations for {}", self.0)
    }
}

impl Error for MissingTranslationsError {
    fn description(&self) -> &str {
        "Could not find translations"
    }
}

impl ResponseError for MissingTranslationsError {
    // this defaults to an empty InternalServerError response
}

pub trait Internationalized {
    fn get(&self) -> Translations;
}

impl<S> FromRequest<S> for I18n
where
    S: Internationalized,
{
    type Config = ();
    type Result = Result<Self, actix_web::Error>;

    fn from_request(req: &HttpRequest<S>, _: &Self::Config) -> Self::Result {
        let state = req.state();
        let langs = state.get();

        let lang = req
            .headers()
            .get(ACCEPT_LANG)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("en")
            .split(",")
            .filter_map(|lang| {
                lang
                    // Get the locale, not the country code
                    .split(|c| c == '-' || c == ';')
                    .nth(0)
            })
            // Get the first requested locale we support
            .find(|lang| langs.iter().any(|l| l.0 == &lang.to_string()))
            .unwrap_or("en");

        match langs.iter().find(|l| l.0 == lang) {
            Some(catalog) => Ok(I18n {
                catalog: catalog.1.clone(),
            }),
            None => Err(MissingTranslationsError(lang.to_owned()).into()),
        }
    }
}
