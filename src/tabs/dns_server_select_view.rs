// Port of adguard_modern/tabs/dns_server_select_view.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use regex::Regex;

use crate::backend;
use crate::i18n;
use crate::tabs::dns_data::{dns_servers_data, DnsServerData};
use crate::widgets::common_controls::BackButton;
use crate::widgets::dns_controls::{AddCustomDnsDialog, DnsServerRow};

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn proxy_yaml() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/proxy.yaml")
}

fn root_proxy_yaml() -> PathBuf {
    PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml")
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    c_lay: gtk4::Box,
    group_headers: RefCell<HashMap<String, gtk4::Label>>,
    rows: RefCell<HashMap<String, DnsServerRow>>,
    custom_container: gtk4::Box,
    btn_add_custom: gtk4::Button,
    btn_select_list: gtk4::Button,
    current_upstream: RefCell<String>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    server_changed_cb: RefCell<Option<Box<dyn Fn(String, String)>>>,
    drilldown_cb: RefCell<Option<Box<dyn Fn(&DnsServerData)>>>,
}

#[derive(Clone)]
pub struct DnsServerSelectView {
    inner: Rc<Inner>,
}

impl DnsServerSelectView {
    pub fn new() -> DnsServerSelectView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("dns-protection-view");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // 1. Back button.
        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);

        // 2. Title row.
        let lbl_title = gtk4::Label::new(Some("DNS server"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);

        // 3. Subtitle.
        let lbl_desc = gtk4::Label::new(Some(
            "Allows to select a DNS server to resolve DNS requests, block ads and trackers, and encrypt DNS traffic",
        ));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        root.append(&spacer(6));

        // 4. Scroll area for server groups.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        container.add_css_class("transparent-container");
        container.set_margin_end(8);
        container.set_margin_bottom(20);
        scroll.set_child(Some(&container));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_desc,
            c_lay: container,
            group_headers: RefCell::new(HashMap::new()),
            rows: RefCell::new(HashMap::new()),
            custom_container: gtk4::Box::new(gtk4::Orientation::Vertical, 4),
            btn_add_custom: gtk4::Button::new(),
            btn_select_list: gtk4::Button::new(),
            current_upstream: RefCell::new("default".to_string()),
            back_cb: RefCell::new(None),
            server_changed_cb: RefCell::new(None),
            drilldown_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.root.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        // Top item: System DNS (standalone).
        add_server_row(&inner, "system_dns", false);

        inner.c_lay.append(&spacer(16));

        // Group 1: AdGuard DNS servers.
        add_group_header(&inner, "grp_adguard", "AdGuard DNS servers");
        for id in ["adguard_default", "adguard_family", "adguard_unfiltered"] {
            add_server_row(&inner, id, true);
        }

        inner.c_lay.append(&spacer(16));

        // Group 2: Public servers.
        add_group_header(&inner, "grp_public", "Public servers");
        for id in [
            "google",
            "cloudflare",
            "cloudflare_malware",
            "cloudflare_adult",
            "opendns",
            "opendns_family",
            "quad9",
        ] {
            add_server_row(&inner, id, true);
        }

        inner.c_lay.append(&spacer(16));

        // Group 3: Custom servers.
        add_group_header(&inner, "grp_custom", "Custom servers");
        inner.c_lay.append(&inner.custom_container);

        // Action buttons.
        let btn_add_custom = inner.btn_add_custom.clone();
        btn_add_custom.set_label("+   Add custom DNS server");
        btn_add_custom.add_css_class("dns-green-action");
        btn_add_custom.set_halign(gtk4::Align::Start);
        let i2 = inner.clone();
        btn_add_custom.connect_clicked(move |_| open_add_custom_dialog(&i2));
        inner.c_lay.append(&btn_add_custom);

        let btn_select_list = inner.btn_select_list.clone();
        btn_select_list.set_label("+   Select DNS server from the list");
        btn_select_list.add_css_class("dns-green-action");
        btn_select_list.set_halign(gtk4::Align::Start);
        btn_select_list.connect_clicked(move |_| {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard-dns.io/kb/general/dns-providers/")
                .spawn();
        });
        inner.c_lay.append(&btn_select_list);

        load_current_upstream(&inner);

        let i3 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i3)));

        retranslate_ui(&inner);

        DnsServerSelectView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_server_changed(&self, cb: Box<dyn Fn(String, String)>) {
        *self.inner.server_changed_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_drilldown_requested(&self, cb: Box<dyn Fn(&DnsServerData)>) {
        *self.inner.drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn load_current_upstream(&self) {
        load_current_upstream(&self.inner);
    }

    pub fn current_upstream(&self) -> String {
        self.inner.current_upstream.borrow().clone()
    }

    pub fn update_radio_selection(&self, upstream: &str) {
        update_radio_selection(&self.inner, upstream);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("dns_sel_title"));
    inner.lbl_desc.set_text(&i18n::tr("dns_sel_desc"));

    let headers: Vec<(String, gtk4::Label)> = inner
        .group_headers
        .borrow()
        .iter()
        .map(|(k, l)| (k.clone(), l.clone()))
        .collect();
    for (gid, lbl) in headers {
        let key = match gid.as_str() {
            "grp_adguard" => "dns_sel_sec_adguard",
            "grp_public" => "dns_sel_sec_public",
            "grp_custom" => "dns_sel_sec_custom",
            _ => continue,
        };
        lbl.set_text(&i18n::tr(key));
    }

    inner.btn_add_custom.set_label(&i18n::tr("dns_sel_btn_add"));
    inner.btn_select_list.set_label(&i18n::tr("dns_sel_btn_list"));
}

fn add_group_header(inner: &Rc<Inner>, group_id: &str, text: &str) {
    let lbl = gtk4::Label::new(Some(text));
    lbl.add_css_class("lbl-14-bold-white");
    lbl.set_halign(gtk4::Align::Start);
    lbl.set_margin_top(6);
    lbl.set_margin_bottom(6);
    inner.c_lay.append(&lbl);
    inner.group_headers.borrow_mut().insert(group_id.to_string(), lbl);
}

fn add_server_row(inner: &Rc<Inner>, server_id: &str, has_drilldown: bool) {
    let all = dns_servers_data();
    let data = match all.iter().find(|d| d.id == server_id) {
        Some(d) => d,
        None => return,
    };
    let row = DnsServerRow::new(server_id, data.name, data.desc, data.upstream, has_drilldown);

    let i = inner.clone();
    row.connect_selected(Box::new(move |sid| on_server_selected(&i, sid)));

    if has_drilldown {
        let i = inner.clone();
        row.connect_drilldown(Box::new(move |sid| on_drilldown_clicked(&i, sid)));
    }

    inner.c_lay.append(row.root());
    inner.rows.borrow_mut().insert(server_id.to_string(), row);
}

fn on_drilldown_clicked(inner: &Rc<Inner>, server_id: &str) {
    let all = dns_servers_data();
    let data = match all.iter().find(|d| d.id == server_id) {
        Some(d) => d,
        None => return,
    };
    if let Some(cb) = inner.drilldown_cb.borrow().as_ref() {
        cb(data);
    }
}

fn load_current_upstream(inner: &Rc<Inner>) {
    let path = proxy_yaml();
    let mut upstream = "default".to_string();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let re_up = Regex::new(r"^\s+upstream:\s*(.*)").unwrap();
            let mut in_dns = false;
            for line in content.lines() {
                let clean = line.split('#').next().unwrap_or("").trim_end();
                if clean.starts_with("dns_filtering:") {
                    in_dns = true;
                    continue;
                }
                if in_dns {
                    if !clean.is_empty() && !clean.starts_with(' ') && !clean.starts_with('\t') {
                        in_dns = false;
                        continue;
                    }
                    if let Some(caps) = re_up.captures(clean) {
                        upstream = caps[1].trim().trim_matches('\'').trim_matches('"').to_string();
                        break;
                    }
                }
            }
        }
    }

    *inner.current_upstream.borrow_mut() = upstream.clone();
    update_radio_selection(inner, &upstream);
}

