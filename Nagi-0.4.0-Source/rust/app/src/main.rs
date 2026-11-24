#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Icon, Window, WindowBuilder},
};
use url::Url;
#[cfg(target_os = "macos")]
use wry::WebViewExtMacOS;
#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewExtWindows};
use wry::{
    NewWindowFeatures, NewWindowResponse, PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder,
    dpi::{LogicalPosition as WebPosition, LogicalSize as WebSize},
    http::Request,
};
#[cfg(target_os = "linux")]
use wry::{WebViewBuilderExtUnix, WebViewExtUnix};

const TOOLBAR_HEIGHT: f64 = 108.0;
const TOOLBAR_OVERLAY_HEIGHT: f64 = 560.0;
const NEW_TAB_URL: &str = "nagi://newtab";
const SEARCH_HOME_URL: &str = "https://www.google.com/";
#[cfg(any(target_os = "windows", test))]
const AD_HOSTS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "adservice.google.com",
    "adservice.google.fr",
    "adnxs.com",
    "adsrvr.org",
    "adform.net",
    "taboola.com",
    "outbrain.com",
    "imasdk.googleapis.com",
    "connect.facebook.net",
    "pixel.facebook.com",
    "scorecardresearch.com",
];

#[derive(Debug)]
enum AppEvent {
    ToolbarMessage(String),
    HomeMessage { tab_id: u64, message: String },
    PageChanged { tab_id: u64, url: String },
    PageLoading { tab_id: u64, loading: bool },
    DownloadChanged(DownloadItem),
    NewWindow(String),
    ToolbarReady,
}

