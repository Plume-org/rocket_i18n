//! # Rocket I18N
//!
//! A crate to help you internationalize your Rocket applications.
//!
//! ## Features
//!
//! - Build helpers (with the `build` feature enabled), to update and compile PO files.
//! - Select the correct locale for each request
//! - Provides a macro to internationalize any string.
//!
//! ## Usage
//!
//! First add it to your `Cargo.toml` (you have to use the git version, because we can't publish the latest version on [https://crates.io](crates.io) as it depends on the `master` branch of Rocket):
//!
//! ```toml
//! [dependencies.rocket_i18n]
//! git = "https://github.com/BaptisteGelez/rocket_i18n"
//! rev = "<LATEST COMMIT>"
//! ```
//!
//! Then, in your `main.rs`:
//!
//! ```rust,ignore
//! extern crate rocket;
//! #[macro_use]
//! extern crate rocket_i18n;
//!
//! fn main() {
//!     rocket::ignite()
//!         // Make Rocket manage your translations.
//!         .manage(rocket_i18n::i18n("your-domain", vec![ "en", "fr", "de", "ja" ]));
//!         // Register routes, etc
//! }
//! ```
//!
//! Then in all your requests you'll be able to use the `i18n` macro to translate anything.
//! It takes a `gettext::Catalog` and a string to translate as argument.
//!
//! ```rust,ignore
//! # #[macro_use] extern crate rocket_i18n;
//!
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
//! Because of its design, rocket_i18n is only compatible with askama. You can use
//! the `t` macro in your templates, as long as they have a field called `catalog` to
//! store your catalog.
//!
//! ### Editing the POT
//!
//! For those strings to be translatable you should also add them to the `po/YOUR_DOMAIN.pot` file. To add a simple message, just do:
//!
//! ```po
//! msgid "Hello, world" # The string you used with your filter
//! msgstr "" # Always empty
//! ```
//!
//! For plural forms, the syntax is a bit different:
//!
//! ```po
//! msgid "You have one new notification" # The singular form
//! msgid_plural "You have {{ count }} new notifications" # The plural one
//! msgstr[0] ""
//! msgstr[1] ""
//! ```
//!

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

/// A request guard to get the right translation catalog for the current request
pub struct I18n {
    pub catalog: Catalog,
}

