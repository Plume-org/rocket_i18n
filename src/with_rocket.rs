use crate::{I18n, Translations, ACCEPT_LANG};

use rocket::{
    http::Status,
    request::{self, FromRequest},
    Outcome, Request, State,
};

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for I18n {
    type Error = ();

    async fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let langs = req
            .guard::<State<Translations>>()
            .await
            .expect("Couldn't retrieve translations because they are not managed by Rocket.");

        let lang = req
            .headers()
            .get_one(ACCEPT_LANG)
            .unwrap_or("en")
            .split(',')
            .filter_map(|lang| {
                lang
                    // Get the locale, not the country code
                    .split(|c| c == '-' || c == ';')
                    .next()
            })
            // Get the first requested locale we support
            .find(|lang| langs.iter().any(|l| l.0 == *lang))
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