fn update_radio_selection(inner: &Rc<Inner>, upstream: &str) {
    let mut matched = false;
    let rows: Vec<(String, DnsServerRow)> = inner
        .rows
        .borrow()
        .iter()
        .map(|(k, r)| (k.clone(), r.clone()))
        .collect();
    for (sid, row) in &rows {
        let data = dns_servers_data().into_iter().find(|d| d.id == *sid);
        let proto_urls: Vec<&str> = match data {
            Some(d) => d.protocols.iter().map(|p| p.value).collect(),
            None => Vec::new(),
        };
        let is_sel = row.upstream() == upstream || proto_urls.iter().any(|u| *u == upstream);
        row.set_checked(is_sel);
        if is_sel {
            matched = true;
        }
    }

    if !matched && upstream != "default" {
        let custom_id = format!("custom_{upstream}");
        let existing = inner.rows.borrow().get(&custom_id).cloned();
        let row = match existing {
            Some(r) => r,
            None => {
                let r = DnsServerRow::new(&custom_id, "Custom DNS", upstream, upstream, false);
                let i = inner.clone();
                r.connect_selected(Box::new(move |sid| on_server_selected(&i, sid)));
                inner.custom_container.append(r.root());
                inner.rows.borrow_mut().insert(custom_id.clone(), r.clone());
                r
            }
        };
        row.set_checked(true);
    }
}