pub type Translations = Vec<(&'static str, Catalog)>;

pub fn i18n(domain: &str, lang: Vec<&'static str>) -> Translations {
    lang.iter().fold(Vec::new(), |mut trans, l| {
        let mo_file = fs::File::open(format!("translations/{}/LC_MESSAGES/{}.mo", l, domain))
            .expect("Couldn't open catalog");
        let cat = Catalog::parse(mo_file).expect(format!("Error while loading catalog ({})", l).as_str());
        trans.push((l, cat));
        trans
    })
}

/// Use this macro to staticaly import translations into your final binary. It's use is similar to
/// [`i18n`](../rocket_i18n/fn.i18n.html)
/// ```rust,ignore
/// # //ignore because there is no translation file provided with rocket_i18n
/// # #[macro_use]
/// # extern crate rocket_i18n;
/// # use rocket_i18n::Translations;
/// let tr: Translations = include_i18n!("plume", ["de", "en", "fr"]);
/// ```
#[macro_export]
macro_rules! include_i18n {
    ( $domain:tt, [$($lang:tt),*] ) => {
        {
            use $crate::Catalog;
            vec![
            $(
                (
                    $lang,
                    Catalog::parse(
                        &include_bytes!(
                            concat!(env!("CARGO_MANIFEST_DIR"), "/translations/", $lang, "/LC_MESSAGES/", $domain, ".mo")
                            )[..]
                        ).expect("Error while loading catalog")
                )
            ),*
            ]
        }
    }
}

#[cfg(feature = "build")]
pub fn update_po(domain: &str, locales: &[&'static str]) {
    use std::{path::Path, process::Command};

    let pot_path = Path::new("po").join(format!("{}.pot", domain));

    for lang in locales {
        let po_path = Path::new("po").join(format!("{}.po", lang.clone()));
        if po_path.exists() && po_path.is_file() {
            println!("Updating {}", lang.clone());
            // Update it
            Command::new("msgmerge")
                .arg("-U")
                .arg(po_path.to_str().unwrap())
                .arg(pot_path.to_str().unwrap())
                .status()
                .map(|s| {
                    if !s.success() {
                        panic!("Couldn't update PO file")
                    }
                })
                .expect("Couldn't update PO file");
        } else {
            println!("Creating {}", lang.clone());
            // Create it from the template
            Command::new("msginit")
                .arg(format!("--input={}", pot_path.to_str().unwrap()))
                .arg(format!("--output-file={}", po_path.to_str().unwrap()))
                .arg("-l")
                .arg(lang)
                .arg("--no-translator")
                .status()
                .map(|s| {
                    if !s.success() {
                        panic!("Couldn't init PO file")
                    }
                })
                .expect("Couldn't init PO file");
        }
    }
}

/// Transforms all the .po files in the `po` directory of your project
#[cfg(feature = "build")]
pub fn compile_po(domain: &str, locales: &[&'static str]) {
    use std::{path::Path, process::Command};

    for lang in locales {
        let po_path = Path::new("po").join(format!("{}.po", lang.clone()));
        let mo_dir = Path::new("translations")
            .join(lang.clone())
            .join("LC_MESSAGES");
        fs::create_dir_all(mo_dir.clone()).expect("Couldn't create MO directory");
        let mo_path = mo_dir.join(format!("{}.mo", domain));

        Command::new("msgfmt")
            .arg(format!("--output-file={}", mo_path.to_str().unwrap()))
            .arg(po_path)
            .status()
            .map(|s| {
                if !s.success() {
                    panic!("Couldn't compile translations")
                }
            })
            .expect("Couldn't compile translations");
    }
}

/// See the crate documentation for information
/// about how to use this macro.
#[macro_export]
macro_rules! i18n {
    ($cat:expr, $msg:expr) => {
        $cat.gettext($msg)
    };
    ($cat:expr, $msg:expr, $plur:expr, $count:expr) => {
        $crate::try_format($cat.ngettext($msg, $plur, $count.clone() as u64), &[ Box::new($count) ])
            .expect("GetText formatting error")
    };

    ($cat:expr, $msg:expr ; $( $args:expr ),*) => {
        $crate::try_format($cat.gettext($msg), &[ $( Box::new($args) ),* ])
            .expect("GetText formatting error")
    };
    ($cat:expr, $msg:expr, $plur:expr, $count:expr ; $( $args:expr ),*) => {
        $crate::try_format($cat.ngettext($msg, $plu, $count.clone() as u64), &[ Box::new($count), $( Box::new($args) ),* ])
            .expect("GetText formatting error")
    };
}

/// Works the same way as `i18n`, but without needing to give a `Catalog`
/// as first argument.
///
/// For use in askama templates.
#[macro_export]
macro_rules! t {
    ($( $args:tt )+) => {
        i18n!(self.catalog, $( $args )+)
    };
}

#[derive(Debug)]
#[doc(hidden)]
pub enum FormatError {
    UnmatchedCurlyBracket,
    InvalidPositionalArgument,
}

#[doc(hidden)]
pub fn try_format<'a>(
    str_pattern: &'a str,
    argv: &[Box<dyn std::fmt::Display + 'a>],
) -> Result<String, FormatError> {
    use std::fmt::Write;

    //first we parse the pattern
    let mut pattern = vec![];
    let mut vars = vec![];
    let mut finish_or_fail = false;
    for (i, part) in str_pattern.split('}').enumerate() {
        if finish_or_fail {
            return Err(FormatError::UnmatchedCurlyBracket);
        }
        if part.contains('{') {
            let mut part = part.split('{');
            let text = part.next().unwrap();
            let arg = part.next().ok_or(FormatError::UnmatchedCurlyBracket)?;
            if part.next() != None {
                return Err(FormatError::UnmatchedCurlyBracket);
            }
            pattern.push(text);
            vars.push(
                argv.get::<usize>(if arg.len() > 0 {
                    arg.parse()
                        .map_err(|_| FormatError::InvalidPositionalArgument)?
                } else {
                    i
                })
                .ok_or(FormatError::InvalidPositionalArgument)?,
            );
        } else {
            finish_or_fail = true;
            pattern.push(part);
        }
    }

    //then we generate the result String
    let mut res = String::with_capacity(str_pattern.len());
    let mut pattern = pattern.iter();
    let mut vars = vars.iter();
    while let Some(text) = pattern.next() {
        res.write_str(text).unwrap();
        if let Some(var) = vars.next() {
            res.write_str(&format!("{}", var)).unwrap();
        }
    }
    Ok(res)
}

#[cfg(test)]
struct FakeCatalog;

#[cfg(test)]
impl FakeCatalog {
    pub fn gettext<'a>(&self, x: &'a str) -> &'a str {
        x
    }
}

#[cfg(test)]
#[test]
fn test_macros() {
    let catalog = FakeCatalog;
    assert_eq!(
        String::from("Hello, John"),
        i18n!(catalog, "Hello, {0}"; "John")
    );
}