#[derive(Debug, Deserialize)]
struct ToolbarCommand {
    action: String,
    url: Option<String>,
    tab_id: Option<u64>,
    index: Option<usize>,
    open: Option<bool>,
    preference: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    dark_mode: bool,
    ad_blocking: bool,
    search_engine: String,
    restore_last_page: bool,
    language: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark_mode: false,
            ad_blocking: true,
            search_engine: "google".into(),
            restore_last_page: true,
            language: "en".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bookmark {
    title: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryItem {
    title: String,
    url: String,
    visited_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadItem {
    name: String,
    url: String,
    path: String,
    complete: bool,
    success: bool,
}

#[derive(Debug, Serialize)]
struct ToolbarTab {
    id: u64,
    label: String,
    active: bool,
}

struct Tab {
    id: u64,
    url: String,
    label: String,
    view: WebView,
    home_mode: Arc<AtomicBool>,
    loading: bool,
}

struct Browser {
    window: Window,
    #[cfg(target_os = "linux")]
    container: gtk::Fixed,
    toolbar: WebView,
    tabs: Vec<Tab>,
    active_tab: u64,
    next_tab_id: u64,
    context: WebContext,
    proxy: EventLoopProxy<AppEvent>,
    bookmarks: Vec<Bookmark>,
    bookmarks_file: PathBuf,
    history: Vec<HistoryItem>,
    history_file: PathBuf,
    downloads: Vec<DownloadItem>,
    downloads_file: PathBuf,
    downloads_directory: PathBuf,
    popover_open: bool,
    preferences: Preferences,
    preferences_file: PathBuf,
    ad_blocking_enabled: Arc<AtomicBool>,
    last_page_file: PathBuf,
    recently_closed: Vec<String>,
}

impl Browser {
    fn new(
        event_loop: &tao::event_loop::EventLoop<AppEvent>,
        initial_inputs: Vec<String>,
        initial_language: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let window = WindowBuilder::new()
            .with_title("Nagi")
            .with_inner_size(LogicalSize::new(1280.0, 820.0))
            .with_window_icon(Some(nagi_icon()?))
            .build(event_loop)?;
        #[cfg(target_os = "linux")]
        let container = {
            use gtk::prelude::*;
            use tao::platform::unix::WindowExtUnix;

            let container = gtk::Fixed::new();
            let Some(root) = window.default_vbox() else {
                return Err("Tao did not create its GTK root container".into());
            };
            root.pack_start(&container, true, true, 0);
            container.show();
            container
        };
        let proxy = event_loop.create_proxy();
        let profile = profile_directory()?;
        let bookmarks_file = profile.parent().unwrap_or(&profile).join("bookmarks.json");
        let history_file = profile.parent().unwrap_or(&profile).join("history.json");
        let downloads_file = profile.parent().unwrap_or(&profile).join("downloads.json");
        let preferences_file = profile.parent().unwrap_or(&profile).join("settings.json");
        let last_page_file = profile.parent().unwrap_or(&profile).join("last-page.json");
        let downloads_directory = downloads_directory();
        fs::create_dir_all(&downloads_directory)?;
        let bookmarks = load_bookmarks(&bookmarks_file);
        let history = load_json_or_default(&history_file);
        let downloads = load_json_or_default(&downloads_file);
        let mut preferences = load_preferences(&preferences_file);
        if let Some(language) = initial_language {
            preferences.language = normalize_language(&language).to_owned();
            save_json(&preferences_file, &preferences)?;
        }
        let ad_blocking_enabled = Arc::new(AtomicBool::new(preferences.ad_blocking));
        let mut context = WebContext::new(Some(profile));
        let toolbar_proxy = proxy.clone();
        let ready_proxy = proxy.clone();
        let toolbar_builder = WebViewBuilder::new_with_web_context(&mut context)
            .with_bounds(toolbar_bounds(&window, false))
            .with_transparent(true)
            .with_initialization_script(include_str!("../assets/i18n.js"))
            .with_html(include_str!("../assets/toolbar.html"))
            .with_ipc_handler(move |request: Request<String>| {
                let _ = toolbar_proxy.send_event(AppEvent::ToolbarMessage(request.body().clone()));
            })
            .with_on_page_load_handler(move |_, _| {
                let _ = ready_proxy.send_event(AppEvent::ToolbarReady);
            });
        #[cfg(target_os = "linux")]
        let toolbar = toolbar_builder.build_gtk(&container)?;
        #[cfg(not(target_os = "linux"))]
        let toolbar = toolbar_builder.build_as_child(&window)?;

        let mut browser = Self {
            window,
            #[cfg(target_os = "linux")]
            container,
            toolbar,
            tabs: Vec::new(),
            active_tab: 0,
            next_tab_id: 1,
            context,
            proxy,
            bookmarks,
            bookmarks_file,
            history,
            history_file,
            downloads,
            downloads_file,
            downloads_directory,
            popover_open: false,
            preferences,
            preferences_file,
            ad_blocking_enabled,
            last_page_file,
            recently_closed: Vec::new(),
        };
        if initial_inputs.is_empty() {
            let last_page = if browser.preferences.restore_last_page {
                load_last_page(&browser.last_page_file)
            } else {
                None
            };
            browser.create_tab(last_page.unwrap_or_else(|| NEW_TAB_URL.to_owned()))?;
        } else {
            for input in initial_inputs {
                browser.create_tab(resolve_input(&input, &browser.preferences.search_engine))?;
            }
        }
        Ok(browser)
    }

    fn create_tab(&mut self, url: String) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let page_proxy = self.proxy.clone();
        let popup_proxy = self.proxy.clone();
        let home_proxy = self.proxy.clone();
        let home_mode = Arc::new(AtomicBool::new(url == NEW_TAB_URL));
        let page_home_mode = home_mode.clone();
        let navigation_home_mode = home_mode.clone();
        let download_proxy = self.proxy.clone();
        let finished_download_proxy = download_proxy.clone();
        let download_dir = self.downloads_directory.clone();
        let ad_blocking_enabled = self.ad_blocking_enabled.clone();
        let page_url = url.clone();
        let mut builder = WebViewBuilder::new_with_web_context(&mut self.context)
            .with_bounds(view_bounds(&self.window))
            // Every tab can later become the built-in home page through `show_home`.
            // Install localization before the first document so that transition works too.
            .with_initialization_script(include_str!("../assets/i18n.js"))
            .with_ipc_handler(move |request: Request<String>| {
                if page_home_mode.load(Ordering::Relaxed) {
                    let _ = home_proxy.send_event(AppEvent::HomeMessage {
                        tab_id: id,
                        message: request.body().clone(),
                    });
                }
            })
            .with_on_page_load_handler(move |event: PageLoadEvent, loaded_url| match event {
                PageLoadEvent::Started => {
                    let _ = page_proxy.send_event(AppEvent::PageLoading {
                        tab_id: id,
                        loading: true,
                    });
                }
                PageLoadEvent::Finished => {
                    let _ = page_proxy.send_event(AppEvent::PageChanged {
                        tab_id: id,
                        url: loaded_url,
                    });
                }
            })
            .with_new_window_req_handler(move |requested_url, _: NewWindowFeatures| {
                let target = if requested_url.is_empty() {
                    page_url.clone()
                } else {
                    requested_url
                };
                let _ = popup_proxy.send_event(AppEvent::NewWindow(target));
                NewWindowResponse::Deny
            })
            .with_navigation_handler(move |target| {
                if target.starts_with("http://") || target.starts_with("https://") {
                    navigation_home_mode.store(false, Ordering::Relaxed);
                }
                true
            })
            .with_download_started_handler(move |url, path| {
                let filename = download_filename(&url);
                let destination = unique_download_path(&download_dir, &filename);
                *path = destination.clone();
                let item = DownloadItem {
                    name: filename,
                    url,
                    path: destination.to_string_lossy().into_owned(),
                    complete: false,
                    success: false,
                };
                let _ = download_proxy.send_event(AppEvent::DownloadChanged(item));
                true
            })
            .with_download_completed_handler(move |url, path, success| {
                let path = path.unwrap_or_default();
                let item = DownloadItem {
                    name: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Téléchargement".to_owned()),
                    url,
                    path: path.to_string_lossy().into_owned(),
                    complete: true,
                    success,
                };
                let _ = finished_download_proxy.send_event(AppEvent::DownloadChanged(item));
            });
        if url == NEW_TAB_URL {
            builder = builder.with_html(include_str!("../assets/newtab.html"));
        } else {
            builder = builder.with_url("about:blank");
        }
        #[cfg(target_os = "linux")]
        let view = builder.build_gtk(&self.container)?;
        #[cfg(not(target_os = "linux"))]
        let view = builder.build_as_child(&self.window)?;
        enable_password_autosave(&view);
        enable_ad_blocking(&view, ad_blocking_enabled);
        if url != NEW_TAB_URL {
            view.load_url(&url)?;
        }

        if let Some(previous) = self.active_view() {
            set_memory_usage_level(previous, false);
            let _ = previous.set_visible(false);
        }
        set_memory_usage_level(&view, true);

        self.active_tab = id;
        self.tabs.push(Tab {
            id,
            label: if url == NEW_TAB_URL {
                "Nouvel onglet".to_owned()
            } else {
                tab_label(&url)
            },
            url,
            view,
            home_mode,
            loading: false,
        });
        if self.popover_open {
            self.raise_toolbar();
        }
        self.sync_toolbar();
        Ok(())
    }

    fn active_view(&self) -> Option<&WebView> {
        self.tabs
            .iter()
            .find(|tab| tab.id == self.active_tab)
            .map(|tab| &tab.view)
    }

    fn raise_toolbar(&self) {
        #[cfg(target_os = "linux")]
        raise_toolbar(&self.toolbar, &self.container);
        #[cfg(not(target_os = "linux"))]
        raise_toolbar(&self.toolbar);
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::ToolbarMessage(message) => self.handle_toolbar_message(&message),
            AppEvent::HomeMessage { tab_id, message } => {
                let Ok(command) = serde_json::from_str::<ToolbarCommand>(&message) else {
                    return;
                };
                if command.action == "navigate"
                    && let Some(input) = command.url
                {
                    self.navigate_tab(tab_id, &input);
                }
            }
            AppEvent::PageChanged { tab_id, url } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if url.starts_with("http://") || url.starts_with("https://") {
                        tab.home_mode.store(false, Ordering::Relaxed);
                        tab.url = url.clone();
                        tab.label = tab_label(&url);
                    } else if url.starts_with("data:") {
                        tab.home_mode.store(true, Ordering::Relaxed);
                        tab.url = NEW_TAB_URL.to_owned();
                        tab.label = "Nouvel onglet".to_owned();
                    }
                }
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.loading = false;
                }
                if url.starts_with("http://") || url.starts_with("https://") {
                    self.history.retain(|item| item.url != url);
                    self.history.insert(
                        0,
                        HistoryItem {
                            title: tab_label(&url),
                            url: url.clone(),
                            visited_at: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        },
                    );
                    self.history.truncate(300);
                    if let Err(error) = save_json(&self.history_file, &self.history) {
                        eprintln!("Impossible d’enregistrer l’historique : {error}");
                    }
                }
                self.save_last_page();
                self.sync_toolbar();
            }
            AppEvent::PageLoading { tab_id, loading } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.loading = loading;
                }
                self.sync_toolbar();
            }
            AppEvent::DownloadChanged(item) => {
                if let Some(existing) = self
                    .downloads
                    .iter_mut()
                    .find(|entry| entry.path == item.path)
                {
                    *existing = item;
                } else {
                    self.downloads.insert(0, item);
                    self.downloads.truncate(100);
                }
                if let Err(error) = save_json(&self.downloads_file, &self.downloads) {
                    eprintln!("Impossible d’enregistrer la liste des téléchargements : {error}");
                }
                self.sync_toolbar();
            }
            AppEvent::NewWindow(url) => {
                let url = resolve_input(&url, &self.preferences.search_engine);
                if let Err(error) = self.create_tab(url) {
                    eprintln!("Impossible d’ouvrir un nouvel onglet : {error}");
                }
            }
            AppEvent::ToolbarReady => self.sync_toolbar(),
        }
    }

    fn handle_toolbar_message(&mut self, message: &str) {
        let Ok(command) = serde_json::from_str::<ToolbarCommand>(message) else {
            return;
        };

        match command.action.as_str() {
            "navigate" => {
                if let Some(input) = command.url {
                    self.navigate_tab(self.active_tab, &input);
                }
            }
            "back" => {
                if let Some(view) = self.active_view()
                    && view.can_go_back().unwrap_or(false)
                {
                    let _ = view.go_back();
                }
            }
            "forward" => {
                if let Some(view) = self.active_view()
                    && view.can_go_forward().unwrap_or(false)
                {
                    let _ = view.go_forward();
                }
            }
            "reload" => {
                if let Some(view) = self.active_view() {
                    let _ = view.reload();
                }
            }
            "new-tab" => {
                if let Err(error) = self.create_tab(NEW_TAB_URL.to_owned()) {
                    eprintln!("Impossible de créer un onglet : {error}");
                }
            }
            "home" => self.show_home(self.active_tab),
            "bookmark" => self.toggle_bookmark(),
            "open-bookmark" => {
                if let Some(index) = command.index
                    && let Some(bookmark) = self.bookmarks.get(index)
                {
                    let url = bookmark.url.clone();
                    self.navigate_tab(self.active_tab, &url);
                }
            }
            "open-history" => {
                if let Some(index) = command.index
                    && let Some(item) = self.history.get(index)
                {
                    let url = item.url.clone();
                    self.navigate_tab(self.active_tab, &url);
                }
            }
            "open-download" => {
                if let Some(index) = command.index
                    && let Some(item) = self.downloads.get(index)
                    && item.complete
                    && item.success
                {
                    reveal_download(Path::new(&item.path));
                }
            }
            "open-downloads-folder" => {
                open_in_file_manager(&self.downloads_directory);
            }
            "open-update" => {
                if let (Some(url), Some(repository)) = (
                    command.url.as_deref(),
                    option_env!("NAGI_GITHUB_REPOSITORY"),
                ) && is_update_download_url(url, repository, update_asset_name())
                {
                    open_external_url(url);
                }
            }
            "popover-state" => self.set_popover_open(command.open.unwrap_or(false)),
            "set-preference" => {
                if let (Some(key), Some(value)) = (command.preference, command.value) {
                    self.set_preference(&key, &value);
                }
            }
            "clear-history" => {
                self.history.clear();
                let _ = save_json(&self.history_file, &self.history);
                self.sync_toolbar();
            }
            "reopen-tab" => {
                if let Some(url) = self.recently_closed.pop()
                    && let Err(error) = self.create_tab(url)
                {
                    eprintln!("Impossible de rouvrir l’onglet : {error}");
                }
            }
            "activate-tab" => {
                if let Some(id) = command.tab_id {
                    self.activate_tab(id);
                }
            }
            "close-tab" => {
                self.close_tab(command.tab_id.unwrap_or(self.active_tab));
            }
            _ => {}
        }
    }

    fn activate_tab(&mut self, id: u64) {
        if !self.tabs.iter().any(|tab| tab.id == id) {
            return;
        }
        if let Some(view) = self.active_view() {
            let _ = view.set_visible(false);
        }
        self.active_tab = id;
        if let Some(view) = self.active_view() {
            set_memory_usage_level(view, true);
            let _ = view.set_visible(true);
            let _ = view.focus();
        }
        self.sync_toolbar();
    }

    fn close_tab(&mut self, id: u64) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return;
        };
        let closed_url = self.tabs[index].url.clone();
        if is_web_url(&closed_url) {
            self.recently_closed.push(closed_url);
            self.recently_closed.truncate(10);
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            if let Err(error) = self.create_tab(NEW_TAB_URL.to_owned()) {
                eprintln!("Impossible de remplacer le dernier onglet : {error}");
            }
            return;
        }
        if self.active_tab == id {
            let next = self.tabs[index.min(self.tabs.len() - 1)].id;
            self.active_tab = next;
            if let Some(view) = self.active_view() {
                set_memory_usage_level(view, true);
                let _ = view.set_visible(true);
            }
        }
        self.save_last_page();
        self.sync_toolbar();
    }

    fn resize(&self) {
        let _ = self
            .toolbar
            .set_bounds(toolbar_bounds(&self.window, self.popover_open));
        if self.popover_open {
            self.raise_toolbar();
        }
        let bounds = view_bounds(&self.window);
        for tab in &self.tabs {
            let _ = tab.view.set_bounds(bounds);
        }
    }

    fn set_popover_open(&mut self, open: bool) {
        if self.popover_open == open {
            return;
        }
        self.popover_open = open;
        let _ = self
            .toolbar
            .set_bounds(toolbar_bounds(&self.window, self.popover_open));
        if self.popover_open {
            self.raise_toolbar();
        }
    }

    fn sync_toolbar(&self) {
        let tabs: Vec<ToolbarTab> = self
            .tabs
            .iter()
            .map(|tab| ToolbarTab {
                id: tab.id,
                label: tab.label.clone(),
                active: tab.id == self.active_tab,
            })
            .collect();
        let active = self.tabs.iter().find(|tab| tab.id == self.active_tab);
        let can_go_back = active.is_some_and(|tab| tab.view.can_go_back().unwrap_or(false));
        let can_go_forward = active.is_some_and(|tab| tab.view.can_go_forward().unwrap_or(false));
        let address = active
            .filter(|tab| tab.url != NEW_TAB_URL)
            .map(|tab| tab.url.clone())
            .unwrap_or_default();
        let bookmarked = active.is_some_and(|tab| {
            self.bookmarks
                .iter()
                .any(|bookmark| bookmark.url == tab.url)
        });
        let state = json!({
            "tabs": tabs,
            "loading": self.tabs.iter().find(|tab| tab.id == self.active_tab).is_some_and(|tab| tab.loading),
            "can_go_back": can_go_back,
            "can_go_forward": can_go_forward,
            "address": address,
            "bookmarks": &self.bookmarks,
            "bookmarked": bookmarked,
            "history": &self.history,
            "downloads": &self.downloads,
            "settings": {
                "dark_mode": self.preferences.dark_mode,
                "ad_blocking": cfg!(target_os = "windows") && self.preferences.ad_blocking,
                "ad_blocking_available": cfg!(target_os = "windows"),
                "search_engine": self.preferences.search_engine
                ,"restore_last_page": self.preferences.restore_last_page,
                "language": self.preferences.language
                ,"version": env!("CARGO_PKG_VERSION")
                ,"update_repository": option_env!("NAGI_GITHUB_REPOSITORY").unwrap_or("")
                ,"update_asset": update_asset_name()
                ,"platform": if cfg!(target_os = "macos") { "macos" } else if cfg!(target_os = "linux") { "linux" } else { "windows" }
            }
        });
        let script = format!("window.nagiSetState({state});");
        let _ = self.toolbar.evaluate_script(&script);
        self.sync_home();
    }

    fn sync_home(&self) {
        let script = format!(
            "window.nagiSetHome({});",
            json!({ "bookmarks": &self.bookmarks, "dark_mode": self.preferences.dark_mode, "language": self.preferences.language })
        );
        for tab in &self.tabs {
            if tab.url == NEW_TAB_URL {
                let _ = tab.view.evaluate_script(&script);
            }
        }
    }

    fn navigate_tab(&mut self, id: u64, input: &str) {
        let target = resolve_input(input, &self.preferences.search_engine);
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
            tab.home_mode.store(false, Ordering::Relaxed);
            tab.url = target.clone();
            tab.label = tab_label(&target);
            if let Err(error) = tab.view.load_url(&target) {
                eprintln!("Navigation impossible : {error}");
            }
        }
        self.save_last_page();
        self.sync_toolbar();
    }

    fn show_home(&mut self, id: u64) {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
            tab.home_mode.store(true, Ordering::Relaxed);
            tab.url = NEW_TAB_URL.to_owned();
            tab.label = "Nouvel onglet".to_owned();
            if let Err(error) = tab.view.load_html(include_str!("../assets/newtab.html")) {
                eprintln!("Impossible d’ouvrir le nouvel onglet : {error}");
            }
        }
        self.save_last_page();
        self.sync_toolbar();
    }

    fn set_preference(&mut self, key: &str, value: &str) {
        match key {
            "theme" => self.preferences.dark_mode = value == "dark",
            "ad_blocking" => {
                self.preferences.ad_blocking = value == "on";
                self.ad_blocking_enabled
                    .store(self.preferences.ad_blocking, Ordering::Relaxed);
            }
            "search_engine" if matches!(value, "google" | "duckduckgo" | "bing") => {
                self.preferences.search_engine = value.to_owned();
            }
            "restore_last_page" => self.preferences.restore_last_page = value == "on",
            "language" => self.preferences.language = normalize_language(value).to_owned(),
            _ => return,
        }
        let _ = save_json(&self.preferences_file, &self.preferences);
        self.save_last_page();
        self.sync_toolbar();
    }

    fn save_last_page(&self) {
        let Some(tab) = self.tabs.iter().find(|tab| tab.id == self.active_tab) else {
            return;
        };
        let last_page = if self.preferences.restore_last_page && is_web_url(&tab.url) {
            Some(tab.url.as_str())
        } else {
            None
        };
        if let Err(error) = save_json(&self.last_page_file, &last_page) {
            eprintln!("Impossible d’enregistrer la dernière page : {error}");
        }
    }

    fn toggle_bookmark(&mut self) {
        let Some(tab) = self.tabs.iter().find(|tab| tab.id == self.active_tab) else {
            return;
        };
        if tab.url == NEW_TAB_URL {
            return;
        }
        let url = tab.url.clone();
        let title = tab.label.clone();
        if let Some(index) = self.bookmarks.iter().position(|item| item.url == url) {
            self.bookmarks.remove(index);
        } else {
            self.bookmarks.push(Bookmark { title, url });
        }
        if let Err(error) = save_bookmarks(&self.bookmarks_file, &self.bookmarks) {
            eprintln!("Impossible d’enregistrer les favoris : {error}");
        }
        self.sync_toolbar();
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Nagi n’a pas pu démarrer : {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut initial_inputs = Vec::new();
    let mut initial_language = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--language=") {
            initial_language = Some(value.to_owned());
        } else if arg == "--language" {
            initial_language = args.next();
        } else {
            initial_inputs.push(arg);
        }
    }
    let mut builder = EventLoopBuilder::<AppEvent>::with_user_event();
    let event_loop = builder.build();
    let mut browser = Browser::new(&event_loop, initial_inputs, initial_language)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(event) => browser.handle_event(event),
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } if window_id == browser.window.id() => {
                browser.save_last_page();
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(_),
                ..
            } if window_id == browser.window.id() => browser.resize(),
            _ => {}
        }
    });
}