fn on_server_selected(inner: &Rc<Inner>, server_id: &str) {
    let rows: Vec<(String, DnsServerRow)> = inner
        .rows
        .borrow()
        .iter()
        .map(|(k, r)| (k.clone(), r.clone()))
        .collect();
    let row = match rows.iter().find(|(k, _)| k == server_id) {
        Some((_, r)) => r.clone(),
        None => return,
    };

    for (k, r) in &rows {
        r.set_checked(k == server_id);
    }

    save_upstream(inner, row.lbl_title().text().to_string(), row.upstream().to_string());
}

fn open_add_custom_dialog(inner: &Rc<Inner>) {
    let dlg = AddCustomDnsDialog::new();
    let name_entry = dlg.input_name.clone();
    let url_entry = dlg.input_url.clone();
    let i = inner.clone();
    dlg.connect_accepted(Box::new(move || {
        let name = name_entry.text().trim().to_string();
        let name = if name.is_empty() { "My DNS server".to_string() } else { name };
        let addr = url_entry.text().trim().to_string();
        if addr.is_empty() {
            return;
        }
        let custom_id = format!("custom_{addr}");
        let existing = i.rows.borrow().get(&custom_id).cloned();
        if let Some(r) = existing {
            r.lbl_title().set_text(&name);
            r.lbl_desc().set_text(&addr);
        } else {
            let r = DnsServerRow::new(&custom_id, &name, &addr, &addr, false);
            let i2 = i.clone();
            r.connect_selected(Box::new(move |sid| on_server_selected(&i2, sid)));
            i.custom_container.append(r.root());
            i.rows.borrow_mut().insert(custom_id.clone(), r.clone());
        }
        on_server_selected(&i, &custom_id);
    }));
    dlg.root.present();
}

fn save_upstream(inner: &Rc<Inner>, display_name: String, upstream: String) {
    *inner.current_upstream.borrow_mut() = upstream.clone();

    for p in [proxy_yaml(), root_proxy_yaml()] {
        if !p.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&p) {
            let re_up = Regex::new(r"^\s*upstream:\s*").unwrap();
            let mut out: Vec<String> = Vec::new();
            let mut in_dns = false;
            for line in content.lines() {
                if line.starts_with("dns_filtering:") {
                    in_dns = true;
                    out.push(line.to_string());
                    continue;
                }
                if in_dns {
                    if !line.trim().is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                        in_dns = false;
                    } else if re_up.is_match(line) {
                        out.push(format!("  upstream: {upstream}"));
                        continue;
                    }
                }
                out.push(line.to_string());
            }
            let _ = std::fs::write(&p, out.join("\n") + "\n");
        }
    }

    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    backend::run_cmd_async(&format!("adguard-cli config set dns_filtering.upstream '{upstream}'"), noop);

    if let Some(cb) = inner.server_changed_cb.borrow().as_ref() {
        cb(display_name, upstream);
    }
}
