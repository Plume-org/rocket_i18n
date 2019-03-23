//! # Rocket I18N
//!
//! A crate to help you internationalize your Rocket or Actix Web applications.
//!
//! It just selects the correct locale for each request, and return the corresponding `gettext::Catalog`.
//!
//! ## Usage
//!
//! First add it to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rocket_i18n = "0.4"
//! gettext-macros = "0.1" # Provides proc-macros to manage translations
//! ```
//!
//! Then, in your `main.rs`:
//!
//! ```rust,ignore
//! # use rocket;
//! use gettext_macros::{compile_i18n, include_i18n, init_i18n};
//!
//! init_i18n!("my_web_app", en, eo, it, pl);
//!
//! fn main() {
//!     rocket::ignite()
//!         // Make Rocket manage your translations.
//!         .manage(include_i18n!());
//!         // Register routes, etc
//! }
//!
//! compile_i18n!();
//! ```
//!
//! Then in all your requests you'll be able to use the `i18n` macro to translate anything.
//! It takes a `gettext::Catalog` and a string to translate as argument.
//!
//! ```rust,ignore
//! use gettext_macros::i18n;
//! use rocket_i18n::I18n;
//!
//! #[get("/")]
//! fn route(i18n: I18n) -> &str {
//!     i18n!(i18n.catalog, "Hello, world!")
//! }
//! ```
//!
//! For strings that may have a plural form, just add the plural and the number of element to the
//! arguments
//!
//! ```rust,ignore
//! i18n!(i18n.catalog, "One new message", "{0} new messages", 42);
//! ```
//!
//! Any extra argument, after a `;`, will be used for formatting.
//!
//! ```rust,ignore
//! let user_name = "Alex";
//! i18n!(i18n.catalog, "Hello {0}!"; user_name);
//! ```
//!
//! When using it with plural, `{0}` will be the number of elements, and other arguments will start
//! at `{1}`.
//!
//! Because of its design, rocket_i18n is only compatible with askama, ructe or compiled templates
//! in general.
//! You can use the `t` macro in your templates, as long as they have a field called `catalog` to
//! store your catalog.


#[cfg(feature = "actix-web")]
extern crate actix_web;
extern crate gettext;
#[cfg(feature = "rocket")]
extern crate rocket;

pub use gettext::*;
use std::fs;

#[cfg(feature = "rocket")]
mod with_rocket;

#[cfg(feature = "actix-web")]
mod with_actix;

#[cfg(feature = "actix-web")]
pub use with_actix::Internationalized;

const ACCEPT_LANG: &'static str = "Accept-Language";

/// A request guard to get the right translation catalog for the current request.
pub struct I18n {
    /// The catalog containing the translated messages, in the correct locale for this request.
    pub catalog: Catalog,
    /// The language of the current request.
    pub lang: &'static str,
}

pub type Translations = Vec<(&'static str, Catalog)>;

/// Loads translations at runtime. Usually used with `rocket::Rocket::manage`.
///
/// Note that the `.mo` files should be present with your binary. If you want to embed them,
/// use `gettext_macros::include_i18n`.
pub fn i18n(domain: &str, lang: Vec<&'static str>) -> Translations {
    lang.iter().fold(Vec::new(), |mut trans, l| {
        let mo_file = fs::File::open(format!("translations/{}/LC_MESSAGES/{}.mo", l, domain))
            .expect("Couldn't open catalog");
        let cat = Catalog::parse(mo_file).expect(format!("Error while loading catalog ({})", l).as_str());
        trans.push((l, cat));
        trans
    })
}

/// Works the same way as `gettext_macros::i18n`, but without needing to give a `gettext::Catalog`
/// as first argument.
///
/// For use in askama templates.
#[macro_export]
macro_rules! t {
    ($( $args:tt )+) => {
        i18n!(self.catalog, $( $args )+)
    };
}