fn profile_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("Library")
        .join("Application Support");
    #[cfg(target_os = "linux")]
    let base = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".local").join("share"))
        })
        .unwrap_or_else(env::temp_dir);
    let engine_profile = if cfg!(target_os = "windows") {
        "WebView2"
    } else {
        "WebKit"
    };
    let directory = base.join("NagiBrowser").join(engine_profile);
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn load_bookmarks(path: &std::path::Path) -> Vec<Bookmark> {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}

fn load_preferences(path: &Path) -> Preferences {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}

fn load_last_page(path: &Path) -> Option<String> {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice::<Option<String>>(&data).ok())
        .flatten()
        .filter(|url| is_web_url(url))
}

fn is_web_url(value: &str) -> bool {
    Url::parse(value)
        .ok()
        .is_some_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

fn update_asset_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "Nagi-macOS-universal.dmg"
    } else if cfg!(target_os = "linux") {
        "Nagi-Linux-x86_64.tar.gz"
    } else {
        "Nagi-Installer.exe"
    }
}

fn is_update_download_url(value: &str, repository: &str, asset_name: &str) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    let Some((owner, name)) = repository.split_once('/') else {
        return false;
    };
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return false;
    }
    let Some(segments) = url.path_segments() else {
        return false;
    };
    let segments: Vec<_> = segments.collect();
    url.scheme() == "https"
        && url.host_str() == Some("github.com")
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && segments.len() == 6
        && segments[0] == owner
        && segments[1] == name
        && segments[2] == "releases"
        && segments[3] == "download"
        && !segments[4].is_empty()
        && segments[5] == asset_name
}

