use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum RecordType {
    A,
    AAAA,
}

impl RecordType {
    fn as_str(&self) -> &str {
        match self {
            RecordType::A => "A",
            RecordType::AAAA => "AAAA",
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    result: T,
    errors: Vec<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: u32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Serialize)]
struct CreateRecordRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
    comment: String,
}

#[derive(Debug, Serialize)]
struct UpdateRecordRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
    comment: String,
}

pub struct Cloudflare {
    token: String,
    base_url: String,
    client: reqwest::blocking::Client,
}

impl Cloudflare {
    pub fn new(token: String, _domain: String) -> Self {
        Self {
            token,
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn get_zone_id(&self, domain: &str) -> Result<String> {
        let url = format!("{}/zones", self.base_url);
        let response = self.get(&url)?;

        let zones: ApiResponse<Vec<Zone>> = serde_json::from_str(&response)
            .context("Failed to parse zones response")?;

        if !zones.success {
            if let Some(error) = zones.errors.first() {
                anyhow::bail!("API error {}: {}", error.code, error.message);
            }
            anyhow::bail!("Unknown API error");
        }

        // Extract the main domain (last two parts)
        let main_domain = domain
            .split('.')
            .rev()
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(".");

        for zone in zones.result {
            if zone.name == main_domain {
                return Ok(zone.id);
            }
        }

        anyhow::bail!(
            "The specified domain wasn't found in the returned response - does the token have clearance to the DNS?"
        )
    }

    pub fn get_record(&self, zone_id: &str, domain: &str) -> Result<DnsRecord> {
        let url = format!("{}/zones/{}/dns_records", self.base_url, zone_id);
        let response = self.get(&url)?;

        let records: ApiResponse<Vec<DnsRecord>> = serde_json::from_str(&response)
            .context("Failed to parse DNS records response")?;

        if !records.success {
            if let Some(error) = records.errors.first() {
                anyhow::bail!("API error {}: {}", error.code, error.message);
            }
            anyhow::bail!("Unknown API error");
        }

        for record in records.result {
            if record.name == domain {
                return Ok(record);
            }
        }

        anyhow::bail!(
            "Got a zone token, but couldn't locate the domain. Is the subdomain correct?"
        )
    }

    pub fn create_record(
        &self,
        zone_id: &str,
        domain: &str,
        ip: &str,
        record_type: RecordType,
        proxied: bool,
    ) -> Result<DnsRecord> {
        let url = format!("{}/zones/{}/dns_records", self.base_url, zone_id);

        let request = CreateRecordRequest {
            record_type: record_type.as_str().to_string(),
            name: domain.to_string(),
            content: ip.to_string(),
            ttl: 3600,
            proxied,
            comment: format!(
                "Automatic by DDNS - Set {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            ),
        };

        let response = self.post(&url, &request)?;

        let result: ApiResponse<DnsRecord> = serde_json::from_str(&response)
            .context("Failed to parse create record response")?;

        if !result.success {
            if let Some(error) = result.errors.first() {
                anyhow::bail!("API error {}: {}", error.code, error.message);
            }
            anyhow::bail!("Unknown API error");
        }

        Ok(result.result)
    }

    pub fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        ip: &str,
        existing_record: &DnsRecord,
    ) -> Result<DnsRecord> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            self.base_url, zone_id, record_id
        );

        let request = UpdateRecordRequest {
            record_type: existing_record.record_type.clone(),
            name: existing_record.name.clone(),
            content: ip.to_string(),
            ttl: existing_record.ttl,
            proxied: existing_record.proxied,
            comment: format!(
                "Automatic by DDNS - Set {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            ),
        };

        let response = self.put(&url, &request)?;

        let result: ApiResponse<DnsRecord> = serde_json::from_str(&response)
            .context("Failed to parse update record response")?;

        if !result.success {
            if let Some(error) = result.errors.first() {
                anyhow::bail!("API error {}: {}", error.code, error.message);
            }
            anyhow::bail!("Unknown API error");
        }

        Ok(result.result)
    }

    fn get(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send()
            .context("Failed to send GET request")?;

        let status = response.status();
        let body = response.text().context("Failed to read response body")?;

        if !status.is_success() {
            let mut error_msg = format!("HTTP error was received: {}", status);
            
            if status.as_u16() == 400 {
                if let Ok(error_response) = serde_json::from_str::<ApiResponse<serde_json::Value>>(&body) {
                    if let Some(error) = error_response.errors.first() {
                        error_msg.push_str(&format!(
                            "\nDetails ({}): {} (is your token correct?)",
                            error.code, error.message
                        ));
                    }
                }
            }
            
            anyhow::bail!(error_msg);
        }

        Ok(body)
    }

    fn post<T: Serialize>(&self, url: &str, data: &T) -> Result<String> {
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .context("Failed to send POST request")?;

        let status = response.status();
        let body = response.text().context("Failed to read response body")?;

        if !status.is_success() {
            let mut error_msg = format!("HTTP error was received sending the payload: {}", status);
            
            if status.as_u16() == 400 {
                if let Ok(error_response) = serde_json::from_str::<ApiResponse<serde_json::Value>>(&body) {
                    if let Some(error) = error_response.errors.first() {
                        error_msg.push_str(&format!(
                            "\nDetails ({}): {}",
                            error.code, error.message
                        ));
                    }
                }
            }
            
            anyhow::bail!(error_msg);
        }

        Ok(body)
    }

    fn put<T: Serialize>(&self, url: &str, data: &T) -> Result<String> {
        let response = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .context("Failed to send PUT request")?;

        let status = response.status();
        let body = response.text().context("Failed to read response body")?;

        if !status.is_success() {
            let mut error_msg = format!("HTTP error was received sending the payload: {}", status);
            
            if status.as_u16() == 400 {
                if let Ok(error_response) = serde_json::from_str::<ApiResponse<serde_json::Value>>(&body) {
                    if let Some(error) = error_response.errors.first() {
                        error_msg.push_str(&format!(
                            "\nDetails ({}): {}",
                            error.code, error.message
                        ));
                    }
                }
            }
            
            anyhow::bail!(error_msg);
        }

        Ok(body)
    }
}
