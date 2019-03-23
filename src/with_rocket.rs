use crate::{ACCEPT_LANG, I18n, Translations};

use rocket::{
    http::Status,
    request::{self, FromRequest},
    Outcome, Request, State,
};

impl<'a, 'r> FromRequest<'a, 'r> for I18n {
    type Error = ();

    fn from_request(req: &'a Request) -> request::Outcome<I18n, ()> {
        let langs = &*req
            .guard::<State<Translations>>()
            .expect("Couldn't retrieve translations because they are not managed by Rocket.");

        let lang = req
            .headers()
            .get_one(ACCEPT_LANG)
            .unwrap_or("en")
            .split(",")
            .filter_map(|lang| lang
                // Get the locale, not the country code
                .split(|c| c == '-' || c == ';')
                .nth(0))
            // Get the first requested locale we support
            .find(|lang| langs.iter().any(|l| l.0 == &lang.to_string()))
            .unwrap_or("en");

        match langs.iter().find(|l| l.0 == lang) {
            Some(translation) => Outcome::Success(I18n {
                catalog: translation.1.clone(),
                lang: translation.0,
            }),
            None => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}