fn normalize_language(value: &str) -> &'static str {
    match value.to_lowercase().as_str() {
        "en" | "english" => "en",
        "es" | "spanish" => "es",
        "de" | "german" => "de",
        "it" | "italian" => "it",
        "pt-br" | "ptbr" | "brazilianportuguese" | "pt" => "pt-BR",
        "nl" | "dutch" => "nl",
        "pl" | "polish" => "pl",
        "tr" | "turkish" => "tr",
        "ru" | "russian" => "ru",
        "ja" | "japanese" => "ja",
        "zh-cn" | "zhcn" | "chinesesimplified" => "zh-CN",
        "zh-tw" | "zhtw" | "chinesetraditional" => "zh-TW",
        "ar" | "arabic" => "ar",
        _ => "en",
    }
}

fn load_json_or_default<T: DeserializeOwned>(path: &Path) -> Vec<T> {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}

fn save_json<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let data = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;
    fs::write(path, data)
}

fn downloads_directory() -> PathBuf {
    #[cfg(target_os = "windows")]
    let home = env::var_os("USERPROFILE");
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    let home = env::var_os("HOME");
    home.map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("Downloads")
        .join("Nagi")
}

fn download_filename(url: &str) -> String {
    let candidate = Url::parse(url)
        .ok()
        .and_then(|url| {
            url.path_segments()
                .and_then(|mut segments| segments.next_back())
                .map(str::to_owned)
        })
        .unwrap_or_default();
    let cleaned: String = candidate
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, '.' | '-' | '_' | ' ' | '(' | ')')
            {
                character
            } else {
                '_'
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches(['.', ' ']);
    if cleaned.is_empty() {
        return "telechargement.bin".to_owned();
    }
    let base = cleaned
        .split('.')
        .next()
        .unwrap_or(cleaned)
        .to_ascii_uppercase();
    if matches!(
        base.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "LPT1"
    ) {
        format!("fichier_{cleaned}")
    } else {
        cleaned.to_owned()
    }
}

