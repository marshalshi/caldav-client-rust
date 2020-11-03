use reqwest::Client;

use caldav_client::dav_client::DAVClient;
use caldav_client::settings::OC_URL;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let mut c = DAVClient::new(OC_URL.to_string());
    //println!("{:#?}", c);
    c.get_principal(&client).await.unwrap();

    if let Some(mut p) = c.principal {
        //println!("{:#?}", p);
        p.get_calendars(&client).await.unwrap();
        let events = p
            .events(
                &client,
                "20201102T000000Z".to_string(),
                "20201107T000000Z".to_string(),
            )
            .await
            .unwrap();

        for event in events {
            println!("------------------");
            for line in event.lines() {
                println!("{}", line);
            }
        }
    }
}
