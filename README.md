# Rocket I18N [![Build Status](https://travis-ci.org/BaptisteGelez/rocket_i18n.svg?branch=master)](https://travis-ci.org/BaptisteGelez/rocket_i18n)

A crate to help you internationalize your Rocket applications.

## Features

- Build helpers (with the `build` feature enabled), to update and compile PO files.
- Select the correct locale for each request
- Provides a macro to internationalize any string.

## Usage

First add it to your `Cargo.toml` (you have to use the git version, because we can't publish the latest version on [https://crates.io](crates.io) as it depends on the `master` branch of Rocket):

```toml
[dependencies.rocket_i18n]
git = "https://github.com/BaptisteGelez/rocket_i18n"
rev = "<LATEST COMMIT>"
```

Then, in your `main.rs`:

```rust
extern crate rocket;
#[macro_use]
extern crate rocket_i18n;

fn main() {
    rocket::ignite()
        // Make Rocket manage your translations.
        .manage(rocket_i18n::i18n("your-domain", vec![ "en", "fr", "de", "ja" ]));
        // Register routes, etc
}
```

Then in all your requests you'll be able to use the `i18n` macro to translate anything.
It takes a `gettext::Catalog` and a string to translate as argument.

```rust
# #[macro_use] extern crate rocket_i18n;

use rocket_i18n::I18n;

#[get("/")]
fn route(i18n: I18n) -> &str {
    i18n!(i18n.catalog, "Hello, world!")
}
```

For strings that may have a plural form, just add the plural and the number of element to the
arguments

```rust
i18n!(i18n.catalog, "One new message", "{0} new messages", 42);
```

Any extra argument, after a `;`, will be used for formatting.

```rust
let user_name = "Alex";
i18n!(i18n.catalog, "Hello {0}!"; user_name);
```

When using it with plural, `{0}` will be the number of elements, and other arguments will start
at `{1}`.

Because of its design, `rocket_i18n` is only compatible with askama. You can use
the `t` macro in your templates, as long as they have a field called `catalog` to
store your catalog.

### Generating the POT

For those strings to be translatable you need to extract them in a `po/YOUR_DOMAIN.pot` file.
You can use the [`xtr`](https://github.com/woboq/tr) tool.

```sh
RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo install xtr
xtr src/main.rs -o po/your-domain.pot -k i18n:2,3
```

RUSTFLAGS is optional, to annotate the .pot files with line numbers.
`-k i18n:2,3` is telling xtr to extract strings from the `i18n!` macro second and third argument.

### Using with Actix Web

First, disable the default features so it doesn't pull in all of Rocket.
```toml
[dependencies.rocket_i18n]
version = "0.2"
default-features = false
features = ["actix-web"]
```

Then add it to your application.
```rust
#[macro_use]
extern crate rocket_i18n;

use rocket_i18n::{I18n, Internationalized, Translations};

fn route_handler(i18n: I18n) -> &str {
    i18n!(i18n.catalog, "Hello, world!")
}

#[derive(Clone)]
struct MyState {
    translations: Translations,
}

impl Internationalized for MyState {
    fn get(&self) -> Translations {
        self.translations.clone()
    }
}

fn main() {
    let state = MyState {
        translations: rocket_i18n::i18n("your-domain", vec![ "en", "fr", "de", "ja" ]);
    };

    App::with_state(state)
        .resource("", |r| r.with(route_handler))
        .finish();
}
```