fn unique_download_path(directory: &Path, filename: &str) -> PathBuf {
    let original = directory.join(filename);
    if !original.exists() {
        return original;
    }
    let stem = original
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("fichier");
    let extension = original.extension().and_then(|value| value.to_str());
    for number in 2..=999 {
        let name = match extension {
            Some(extension) => format!("{stem} ({number}).{extension}"),
            None => format!("{stem} ({number})"),
        };
        let candidate = directory.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!(
        "{stem}-{}.download",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ))
}

fn save_bookmarks(path: &std::path::Path, bookmarks: &[Bookmark]) -> std::io::Result<()> {
    let data = serde_json::to_vec_pretty(bookmarks).map_err(std::io::Error::other)?;
    fs::write(path, data)
}

fn point_in_polygon(x: f32, y: f32, points: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn nagi_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    const N_MAIN: [(f32, f32); 11] = [
        (10.0, 20.0),
        (10.0, 9.0),
        (17.0, 9.0),
        (32.0, 26.0),
        (32.0, 9.0),
        (44.0, 9.0),
        (39.0, 14.0),
        (39.0, 39.0),
        (32.0, 39.0),
        (17.0, 22.0),
        (17.0, 20.0),
    ];
    const N_FOOT: [(f32, f32); 4] = [(10.0, 39.0), (10.0, 23.0), (17.0, 23.0), (17.0, 39.0)];
    const FOLD: [(f32, f32); 3] = [(39.0, 9.0), (44.0, 9.0), (39.0, 14.0)];
    let mut pixels = vec![0_u8; 64 * 64 * 4];
    for y in 0..64_u32 {
        for x in 0..64_u32 {
            let offset = ((y * 64 + x) * 4) as usize;
            let dx = x as i32 - 32;
            let dy = y as i32 - 32;
            let in_circle = dx * dx + dy * dy <= 31 * 31;
            let (mut r, mut g, mut b, mut alpha) = if in_circle {
                (24, 52, 71, 255)
            } else {
                (0, 0, 0, 0)
            };

            // Draw the approved folded-page N on the same grid as the SVG logo.
            let gx = 10.0 + (x as f32 - 12.0) / 1.2;
            let gy = 9.0 + (y as f32 - 14.0) / 1.2;
            let is_n = point_in_polygon(gx, gy, &N_MAIN) || point_in_polygon(gx, gy, &N_FOOT);
            if is_n {
                (r, g, b, alpha) = (245, 247, 250, 255);
            }
            if point_in_polygon(gx, gy, &FOLD) {
                (r, g, b, alpha) = (73, 169, 150, 255);
            }
            pixels[offset..offset + 4].copy_from_slice(&[r, g, b, alpha]);
        }
    }
    Ok(Icon::from_rgba(pixels, 64, 64)?)
}

