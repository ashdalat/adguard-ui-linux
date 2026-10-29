// Port of adguard_modern/tabs/dns_data.py — DNS module data and server metadata.
use crate::widgets::dns_controls::DnsProvider;

#[derive(Clone)]
pub struct DnsFeature {
    pub key: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
}

#[derive(Clone)]
pub struct Protocol {
    pub name: &'static str,
    pub value: &'static str,
}

#[derive(Clone)]
pub struct DnsServerData {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub upstream: &'static str,
    pub website: &'static str,
    pub protocols: Vec<Protocol>,
    pub features: Vec<DnsFeature>,
}

fn p(name: &'static str, value: &'static str) -> Protocol {
    Protocol { name, value }
}

fn f(key: &'static str, title: &'static str, desc: &'static str) -> DnsFeature {
    DnsFeature { key, title, desc }
}

/// Port of DNS_SERVERS_DATA (insertion order preserved).
pub fn dns_servers_data() -> Vec<DnsServerData> {
    vec![
        DnsServerData {
            id: "system_dns",
            name: "System DNS",
            desc: "DNS server configured in your device's operating system",
            upstream: "default",
            website: "https://adguard.com/",
            protocols: vec![p("Regular", "default")],
            features: vec![],
        },
        DnsServerData {
            id: "adguard_default",
            name: "AdGuard DNS",
            desc: "AdGuard DNS service that blocks ads and trackers",
            upstream: "https://dns.adguard-dns.com/dns-query",
            website: "https://adguard-dns.io/",
            protocols: vec![
                p("Regular", "94.140.14.14"),
                p("DNS-over-HTTPS", "https://dns.adguard-dns.com/dns-query"),
                p("DNS-over-TLS", "tls://dns.adguard-dns.com"),
                p("DNS-over-QUIC", "quic://dns.adguard-dns.com"),
                p("DNS-over-HTTP/3", "h3://dns.adguard-dns.com/dns-query"),
            ],
            features: vec![
                f("ad_blocking", "Ad blocking", "Blocks ads and tracking servers"),
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "adguard_family",
            name: "AdGuard DNS Family Protection",
            desc: "Extended AdGuard DNS service that also blocks adult content and enables Safe Search and Safe Mode",
            upstream: "https://family.adguard-dns.com/dns-query",
            website: "https://adguard-dns.io/",
            protocols: vec![
                p("Regular", "94.140.14.15"),
                p("DNS-over-HTTPS", "https://family.adguard-dns.com/dns-query"),
                p("DNS-over-TLS", "tls://family.adguard-dns.com"),
                p("DNS-over-QUIC", "quic://family.adguard-dns.com"),
                p("DNS-over-HTTP/3", "h3://family.adguard-dns.com/dns-query"),
            ],
            features: vec![
                f("ad_blocking", "Ad blocking", "Blocks ads and tracking servers"),
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
                f("adult_blocking", "Adult content blocking", "Blocks websites with adult content"),
                f(
                    "safe_search",
                    "Safe search",
                    "Enforces safe search for popular search engines and YouTube",
                ),
            ],
        },
        DnsServerData {
            id: "adguard_unfiltered",
            name: "AdGuard DNS Non-filtering",
            desc: "AdGuard DNS service that provides a secure and reliable connection, but doesn't filter anything",
            upstream: "https://unfiltered.adguard-dns.com/dns-query",
            website: "https://adguard-dns.io/",
            protocols: vec![
                p("Regular", "94.140.14.140"),
                p("DNS-over-HTTPS", "https://unfiltered.adguard-dns.com/dns-query"),
                p("DNS-over-TLS", "tls://unfiltered.adguard-dns.com"),
                p("DNS-over-QUIC", "quic://unfiltered.adguard-dns.com"),
                p("DNS-over-HTTP/3", "h3://unfiltered.adguard-dns.com/dns-query"),
            ],
            features: vec![
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "google",
            name: "Google DNS",
            desc: "DNS service offered by Google",
            upstream: "https://dns.google/dns-query",
            website: "https://developers.google.com/speed/public-dns",
            protocols: vec![
                p("Regular", "8.8.8.8"),
                p("DNS-over-HTTPS", "https://dns.google/dns-query"),
                p("DNS-over-TLS", "tls://dns.google"),
            ],
            features: vec![f(
                "encryption",
                "Encryption",
                "Does not let third-parties spy on your browsing activity",
            )],
        },
        DnsServerData {
            id: "cloudflare",
            name: "Cloudflare DNS",
            desc: "Public DNS service offered by Cloudflare",
            upstream: "https://cloudflare-dns.com/dns-query",
            website: "https://1.1.1.1/",
            protocols: vec![
                p("Regular", "1.1.1.1"),
                p("DNS-over-HTTPS", "https://cloudflare-dns.com/dns-query"),
                p("DNS-over-TLS", "tls://1.1.1.1"),
            ],
            features: vec![
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "cloudflare_malware",
            name: "Cloudflare for Families – Malware Blocking",
            desc: "Public DNS service that blocks malware, offered by Cloudflare",
            upstream: "https://security.cloudflare-dns.com/dns-query",
            website: "https://1.1.1.1/family/",
            protocols: vec![
                p("Regular", "1.1.1.2"),
                p("DNS-over-HTTPS", "https://security.cloudflare-dns.com/dns-query"),
            ],
            features: vec![
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "cloudflare_adult",
            name: "Cloudflare for Families – Malware and Adult Content",
            desc: "Public DNS service that blocks both malware and adult content, offered by Cloudflare",
            upstream: "https://family.cloudflare-dns.com/dns-query",
            website: "https://1.1.1.1/family/",
            protocols: vec![
                p("Regular", "1.1.1.3"),
                p("DNS-over-HTTPS", "https://family.cloudflare-dns.com/dns-query"),
            ],
            features: vec![
                f("adult_blocking", "Adult content blocking", "Blocks websites with adult content"),
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "opendns",
            name: "OpenDNS",
            desc: "Public DNS service that blocks malware and phishing, offered by Cisco",
            upstream: "https://doh.opendns.com/dns-query",
            website: "https://www.opendns.com/",
            protocols: vec![
                p("Regular", "208.67.222.222"),
                p(
                    "DNSCrypt",
                    "sdns://AQAAAAAAAAAADjIwOC42Ny4yMjAuMjIwABJkbnNjcnlwdC5vcGVuZG5zLmNvbQ",
                ),
                p("DNS-over-HTTPS", "https://doh.opendns.com/dns-query"),
            ],
            features: vec![
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "opendns_family",
            name: "OpenDNS with Family Shield",
            desc: "Public DNS service that blocks adult content, offered by Cisco",
            upstream: "https://doh.familyshield.opendns.com/dns-query",
            website: "https://www.opendns.com/setup-guide/#familyshield",
            protocols: vec![
                p("Regular", "208.67.222.123"),
                p(
                    "DNSCrypt",
                    "sdns://AQAAAAAAAAAADjIwOC42Ny4yMjAuMTIzABJkbnNjcnlwdC5vcGVuZG5zLmNvbQ",
                ),
                p("DNS-over-HTTPS", "https://doh.familyshield.opendns.com/dns-query"),
            ],
            features: vec![
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f("adult_blocking", "Adult content blocking", "Blocks websites with adult content"),
                f(
                    "safe_search",
                    "Safe search",
                    "Enforces safe search for popular search engines and YouTube",
                ),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
            ],
        },
        DnsServerData {
            id: "quad9",
            name: "Quad9",
            desc: "Public DNS service that blocks malicious domains",
            upstream: "https://dns.quad9.net/dns-query",
            website: "https://www.quad9.net/",
            protocols: vec![
                p("Regular", "9.9.9.9"),
                p(
                    "DNSCrypt",
                    "sdns://AQMAAAAAAAAABzkuOS45LjkACmRucy5xdWFkOS5uZXQvcGx1ZwovZG5zLXF1ZXJ5",
                ),
                p("DNS-over-HTTPS", "https://dns.quad9.net/dns-query"),
                p("DNS-over-TLS", "tls://dns.quad9.net"),
            ],
            features: vec![
                f("secure", "Secure", "Blocks known phishing and other dangerous domains"),
                f(
                    "encryption",
                    "Encryption",
                    "Does not let third-parties spy on your browsing activity",
                ),
                f("no_logs", "No logs", "Does not store your DNS queries logs"),
            ],
        },
    ]
}

/// Complete library from the official AdGuard DNS knowledge base
/// (https://adguard-dns.io/kb/general/dns-providers/).
pub fn known_dns_providers() -> Vec<DnsProvider> {
    vec![
        DnsProvider {
            name: "360 Secure DNS".to_string(),
            desc: "Industry-leading recursive DNS service with security threat protection".to_string(),
        },
        DnsProvider {
            name: "Ali DNS".to_string(),
            desc: "Free recursive DNS service providing fast, stable and secure DNS resolution".to_string(),
        },
        DnsProvider {
            name: "BebasDNS (Default)".to_string(),
            desc: "Free and neutral public resolver that blocks ads, malware, and phishing".to_string(),
        },
        DnsProvider {
            name: "CleanBrowsing (Security Filter)".to_string(),
            desc: "Blocks phishing, spam, and malicious domains".to_string(),
        },
        DnsProvider {
            name: "CleanBrowsing (Family Filter)".to_string(),
            desc: "Blocks adult websites, enforces SafeSearch, and blocks malicious sites".to_string(),
        },
        DnsProvider {
            name: "Control D (Malware + Ads Blocking)".to_string(),
            desc: "Blocks malware, phishing, advertisements and tracking domains".to_string(),
        },
        DnsProvider {
            name: "Control D (Family Friendly)".to_string(),
            desc: "Blocks malware, adult content, and enforces SafeSearch".to_string(),
        },
        DnsProvider {
            name: "CIRA Canadian Shield (Protected)".to_string(),
            desc: "Protects against malware, phishing and cyber threats within Canada".to_string(),
        },
        DnsProvider {
            name: "DNS.SB".to_string(),
            desc: "Fast, privacy-respecting DNS without logging".to_string(),
        },
        DnsProvider {
            name: "Mullvad DNS (Ad-blocking)".to_string(),
            desc: "Privacy-focused DNS service with ad and tracker blocking".to_string(),
        },
        DnsProvider {
            name: "Mullvad DNS (Family)".to_string(),
            desc: "Privacy-focused DNS service blocking ads, trackers, and adult content".to_string(),
        },
        DnsProvider {
            name: "NextDNS".to_string(),
            desc: "Next-generation DNS service with rich security and privacy features".to_string(),
        },
        DnsProvider {
            name: "RethinkDNS".to_string(),
            desc: "Open-source, fast, secure DNS with privacy protection".to_string(),
        },
    ]
}
