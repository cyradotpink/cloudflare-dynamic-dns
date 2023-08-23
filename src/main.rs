use std::pin::Pin;

use cloudflare::endpoints::dns::{DnsContent, DnsRecord};
use futures::{Future, FutureExt, TryFutureExt};

mod cf_wrappers;
mod discord_api;

async fn current_ipv6() -> Result<std::net::Ipv6Addr, anyhow::Error> {
    Ok(reqwest::get("https://ipv6.cyra.pink/")
        .await?
        .text()
        .await?
        .parse()?)
}
async fn current_ipv4() -> Result<std::net::Ipv4Addr, anyhow::Error> {
    Ok(reqwest::get("https://ipv4.cyra.pink/")
        .await?
        .text()
        .await?
        .parse()?)
}

async fn dyndns_update(name: String, zone_id: &str) -> anyhow::Result<()> {
    let list_fut = cf_wrappers::ListDnsRecordsBuilder::new(zone_id)
        .name(name.clone())
        .execute();
    let (current_ipv4, current_ipv6, dns_records) =
        futures::try_join!(current_ipv4(), current_ipv6(), list_fut.map_err(Into::into))?;
    let ipv4_record: &DnsRecord = dns_records
        .iter()
        .find(|v| v.name == name && matches!(v.content, DnsContent::A { .. }))
        .ok_or(anyhow::anyhow!("No IPv4 record found"))?;
    let ipv6_record: &DnsRecord = dns_records
        .iter()
        .find(|v| v.name == name && matches!(v.content, DnsContent::AAAA { .. }))
        .ok_or(anyhow::anyhow!("No IPv6 record found"))?;

    let mut post_futures: Vec<Pin<Box<dyn Future<Output = anyhow::Result<()>>>>> = Vec::new();
    let mut creation_messages: Vec<String> = Vec::new();

    if match ipv4_record.content {
        DnsContent::A { content } => content != current_ipv4,
        _ => false,
    } {
        creation_messages.push(format!("New IPv4 address ({})", current_ipv4));
        post_futures.push(Box::pin(
            cf_wrappers::UpdateDnsRecordBuilder::new(ipv4_record)
                .content(DnsContent::A {
                    content: current_ipv4,
                })
                .execute()
                .map(|v| v.map(|_| ()).map_err(Into::into)),
        ))
    }
    if match ipv6_record.content {
        DnsContent::AAAA { content } => content != current_ipv6,
        _ => false,
    } {
        creation_messages.push(format!("New IPv6 address ({})", current_ipv6));
        post_futures.push(Box::pin(
            cf_wrappers::UpdateDnsRecordBuilder::new(ipv6_record)
                .content(DnsContent::AAAA {
                    content: current_ipv6,
                })
                .execute()
                .map(|v| v.map(|_| ()).map_err(Into::into)),
        ))
    }
    if post_futures.len() <= 0 {
        return Ok(());
    }

    let results = futures::future::join_all(post_futures).await;
    let mut message = format!("Dyndns update ({})", name);
    results
        .into_iter()
        .zip(creation_messages.into_iter())
        .for_each(|(result, msg_part)| match result {
            Ok(_) => message.push_str(&format!("\n- Updated: {}", msg_part)),
            Err(err) => message.push_str(&format!(
                "\n- Update failed: {}; Error was: {}",
                msg_part, err
            )),
        });
    eprintln!("Sending message to webhook:\n{}", message);
    discord_api::execute_webhook(include_str!("../config/discord-webhook.txt"), &message).await?;

    Ok(())
}

async fn async_main() -> anyhow::Result<()> {
    dyndns_update(
        include_str!("../config/name.txt").to_string(),
        include_str!("../config/cloudflare-zone.txt"),
    )
    .await?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async_main())?;

    Ok(())
}