#[cfg(target_os = "windows")]
fn enable_ad_blocking(view: &WebView, enabled: Arc<AtomicBool>) {
    use webview2_com::{
        CoTaskMemPWSTR,
        Microsoft::Web::WebView2::Win32::{COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL, ICoreWebView2_2},
        WebResourceRequestedEventHandler,
    };
    use windows::Win32::System::Com::IStream;
    use windows_core::{Interface, PWSTR, w};

    let core = view.webview();
    let result = (|| -> windows_core::Result<()> {
        // SAFETY: WebView2 COM calls occur on Wry's live UI-thread WebView.
        let env = unsafe { core.cast::<ICoreWebView2_2>()?.Environment()? };
        let mut token = 0_i64;
        let handler = WebResourceRequestedEventHandler::create(Box::new(move |_sender, args| {
            if !enabled.load(Ordering::Relaxed) {
                return Ok(());
            }
            let Some(args) = args else { return Ok(()) };
            // SAFETY: The event arguments and request are valid for this callback's duration.
            let request = unsafe { args.Request()? };
            let mut raw_uri = PWSTR::null();
            unsafe { request.Uri(&mut raw_uri)? };
            let uri = CoTaskMemPWSTR::from(raw_uri).to_string();
            if is_ad_host(&uri) {
                // A 204 response stops the matching request without loading an error page.
                // SAFETY: The captured WebView2 environment is retained for the callback.
                let response = unsafe {
                    env.CreateWebResourceResponse(None::<&IStream>, 204, w!("No Content"), w!(""))?
                };
                unsafe { args.SetResponse(&response)? };
            }
            Ok(())
        }));
        // Filter only known ad and tracker domains so normal site requests bypass this callback.
        unsafe {
            for domain in AD_HOSTS {
                for pattern in [format!("*://{domain}/*"), format!("*://*.{domain}/*")] {
                    let filter = CoTaskMemPWSTR::from(pattern.as_str());
                    core.AddWebResourceRequestedFilter(
                        *filter.as_ref().as_pcwstr(),
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                    )?;
                }
            }
            // SAFETY: Callback and filters are registered on this live WebView2 instance.
            core.add_WebResourceRequested(&handler, &mut token)?;
        }
        let _ = token;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Le bloqueur de publicités WebView2 n’a pas pu démarrer : {error}");
    }
}

#[cfg(target_os = "macos")]
fn enable_ad_blocking(_view: &WebView, _enabled: Arc<AtomicBool>) {}

#[cfg(target_os = "linux")]
fn enable_ad_blocking(_view: &WebView, _enabled: Arc<AtomicBool>) {}

#[cfg(any(target_os = "windows", test))]
fn is_ad_host(uri: &str) -> bool {
    let Ok(parsed) = Url::parse(uri) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    AD_HOSTS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

#[cfg(target_os = "windows")]
fn enable_password_autosave(view: &WebView) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings4;
    use windows_core::Interface;

    let core = view.webview();
    let result = (|| {
        // SAFETY: Wry owns this WebView2 COM object, and this function runs on the GUI event-loop
        // thread while the WebView is alive. The settings interface is queried from that object.
        let settings = unsafe { core.Settings()? };
        let settings4 = settings.cast::<ICoreWebView2Settings4>()?;
        // SAFETY: The settings object was queried from the live WebView2 instance on its UI thread.
        unsafe { settings4.SetIsPasswordAutosaveEnabled(true) }
    })();

    if let Err(error) = result {
        eprintln!("La sauvegarde des mots de passe WebView2 n’est pas disponible : {error}");
    }
}

#[cfg(target_os = "macos")]
fn enable_password_autosave(_view: &WebView) {}

#[cfg(target_os = "linux")]
fn enable_password_autosave(_view: &WebView) {}

#[cfg(target_os = "windows")]
fn set_memory_usage_level(view: &WebView, active: bool) {
    let level = if active {
        MemoryUsageLevel::Normal
    } else {
        MemoryUsageLevel::Low
    };
    let _ = view.set_memory_usage_level(level);
}

#[cfg(target_os = "macos")]
fn set_memory_usage_level(_view: &WebView, _active: bool) {}

#[cfg(target_os = "linux")]
fn set_memory_usage_level(_view: &WebView, _active: bool) {}

fn toolbar_bounds(window: &Window, overlay: bool) -> Rect {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(
            size.width,
            if overlay {
                TOOLBAR_OVERLAY_HEIGHT.min(size.height)
            } else {
                TOOLBAR_HEIGHT.min(size.height)
            },
        )
        .into(),
    }
}

