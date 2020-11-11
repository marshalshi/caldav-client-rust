use minidom::Element;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, Result};
use std::collections::HashMap;
use strfmt::strfmt;
use url::Url;

use crate::settings;
use crate::utils::{find_elem, find_elems};

static HOMESET_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" >
      <d:self/>
      <d:prop>
        <c:calendar-home-set />
      </d:prop>
    </d:propfind>
"#;

static CAL_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" >
       <d:prop>
         <d:displayname />
         <d:resourcetype />
         <c:supported-calendar-component-set />
       </d:prop>
    </d:propfind>
"#;

// TODO We only fetch `VEVENT` here but this value should from CAL_BODY result.
static EVENT_BODY_TEMP: &str = r#"
    <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
      <d:prop>
        <d:getetag />
        <c:calendar-data />
      </d:prop>
      <c:filter>
        <c:comp-filter name="VCALENDAR">
          <c:comp-filter name="VEVENT" >
            <c:time-range start="{start}" end="{end}" />
          </c:comp-filter>
        </c:comp-filter>
      </c:filter>
    </c:calendar-query>
"#;

#[derive(Debug)]
pub struct Principal {
    url: Url,
    calendar_home_set_url: Option<Url>,
    pub calendars: Vec<String>, // Only save path here.
}

impl Principal {
    pub fn new(url: Url) -> Self {
        Principal {
            url,
            calendar_home_set_url: None,
            calendars: Vec::new(),
        }
    }

    pub async fn get_cal_home_set(&mut self, client: &Client) -> Result<()> {
        let method = Method::from_bytes(b"PROPFIND")
            .expect("cannot create PROPFIND method. principal");

        let res = client
            .request(method, self.url.as_str())
            .header("Depth", 0)
            .header(CONTENT_TYPE, "application/xml")
            .basic_auth(settings::USERNAME, Some(settings::PASSWD))
            .body(HOMESET_BODY)
            .send()
            .await?;

        let text = res.text().await?;

        let root: Element = text.parse().unwrap();
        let chs = find_elem(&root, "calendar-home-set".to_string()).unwrap();
        let chs_href = find_elem(chs, "href".to_string()).unwrap();
        let chs_str = chs_href.text();

        let mut chs_url = self.url.clone();
        chs_url.set_path(&chs_str);
        self.calendar_home_set_url = Some(chs_url);
        Ok(())
    }

    pub async fn get_calendars(&mut self, client: &Client) -> Result<()> {
        if self.calendar_home_set_url.is_none() {
            self.get_cal_home_set(client).await?;
            if let Some(chs_url) = &self.calendar_home_set_url {
                let method = Method::from_bytes(b"PROPFIND")
                    .expect("cannot create PROPFIND method.");

                let res = client
                    .request(method, chs_url.as_str())
                    .header("Depth", 1)
                    .header(CONTENT_TYPE, "application/xml")
                    .basic_auth(settings::USERNAME, Some(settings::PASSWD))
                    .body(CAL_BODY)
                    .send()
                    .await?;
                let text = res.text().await?;

                let root: Element = text.parse().unwrap();
                let reps = find_elems(&root, "response".to_string());
                for rep in reps {
                    // TODO checking `displayname` here but may there are better way
                    let displayname = find_elem(rep, "displayname".to_string())
                        .unwrap()
                        .text();
                    if displayname == "" {
                        continue;
                    }

                    let href = find_elem(rep, "href".to_string()).unwrap();
                    let href_text = href.text();
                    self.calendars.push(href_text.to_string());
                }
            }
        }

        Ok(())
    }

    pub async fn events(
        &self,
        client: &Client,
        start: String,
        end: String,
    ) -> Result<Vec<String>> {
        let mut events = Vec::new();

        let mut vars = HashMap::new();
        vars.insert("start".to_string(), &start);
        vars.insert("end".to_string(), &end);

        for cal in &self.calendars {
            let method =
                Method::from_bytes(b"REPORT").expect("cannot create method.");
            let mut url = self.url.clone();
            url.set_path(cal);
            let ebody = strfmt(EVENT_BODY_TEMP, &vars).unwrap();

            let res = client
                .request(method, url.as_str())
                .header("Depth", 1)
                .header(CONTENT_TYPE, "application/xml")
                .basic_auth(settings::USERNAME, Some(settings::PASSWD))
                .body(ebody)
                .send()
                .await?;
            let text = res.text().await?;

            let root: Element = text.parse().unwrap();
            let datas = find_elems(&root, "calendar-data".to_string());
            for data in datas {
                let etext = data.text();
                events.push(etext);
            }
        }

        Ok(events)
    }
}
