//! ctmpnumis.fr updater

use anyhow::Context;
use futures::lock::Mutex;
use notify_rust::*;
use once_cell::sync::OnceCell;
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::time;
use tray_item::TrayItem;

/// Website entrypoint URL
const WEBSITE_URL: &'static str = "https://www.ctmpnumis.fr/en/";
/// Last value of first listing's URL
static LAST_VALUE: OnceCell<Mutex<String>> = OnceCell::new();

/// Checks if the website received an update
async fn is_new_update(client: &Client) -> anyhow::Result<bool> {
    // retrieve HTML body
    let body_str = client
        .get(WEBSITE_URL)
        .send()
        .await
        .context("Failed to invoke client.get(url).send()")?
        .text()
        .await
        .context("Failed to invoke body.text()")?;

    // extract href from the first item
    // { } block is used, because...
    let href = {
        // ...we need to restrict this binding's scope
        // scraper::Html cannot be sent between threads safely
        // Rust compiler disallows manual std::mem::drop to prevent nested panics
        let body = Html::parse_document(&body_str);

        // prepare a selector...
        let selector = Selector::parse(".first a[href]").unwrap();

        // ...and use it to extract target href
        match body
            .select(&selector)
            .next()
            .map(|x| x.value())
            .and_then(|x| x.attr("href"))
        {
            Some(elem) => elem,
            None => {
                anyhow::bail!("Selector failed");
            }
        }
        .to_string()
    };

    // extract OnceCell contents
    match LAST_VALUE.get() {
        // OnceCell had been set before
        Some(value_mutex) => {
            // extract Mutex contents
            let mut value = value_mutex.lock().await;

            // compare new href value with the previous one
            if href == *value {
                // they're the same, no notification will be emitted
                Ok(false)
            } else {
                // they're different, so the website received an update
                // replace the old Mutex value...
                *value = href.to_owned();
                // ...and display a notification
                Ok(true)
            }
        }
        // OnceCell had not been set before
        None => {
            // initialize the OnceCell
            LAST_VALUE
                .set(Mutex::new(href.to_owned()))
                .expect("Failed to initialize once_cell");

            // there's no need to show a notification
            Ok(false)
        }
    }
}

/// Starts the application
#[tokio::main]
async fn main() {
    // we need to spawn a new thread, since the tray icon thread is fussy on MacOS
    // (vide tray-item documentation)
    std::thread::spawn(|| {
        // create a new runtime...
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // ...and use it to block on an async { } block
        rt.block_on(async {
            // create new reqwest::Client
            let client = Client::new();
            // set website polling interval
            let mut interval = time::interval(time::Duration::from_secs(10));

            // notify the user that the app is now working in the background
            Notification::new()
                .summary("ctmpnumis.fr")
                .body("Aplikacja została uruchomiona w tle")
                .show()
                .expect("Failed to invoke Notification.show()");

            // repeat forever:
            loop {
                // 1. check if the website received an update
                match is_new_update(&client).await {
                    // 2. if it didn't, do nothing
                    Ok(false) => { /* pass */ }
                    // 3. if it did, display a notification
                    Ok(true) => {
                        Notification::new()
                            .summary("ctmpnumis.fr")
                            .body("Dostępne są nowe ogłoszenia")
                            .timeout(Timeout::Never)
                            .show()
                            .expect("Failed to invoke Notification.show()");
                    }
                    // 4. if an unexpected error occurred, print it to stderr
                    Err(e) => eprintln!("{:#?}", e),
                }

                // 5. wait for a set interval
                interval.tick().await;
            }
        })
    });

    // create a menu bar icon with a Quit button
    let mut tray = TrayItem::new("ctmpnumis.fr", "").unwrap();
    let inner = tray.inner_mut();
    inner.add_quit_item("Wyłącz");
    inner.display();
}