#[cfg(target_os = "windows")]
fn raise_toolbar(toolbar: &WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{
        HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
    };

    // SAFETY: The WebView HWND is owned by the live toolbar, and this runs on the UI thread.
    let _ = unsafe {
        SetWindowPos(
            toolbar.hwnd(),
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        )
    };
}

#[cfg(target_os = "macos")]
fn raise_toolbar(toolbar: &WebView) {
    use objc2_app_kit::{NSView, NSWindowOrderingMode};

    let webview = toolbar.webview();
    let toolbar_view: &NSView = &webview;
    // SAFETY: Wry owns this live WKWebView, and this runs on the GUI event-loop thread.
    if let Some(parent) = unsafe { toolbar_view.superview() } {
        parent.addSubview_positioned_relativeTo(toolbar_view, NSWindowOrderingMode::Above, None);
    }
}

#[cfg(target_os = "linux")]
fn raise_toolbar(toolbar: &WebView, container: &gtk::Fixed) {
    let _ = toolbar.reparent(container);
}

fn reveal_download(path: &Path) {
    #[cfg(target_os = "windows")]
    let mut command = process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = process::Command::new("open");
    #[cfg(target_os = "macos")]
    command.arg("-R");
    #[cfg(target_os = "linux")]
    let mut command = process::Command::new("xdg-open");
    #[cfg(target_os = "linux")]
    let path = path.parent().unwrap_or(path);
    let _ = command.arg(path).spawn();
}

