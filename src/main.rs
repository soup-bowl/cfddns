mod cloudflare;

use anyhow::{Context, Result};
use clap::Parser;
use cloudflare::{Cloudflare, RecordType};

/// Cloudflare Dynamic DNS (CDDNS) - Updates your Cloudflare DNS records with your current IP
#[derive(Parser, Debug)]
#[command(name = "cddns")]
#[command(author = "soup-bowl <code@soupbowl.io>")]
#[command(version)]
#[command(about = "Cloudflare Dynamic DNS updater", long_about = None)]
struct Args {
    /// Cloudflare API token
    #[arg(short, long, env = "CF_TOKEN")]
    token: String,

    /// Domain to update (FQDN)
    #[arg(short, long, env = "CF_DOMAIN")]
    domain: String,

    /// Use IPv6 (AAAA record) instead of IPv4 (A record)
    #[arg(long, env = "CF_IPV6")]
    ipv6: bool,

    /// Enable Cloudflare proxy for new records
    #[arg(short, long, env = "CF_PROXY")]
    proxy: bool,

    /// Enable debug output
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cf = Cloudflare::new(args.token, args.domain.clone());

    // Get zone token
    let zone_id = cf
        .get_zone_id(&args.domain)
        .context("Failed to get zone ID")?;
    
    if args.debug {
        println!("\x1b[94mDebug\x1b[0m: Fetched zone token {}", zone_id);
    }

    // Get current IP
    let current_ip = get_ip(args.ipv6)?;
    if args.debug {
        println!("\x1b[94mDebug\x1b[0m: Current IP address: {}", current_ip);
    }

    // Get DNS record or create if it doesn't exist
    match cf.get_record(&zone_id, &args.domain) {
        Ok(record) => {
            if args.debug {
                println!(
                    "\x1b[94mDebug\x1b[0m: Fetched DNS record ({}/{})",
                    record.name, record.content
                );
            }

            // Update existing record
            let result = cf
                .update_record(&zone_id, &record.id, &current_ip, &record)
                .context("Failed to update DNS record")?;

            if args.debug {
                println!(
                    "\x1b[94mDebug\x1b[0m: Updated DNS record ({}/{})",
                    result.name, result.content
                );
            }

            println!(
                "\x1b[92mSuccess\x1b[0m: Your address {} has been changed to the IP {}",
                result.name, result.content
            );
        }
        Err(_) => {
            if args.debug {
                println!("\x1b[94mDebug\x1b[0m: No record was found. Creating a new one...");
            }

            let record_type = if args.ipv6 {
                RecordType::AAAA
            } else {
                RecordType::A
            };

            let result = cf
                .create_record(&zone_id, &args.domain, &current_ip, record_type, args.proxy)
                .context("Failed to create DNS record")?;

            if args.debug {
                println!(
                    "\x1b[94mDebug\x1b[0m: Created new DNS record ({}/{})",
                    result.name, result.content
                );
            }

            println!(
                "\x1b[92mSuccess\x1b[0m: Your address {} has been changed to the IP {}",
                result.name, result.content
            );
        }
    }

    Ok(())
}

fn get_ip(ipv6: bool) -> Result<String> {
    let url = if ipv6 {
        "https://6.ident.me/"
    } else {
        "https://4.ident.me/"
    };

    let response = reqwest::blocking::get(url)
        .context("Failed to retrieve IP address")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failure retrieving IP address: HTTP {}",
            response.status()
        );
    }

    let ip = response.text().context("Failed to read IP address response")?;
    Ok(ip.trim().to_string())
}
