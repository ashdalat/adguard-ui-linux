// Port of adguard_modern/widgets/filtering_log_inspector.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::i18n;

#[derive(Clone)]
pub struct LogEntry {
    pub name: String,
    pub status: String,
    pub rule: String,
    pub filter: String,
    pub r#type: String,
    pub source: String,
    pub proc: String,
}

struct Inner {
    root: gtk4::Box,
    stack: gtk4::Stack,
    btn_general: gtk4::Button,
    btn_network: gtk4::Button,
    btn_preview: gtk4::Button,
    btn_block: gtk4::Button,
    lbl_url_title: gtk4::Label,
    lbl_url: gtk4::Label,
    lbl_status_title: gtk4::Label,
    lbl_status: gtk4::Label,
    lbl_source_title: gtk4::Label,
    lbl_source: gtk4::Label,
    lbl_proc_title: gtk4::Label,
    lbl_proc: gtk4::Label,
    lbl_tls_title: gtk4::Label,
    lbl_tls: gtk4::Label,
    lbl_cert_link: gtk4::Label,
    lbl_rule_title: gtk4::Label,
    lbl_rule: gtk4::Label,
    lbl_stealth_title: gtk4::Label,
    lbl_stealth: gtk4::Label,
    lbl_type_title: gtk4::Label,
    lbl_type: gtk4::Label,
    lbl_time_title: gtk4::Label,
    lbl_duration: gtk4::Label,
    lbl_size_title: gtk4::Label,
    lbl_size: gtk4::Label,
    lbl_net_title: gtk4::Label,
    lbl_network_info: gtk4::Label,
    lbl_prev_title: gtk4::Label,
    lbl_preview_box: gtk4::Label,
    current_entry: RefCell<Option<LogEntry>>,
    closed_cb: RefCell<Option<Box<dyn Fn()>>>,
    block_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct FilteringLogInspector {
    inner: Rc<Inner>,
}

impl FilteringLogInspector {
    pub fn new() -> FilteringLogInspector {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_size_request(340, -1);
        root.add_css_class("inspector");

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        header.add_css_class("inspector-header");
        header.set_size_request(-1, 38);
        header.set_margin_start(12);
        header.set_margin_end(10);

        let btn_general = tab_button("General", true);
        let btn_network = tab_button("Network", false);
        let btn_preview = tab_button("Preview", false);
        header.append(&btn_general);
        header.append(&btn_network);
        header.append(&btn_preview);
        let hs = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        hs.set_hexpand(true);
        header.append(&hs);
        let btn_close = gtk4::Button::new();
        btn_close.add_css_class("dlg-close");
        btn_close.set_size_request(22, 22);
        btn_close.set_label("✕");
        header.append(&btn_close);
        root.append(&header);

        let stack = gtk4::Stack::new();
        stack.set_vhomogeneous(false);
        root.append(&stack);

        // Page 0: General
        let general = scroll_page();
        let gv = page_content(&general);
        let lbl_url_title = bold_title("URL yêu cầu");
        let lbl_url = value_label("-", true);
        let lbl_status_title = bold_title("Trạng thái");
        let lbl_status = status_label("Được phép", true);
        let act_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        let btn_block = gtk4::Button::new();
        btn_block.add_css_class("link-red");
        btn_block.set_label("Chặn yêu cầu...");
        act_box.append(&btn_block);
        let lbl_source_title = bold_title("URL gốc");
        let lbl_source = value_label("-", true);
        let lbl_proc_title = bold_title("Tiến trình");
        let lbl_proc = value_label("brave (10376)", false);
        let lbl_tls_title = bold_title("TLS");
        let lbl_tls = small_muted(
            "Cơ quan cấp chứng chỉ: Google Trust Services (được xác minh bởi AdGuard)\n\
             Phiên bản TLS: TLS 1.3\n\
             ClientHello: Không được mã hóa\n\
             Bộ mật mã: TLS_AES_128_GCM_SHA256",
        );
        let lbl_cert_link = gtk4::Label::new(Some("<a href='#cert' style='color: #5EBB6A; text-decoration: none;'>Xem chứng chỉ trang web</a>"));
        lbl_cert_link.add_css_class("lbl-11");
        lbl_cert_link.set_halign(gtk4::Align::Start);
        lbl_cert_link.set_use_markup(true);
        let lbl_rule_title = bold_title("Quy tắc");
        let lbl_rule = value_label("Stealth mode", false);
        let lbl_stealth_title = bold_title("Chế độ ẩn danh");
        let lbl_stealth = small_muted(
            "Referrer has been hidden\nThe IP has been hidden\nThe User Agent has been hidden\n\
             DNT Header has been sent\nCache has been disabled",
        );
        let lbl_type_title = bold_title("Loại nội dung");
        let lbl_type = value_label("Image", false);
        let meta = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let vbox_time = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let lbl_time_title = bold_title("Thời gian");
        let lbl_duration = value_label("756 ms", false);
        vbox_time.append(&lbl_time_title);
        vbox_time.append(&lbl_duration);
        meta.append(&vbox_time);
        let vbox_size = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let lbl_size_title = bold_title("Kích cỡ");
        let lbl_size = value_label("10 B +651 B", false);
        vbox_size.append(&lbl_size_title);
        vbox_size.append(&lbl_size);
        meta.append(&vbox_size);

        gv.append(&lbl_url_title);
        gv.append(&lbl_url);
        gv.append(&lbl_status_title);
        gv.append(&lbl_status);
        gv.append(&act_box);
        gv.append(&lbl_source_title);
        gv.append(&lbl_source);
        gv.append(&lbl_proc_title);
        gv.append(&lbl_proc);
        gv.append(&lbl_tls_title);
        gv.append(&lbl_tls);
        gv.append(&lbl_cert_link);
        gv.append(&lbl_rule_title);
        gv.append(&lbl_rule);
        gv.append(&lbl_stealth_title);
        gv.append(&lbl_stealth);
        gv.append(&lbl_type_title);
        gv.append(&lbl_type);
        gv.append(&meta);
        stack.add_named(&general, Some("general"));

        // Page 1: Network
        let network = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        network.add_css_class("inspector-page");
        network.set_margin_start(14);
        network.set_margin_end(14);
        network.set_margin_top(14);
        network.set_margin_bottom(14);
        let lbl_net_title = bold_title("Thông số mạng (Headers)");
        let lbl_network_info = small_muted(
            "Host: i.ytimg.com\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64)\n\
             Accept: image/avif,image/webp,image/apng,*/*\nSec-Fetch-Site: cross-site\n\
             Sec-Fetch-Mode: no-cors\nSec-Fetch-Dest: image\nConnection: keep-alive",
        );
        network.append(&lbl_net_title);
        network.append(&lbl_network_info);
        stack.add_named(&network, Some("network"));

        // Page 2: Preview
        let preview = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        preview.add_css_class("inspector-page");
        preview.set_margin_start(14);
        preview.set_margin_end(14);
        preview.set_margin_top(14);
        preview.set_margin_bottom(14);
        let lbl_prev_title = bold_title("Xem trước nội dung (Preview)");
        let lbl_preview_box = gtk4::Label::new(Some("Chưa có xem trước nội dung cho yêu cầu này"));
        lbl_preview_box.add_css_class("lbl-11");
        lbl_preview_box.set_halign(gtk4::Align::Center);
        lbl_preview_box.set_vexpand(true);
        preview.append(&lbl_prev_title);
        preview.append(&lbl_preview_box);
        stack.add_named(&preview, Some("preview"));

        let inner = Rc::new(Inner {
            root,
            stack,
            btn_general,
            btn_network,
            btn_preview,
            btn_block: btn_block.clone(),
            lbl_url_title,
            lbl_url,
            lbl_status_title,
            lbl_status,
            lbl_source_title,
            lbl_source,
            lbl_proc_title,
            lbl_proc,
            lbl_tls_title,
            lbl_tls,
            lbl_cert_link,
            lbl_rule_title,
            lbl_rule,
            lbl_stealth_title,
            lbl_stealth,
            lbl_type_title,
            lbl_type,
            lbl_time_title,
            lbl_duration,
            lbl_size_title,
            lbl_size,
            lbl_net_title,
            lbl_network_info,
            lbl_prev_title,
            lbl_preview_box,
            current_entry: RefCell::new(None),
            closed_cb: RefCell::new(None),
            block_cb: RefCell::new(None),
        });
        let w = FilteringLogInspector { inner: inner.clone() };

        for (btn, page) in [
            (&inner.btn_general, "general"),
            (&inner.btn_network, "network"),
            (&inner.btn_preview, "preview"),
        ] {
            let i2 = inner.clone();
            let p = page.to_string();
            btn.connect_clicked(move |_| {
                i2.stack.set_visible_child_name(&p);
                set_tab_active(&i2, &p);
            });
        }

        let i3 = inner.clone();
        btn_close.connect_clicked(move |_| {
            if let Some(cb) = i3.closed_cb.borrow().as_ref() {
                cb();
            }
        });

        let i4 = inner.clone();
        btn_block.connect_clicked(move |_| {
            let entry = i4.current_entry.borrow().clone();
            if let Some(e) = entry {
                if let Some(cb) = i4.block_cb.borrow().as_ref() {
                    cb(&e.name);
                }
            }
        });

        i18n::connect_language_changed(Box::new({
            let w2 = w.clone();
            move |_| w2.retranslate_ui()
        }));
        w.retranslate_ui();

        w
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_closed(&self, cb: Box<dyn Fn()>) {
        *self.inner.closed_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_block_rule_requested(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.block_cb.borrow_mut() = Some(cb);
    }

    pub fn retranslate_ui(&self) {
        let i = &self.inner;
        i.btn_general.set_label(&i18n::tr("flog_general"));
        i.btn_network.set_label(&i18n::tr("flog_network"));
        i.btn_preview.set_label(&i18n::tr("flog_preview"));
        i.lbl_url_title.set_text(&i18n::tr("flog_req_url"));
        i.lbl_status_title.set_text(&i18n::tr("flog_req_status"));
        i.lbl_source_title.set_text(&i18n::tr("flog_orig_url"));
        i.lbl_proc_title.set_text(&i18n::tr("flog_col_process"));
        i.lbl_tls_title.set_text(&i18n::tr("flog_tls_title"));
        i.lbl_tls.set_text(&format!(
            "{}\n{}\n{}\n{}",
            i18n::tr("flog_tls_ca"),
            i18n::tr("flog_tls_ver"),
            i18n::tr("flog_tls_client_hello"),
            i18n::tr("flog_tls_cipher")
        ));
        i.lbl_cert_link.set_text(&format!(
            "<a href='#cert' style='color: #5EBB6A; text-decoration: none;'>{}</a>",
            i18n::tr("flog_view_cert")
        ));
        i.lbl_rule_title.set_text(&i18n::tr("flog_col_rule"));
        i.lbl_stealth_title.set_text(&i18n::tr("flog_stealth_title"));
        i.lbl_type_title.set_text(&i18n::tr("flog_content_type"));
        i.lbl_time_title.set_text(&i18n::tr("flog_time"));
        i.lbl_size_title.set_text(&i18n::tr("flog_size"));
        i.lbl_net_title.set_text(&i18n::tr("flog_network_headers"));
        i.lbl_prev_title.set_text(&i18n::tr("flog_preview"));
        if self.inner.current_entry.borrow().is_none() {
            i.lbl_preview_box.set_text(&i18n::tr("flog_no_preview"));
        }
        let entry = self.inner.current_entry.borrow().clone();
        if let Some(e) = entry {
            self.set_entry(e);
        }
    }

    pub fn set_entry(&self, entry: LogEntry) {
        let i = &self.inner;
        let is_en = i18n::get_current_language() == i18n::LANG_EN;
        *i.current_entry.borrow_mut() = Some(entry.clone());

        let url = entry.name.clone();
        let raw_status = entry.status.clone();
        let is_blocked = raw_status == "Đã chặn" || raw_status == "Blocked";
        let rule = entry.rule.clone();
        let req_type = entry.r#type.clone();
        let source = entry.source.clone();
        let proc = entry.proc.clone();

        i.lbl_url.set_text(&url);
        if is_blocked {
            i.lbl_status.set_text(&i18n::tr("flog_status_blocked"));
            i.lbl_status.add_css_class("status-red");
            i.lbl_status.remove_css_class("status-green");
            i.btn_block.set_label(&i18n::tr("flog_unblock_req"));
            i.btn_block.add_css_class("link-green");
            i.btn_block.remove_css_class("link-red");
        } else {
            i.lbl_status.set_text(&i18n::tr("flog_status_allowed"));
            i.lbl_status.add_css_class("status-green");
            i.lbl_status.remove_css_class("status-red");
            i.btn_block.set_label(&i18n::tr("flog_block_req"));
            i.btn_block.add_css_class("link-red");
            i.btn_block.remove_css_class("link-green");
        }

        i.lbl_source.set_text(&source);
        i.lbl_proc.set_text(&format!("{} (10376)", proc));
        i.lbl_rule.set_text(&rule);
        i.lbl_type.set_text(&req_type);

        let is_stealth = rule == "Stealth mode";
        i.lbl_stealth_title.set_visible(is_stealth);
        i.lbl_stealth.set_visible(is_stealth);

        let domain = if url.contains("://") {
            url.split("://").last().unwrap_or(&url).split('/').next().unwrap_or(&url).to_string()
        } else {
            url.clone()
        };
        i.lbl_network_info.set_text(&format!(
            "Host: {}\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64; AdGuard/CLI)\n\
             Accept: */*\nSec-Fetch-Site: cross-site\nConnection: keep-alive",
            domain
        ));
        let req_lbl = if is_en { "Request type" } else { "Loại yêu cầu" };
        let dest_lbl = if is_en { "Target" } else { "Đích" };
        i.lbl_preview_box.set_text(&format!("{}: {}\n{}: {}", req_lbl, req_type, dest_lbl, domain));
    }
}

fn set_tab_active(i: &Inner, page: &str) {
    for (btn, p) in [(&i.btn_general, "general"), (&i.btn_network, "network"), (&i.btn_preview, "preview")] {
        if p == page {
            btn.add_css_class("tab-active");
        } else {
            btn.remove_css_class("tab-active");
        }
    }
}

fn tab_button(label: &str, active: bool) -> gtk4::Button {
    let b = gtk4::Button::new();
    b.add_css_class("tab-btn");
    if active {
        b.add_css_class("tab-active");
    }
    b.set_label(label);
    b
}

fn scroll_page() -> gtk4::ScrolledWindow {
    let s = gtk4::ScrolledWindow::new();
    s.add_css_class("inspector-page");
    s
}

fn page_content(scroll: &gtk4::ScrolledWindow) -> gtk4::Box {
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    content.set_margin_start(14);
    content.set_margin_end(14);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    scroll.set_child(Some(&content));
    content
}

fn bold_title(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("inspector-title");
    l.set_halign(gtk4::Align::Start);
    l
}

fn value_label(text: &str, wrap: bool) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("inspector-value");
    l.set_wrap(wrap);
    l.set_halign(gtk4::Align::Start);
    l.set_selectable(wrap);
    l
}

fn status_label(text: &str, green: bool) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("inspector-status");
    if green {
        l.add_css_class("status-green");
    }
    l.set_halign(gtk4::Align::Start);
    l
}

fn small_muted(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("lbl-11");
    l.set_wrap(true);
    l.set_halign(gtk4::Align::Start);
    l
}