fn open_in_file_manager(path: &Path) {
    #[cfg(target_os = "windows")]
    let mut command = process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = process::Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = process::Command::new("xdg-open");
    let _ = command.arg(path).spawn();
}

fn open_external_url(url: &str) {
    #[cfg(target_os = "windows")]
    let mut command = process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = process::Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = process::Command::new("xdg-open");
    let _ = command.arg(url).spawn();
}

fn view_bounds(window: &Window) -> Rect {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    let toolbar = TOOLBAR_HEIGHT.min(size.height);
    Rect {
        position: WebPosition::new(0.0, toolbar).into(),
        size: WebSize::new(size.width, (size.height - toolbar).max(1.0)).into(),
    }
}

fn resolve_input(input: &str, search_engine: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return SEARCH_HOME_URL.to_owned();
    }

    if let Ok(parsed) = Url::parse(trimmed)
        && matches!(parsed.scheme(), "http" | "https")
        && parsed.host_str().is_some()
    {
        return parsed.into();
    }

    if !trimmed.chars().any(char::is_whitespace)
        && (trimmed.starts_with("localhost") || trimmed.contains('.'))
    {
        let candidate = format!("https://{trimmed}");
        if let Ok(parsed) = Url::parse(&candidate)
            && parsed.host_str().is_some()
        {
            return parsed.into();
        }
    }

    let encoded = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("q", trimmed)
        .finish();
    let base = match search_engine {
        "duckduckgo" => "https://duckduckgo.com/?",
        "bing" => "https://www.bing.com/search?",
        _ => "https://www.google.com/search?",
    };
    format!("{base}{encoded}")
}

fn tab_label(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_owned))
        .unwrap_or_else(|| "Nouvel onglet".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_link_accepts_only_the_exact_release_asset() {
        assert!(is_update_download_url(
            "https://github.com/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe",
            "kenzo/nagi-browser",
            "Nagi-Installer.exe"
        ));
        assert!(is_update_download_url(
            "https://github.com/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-macOS-universal.dmg",
            "kenzo/nagi-browser",
            "Nagi-macOS-universal.dmg"
        ));
    }

    #[test]
    fn update_link_rejects_untrusted_urls_and_paths() {
        let repository = "kenzo/nagi-browser";
        for url in [
            "http://github.com/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe",
            "https://github.com.evil.test/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe",
            "https://github.com/other/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe",
            "https://github.com/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe/extra",
            "https://github.com/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe?next=https://evil.test",
            "https://user@github.com/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe",
            "https://github.com:444/kenzo/nagi-browser/releases/download/v0.4.0/Nagi-Installer.exe",
        ] {
            assert!(
                !is_update_download_url(url, repository, "Nagi-Installer.exe"),
                "accepted {url}"
            );
        }
    }

    #[test]
    fn search_input_does_not_navigate_to_script_schemes() {
        let result = resolve_input("javascript:alert(1)", "google");
        assert!(result.starts_with("https://www.google.com/search?"));
        assert!(!result.starts_with("javascript:"));
    }

    #[test]
    fn ad_host_matching_respects_domain_boundaries() {
        assert!(is_ad_host("https://doubleclick.net/ad.js"));
        assert!(is_ad_host("https://sub.doubleclick.net/ad.js"));
        assert!(!is_ad_host("https://notdoubleclick.net/ad.js"));
    }
}
