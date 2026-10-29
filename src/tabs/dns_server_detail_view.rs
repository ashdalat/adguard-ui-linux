// Port of adguard_modern/tabs/dns_server_detail_view.py
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use regex::Regex;

use crate::backend;
use crate::i18n;
use crate::tabs::dns_data::DnsServerData;
use crate::widgets::common_controls::{no_wheel_combo, BackButton};
use crate::widgets::dns_controls::dns_feature_row;

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

fn string_list(items: &[&str]) -> gtk4::StringList {
    let sl = gtk4::StringList::new(&[]);
    for it in items {
        sl.append(it);
    }
    sl
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    btn_website: gtk4::Button,
    c_lay: gtk4::Box,
    lbl_proto: gtk4::Label,
    combo_protocol: gtk4::DropDown,
    lbl_feat_head: gtk4::Label,
    features_container: gtk4::Box,
    btn_select: gtk4::Button,
    server_data: RefCell<Option<DnsServerData>>,
    protocol_urls: RefCell<Vec<String>>,
    current_selected_upstream: RefCell<String>,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    server_selected_cb: RefCell<Option<Box<dyn Fn(String, String)>>>,
}

#[derive(Clone)]
pub struct DnsServerDetailView {
    inner: Rc<Inner>,
}

impl DnsServerDetailView {
    pub fn new() -> DnsServerDetailView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("dns-protection-view");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // 1. Back button.
        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);

        // 2. Title.
        let lbl_title = gtk4::Label::new(Some("DNS Server"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_wrap(true);
        lbl_title.set_xalign(0.0);
        root.append(&lbl_title);

        // 3. Subtitle.
        let lbl_desc = gtk4::Label::new(Some("DNS service description"));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        // 4. Official website link.
        let btn_website = gtk4::Button::new();
        btn_website.set_label("Official website  ↗");
        btn_website.add_css_class("green-link");
        btn_website.set_halign(gtk4::Align::Start);
        root.append(&btn_website);

        root.append(&spacer(6));

        // 5. Scroll area for protocol + features.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        container.add_css_class("transparent-container");
        container.set_margin_end(8);
        container.set_margin_bottom(16);
        scroll.set_child(Some(&container));
        root.append(&scroll);

        // Protocol selection section.
        let protocol_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        let lbl_proto = gtk4::Label::new(Some("Protocol"));
        lbl_proto.add_css_class("dns-proto-label");
        lbl_proto.set_halign(gtk4::Align::Start);
        protocol_box.append(&lbl_proto);

        let combo_protocol = no_wheel_combo();
        combo_protocol.add_css_class("dns-proto-combo");
        protocol_box.append(&combo_protocol);
        container.append(&protocol_box);

        container.append(&spacer(6));

        // Features section header.
        let lbl_feat_head = gtk4::Label::new(Some("Features"));
        lbl_feat_head.add_css_class("lbl-14-bold-white");
        lbl_feat_head.set_halign(gtk4::Align::Start);
        container.append(&lbl_feat_head);

        // Features container.
        let features_container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        features_container.add_css_class("transparent-container");
        container.append(&features_container);

        // 6. Bottom "Select this server" button.
        let btn_select = gtk4::Button::new();
        btn_select.set_label("Select this server");
        btn_select.set_size_request(190, 40);
        btn_select.add_css_class("dns-select-btn");
        btn_select.set_halign(gtk4::Align::Start);
        root.append(&btn_select);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_desc,
            btn_website,
            c_lay: container,
            lbl_proto,
            combo_protocol,
            lbl_feat_head,
            features_container,
            btn_select,
            server_data: RefCell::new(None),
            protocol_urls: RefCell::new(Vec::new()),
            current_selected_upstream: RefCell::new("default".to_string()),
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            server_selected_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.root.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        inner.btn_website.connect_clicked(move |_| open_website(&i2));

        let i3 = inner.clone();
        inner.combo_protocol.connect_selected_notify(move |_c| {
            if *i3.syncing.borrow() {
                return;
            }
            update_select_button_state(&i3);
        });

        let i4 = inner.clone();
        inner.btn_select.connect_clicked(move |_| on_select_clicked(&i4));

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        retranslate_ui(&inner);

        DnsServerDetailView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_server_selected(&self, cb: Box<dyn Fn(String, String)>) {
        *self.inner.server_selected_cb.borrow_mut() = Some(cb);
    }

    pub fn load_server(&self, data: &DnsServerData, current_upstream: &str) {
        *self.inner.server_data.borrow_mut() = Some(data.clone());
        *self.inner.current_selected_upstream.borrow_mut() = current_upstream.to_string();

        self.inner.lbl_title.set_text(data.name);
        self.inner.lbl_desc.set_text(data.desc);

        // Deviation: ProtocolCheckmarkDelegate not available; plain StringList
        // combo with the protocol URLs kept alongside for currentData().
        *self.inner.syncing.borrow_mut() = true;
        let protocols: Vec<(&str, &str)> = if data.protocols.is_empty() {
            vec![("Regular", if data.upstream.is_empty() { "default" } else { data.upstream })]
        } else {
            data.protocols.iter().map(|p| (p.name, p.value)).collect()
        };
        let names: Vec<&str> = protocols.iter().map(|(n, _)| *n).collect();
        self.inner.combo_protocol.set_model(Some(&string_list(&names)));
        let urls: Vec<String> = protocols.iter().map(|(_, u)| u.to_string()).collect();
        *self.inner.protocol_urls.borrow_mut() = urls;
        let mut selected_idx = 0usize;
        for (idx, (_, u)) in protocols.iter().enumerate() {
            if *u == current_upstream {
                selected_idx = idx;
            }
        }
        self.inner.combo_protocol.set_selected(selected_idx as u32);
        *self.inner.syncing.borrow_mut() = false;

        while let Some(child) = self.inner.features_container.first_child() {
            self.inner.features_container.remove(&child);
        }
        for f in &data.features {
            let row = dns_feature_row(f.key, f.title, f.desc);
            self.inner.features_container.append(&row);
        }

        update_select_button_state(&self.inner);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.btn_website.set_label(&i18n::tr("dns_dt_website"));
    inner.lbl_proto.set_text(&i18n::tr("dns_dt_protocol"));
    inner.lbl_feat_head.set_text(&i18n::tr("dns_dt_features"));
    update_select_button_state(inner);
}

fn current_data(inner: &Rc<Inner>) -> String {
    let idx = inner.combo_protocol.selected() as usize;
    inner.protocol_urls.borrow().get(idx).cloned().unwrap_or_else(|| "default".to_string())
}

fn update_select_button_state(inner: &Rc<Inner>) {
    let active_url = current_data(inner);
    let current = inner.current_selected_upstream.borrow().clone();
    if active_url == current {
        inner.btn_select.set_label(&i18n::tr("dns_dt_btn_selected"));
        inner.btn_select.set_sensitive(false);
    } else {
        inner.btn_select.set_label(&i18n::tr("dns_dt_btn_select"));
        inner.btn_select.set_sensitive(true);
    }
}

fn open_website(inner: &Rc<Inner>) {
    let url = match inner.server_data.borrow().as_ref() {
        Some(d) if !d.website.is_empty() => d.website.to_string(),
        _ => "https://adguard-dns.io/".to_string(),
    };
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
}

fn on_select_clicked(inner: &Rc<Inner>) {
    let chosen_upstream = current_data(inner);
    let name = match inner.server_data.borrow().as_ref() {
        Some(d) => d.name.to_string(),
        None => "DNS Server".to_string(),
    };
    *inner.current_selected_upstream.borrow_mut() = chosen_upstream.clone();
    update_select_button_state(inner);

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
                        out.push(format!("  upstream: {chosen_upstream}"));
                        continue;
                    }
                }
                out.push(line.to_string());
            }
            let _ = std::fs::write(&p, out.join("\n") + "\n");
        }
    }

    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    backend::run_cmd_async(&format!("adguard-cli config set dns_filtering.upstream '{chosen_upstream}'"), noop);

    if let Some(cb) = inner.server_selected_cb.borrow().as_ref() {
        cb(name, chosen_upstream);
    }
}
