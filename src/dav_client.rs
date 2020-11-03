use minidom::Element;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, Result};
use url::Url;

use crate::principal::Principal;
use crate::settings;
use crate::utils::find_elem;

static DAVCLIENT_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:">
       <d:prop>
           <d:current-user-principal />
       </d:prop>
    </d:propfind>
"#;

#[derive(Debug)]
pub struct DAVClient {
    url: Url,
    pub principal: Option<Principal>,
}

impl DAVClient {
    pub fn new(string_url: String) -> Self {
        let url = Url::parse(&string_url).expect("String URL parsing error");
        DAVClient {
            url,
            principal: None,
        }
    }

    pub async fn get_principal(&mut self, client: &Client) -> Result<()> {
        if self.principal.is_none() {
            let method = Method::from_bytes(b"PROPFIND")
                .expect("cannot create PROPFIND method.");

            let res = client
                .request(method, self.url.as_str())
                .header("Depth", 0)
                .header(CONTENT_TYPE, "application/xml")
                .basic_auth(settings::USERNAME, Some(settings::PASSWD))
                .body(DAVCLIENT_BODY)
                .send()
                .await?;
            let text = res.text().await?;

            let root: Element = text.parse().unwrap();
            let principal =
                find_elem(&root, "current-user-principal".to_string()).unwrap();
            let principal_href =
                find_elem(principal, "href".to_string()).unwrap();
            let h_str = principal_href.text();

            let mut url_clone = self.url.clone();
            url_clone.set_path(&h_str);
            let p = Principal::new(url_clone);
            self.principal = Some(p);
        }

        Ok(())
    }
}
