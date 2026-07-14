use std::sync::atomic::{AtomicUsize, Ordering};

use sycamore::prelude::*;

/// Waline login mode.
///
/// See: https://waline.js.org/reference/client/props.html#login
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalineLogin {
    /// Users can choose to login or comment anonymously.
    Enable,
    /// Login is disabled, users must fill in information to comment.
    Disable,
    /// Forced login, users must login to comment.
    Force,
}

impl WalineLogin {
    fn as_str(&self) -> &'static str {
        match self {
            WalineLogin::Enable => "enable",
            WalineLogin::Disable => "disable",
            WalineLogin::Force => "force",
        }
    }
}

/// Waline dark mode option.
///
/// See: https://waline.js.org/reference/client/props.html#dark
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalineDarkMode {
    /// Follow device/system preference.
    Auto,
    /// Explicitly enable or disable dark mode.
    Bool(bool),
    /// CSS selector that matches a dark-mode ancestor.
    Selector(String),
}

impl WalineDarkMode {
    fn as_js(&self) -> String {
        match self {
            WalineDarkMode::Auto => "'auto'".to_string(),
            WalineDarkMode::Bool(v) => v.to_string(),
            WalineDarkMode::Selector(s) => format!("'{}'", escape_js_string(s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalineOptions {
    pub server_url: String,
    pub path: String,
    pub lang: String,
    pub login: WalineLogin,
    pub dark: WalineDarkMode,
    pub page_size: usize,
    pub word_limit: Option<(usize, usize)>,
    pub theme_color: String,
    pub reaction: bool,
}

impl WalineOptions {
    pub fn new(server_url: String, path: String) -> Self {
        Self {
            server_url,
            path,
            lang: "zh-CN".to_string(),
            login: WalineLogin::Enable,
            dark: WalineDarkMode::Auto,
            page_size: 10,
            word_limit: None,
            theme_color: "var(--accent-color)".to_string(),
            reaction: false,
        }
    }

    pub fn with_lang(mut self, lang: String) -> Self {
        self.lang = lang;
        self
    }

    /// Convenience: `true` => `WalineLogin::Enable`, `false` => `WalineLogin::Disable`.
    /// Use [`with_login_mode`] if you need `WalineLogin::Force`.
    pub fn with_login(mut self, login: bool) -> Self {
        self.login = if login {
            WalineLogin::Enable
        } else {
            WalineLogin::Disable
        };
        self
    }

    pub fn with_login_mode(mut self, login: WalineLogin) -> Self {
        self.login = login;
        self
    }

    pub fn with_dark_mode(mut self, dark: WalineDarkMode) -> Self {
        self.dark = dark;
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn with_word_limit(mut self, min: usize, max: usize) -> Self {
        self.word_limit = Some((min, max));
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = path;
        self
    }

    pub fn with_theme_color(mut self, theme_color: String) -> Self {
        self.theme_color = theme_color;
        self
    }

    pub fn with_reaction(mut self, reaction: bool) -> Self {
        self.reaction = reaction;
        self
    }
}

const WALINE_CSS_ID: &str = "waline-client-css";
const WALINE_SCRIPT_URL: &str = "https://unpkg.com/@waline/client@v3/dist/waline.js";
const WALINE_CSS_URL: &str = "https://unpkg.com/@waline/client@v3/dist/waline.css";

static WALINE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn inject_waline_css() {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    if document.get_element_by_id(WALINE_CSS_ID).is_some() {
        return;
    }

    if let Ok(link) = document.create_element("link") {
        let _ = link.set_attribute("id", WALINE_CSS_ID);
        let _ = link.set_attribute("rel", "stylesheet");
        let _ = link.set_attribute("href", WALINE_CSS_URL);
        let _ = document
            .query_selector("head")
            .ok()
            .flatten()
            .and_then(|head| head.append_child(&link).ok());
    }
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
}

fn build_options_js(id: &str, options: &WalineOptions) -> String {
    let mut parts = vec![
        format!("el: '#{}'", id),
        format!("serverURL: '{}'", escape_js_string(&options.server_url)),
        format!("lang: '{}'", escape_js_string(&options.lang)),
        format!("login: '{}'", options.login.as_str()),
        format!("dark: {}", options.dark.as_js()),
        format!("pageSize: {}", options.page_size),
        format!("reaction: {}", options.reaction),
    ];

    if !options.path.is_empty() {
        parts.push(format!("path: '{}'", escape_js_string(&options.path)));
    }

    if let Some((min, max)) = options.word_limit {
        parts.push(format!("wordLimit: [{}, {}]", min, max));
    }

    format!("{{ {} }}", parts.join(", "))
}

#[component(inline_props)]
pub fn Waline(options: WalineOptions) -> View {
    let id = format!(
        "waline-{}",
        WALINE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    );

    let container_id = id.clone();
    let style_id = format!("{}-style", id);
    let mount_style_id = style_id.clone();
    let cleanup_style_id = style_id.clone();
    let mount_options = options.clone();

    on_mount(move || {
        inject_waline_css();

        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let document = match window.document() {
            Some(d) => d,
            None => return,
        };

        // Inject per-instance theme CSS variables.
        if document.get_element_by_id(&mount_style_id).is_none() {
            if let Ok(style) = document.create_element("style") {
                let _ = style.set_attribute("id", &mount_style_id);
                let css = format!(
                    "#{} {{\n  --waline-theme-color: {};\n  --waline-active-color: {};\n}}",
                    container_id,
                    escape_js_string(&mount_options.theme_color),
                    escape_js_string(&mount_options.theme_color)
                );
                let _ = style.set_text_content(Some(&css));
                let _ = document
                    .query_selector("head")
                    .ok()
                    .flatten()
                    .and_then(|head| head.append_child(&style).ok());
            }
        }

        let module_script = document.create_element("script").ok().and_then(|el| {
            el.set_attribute("type", "module").ok()?;
            Some(el)
        });

        if let Some(script) = module_script {
            let opts_js = build_options_js(&container_id, &mount_options);
            let content = format!(
                "import {{ init }} from '{}';\nconst el = document.getElementById('{}');\nif (el) {{\n  const instance = init({});\n  if (!window.__walineInstances) window.__walineInstances = {{}};\n  window.__walineInstances['{}'] = instance;\n}}",
                WALINE_SCRIPT_URL, container_id, opts_js, container_id
            );
            let _ = script.set_text_content(Some(&content));
            let _ = document
                .query_selector("head")
                .ok()
                .flatten()
                .and_then(|head| head.append_child(&script).ok());
        }
    });

    let cleanup_id = id.clone();
    on_cleanup(move || {
        let js = format!(
            "(() => {{\n  const map = window.__walineInstances || {{}};\n  const inst = map['{}'];\n  if (inst && typeof inst.destroy === 'function') inst.destroy();\n  delete map['{}'];\n  const style = document.getElementById('{}');\n  if (style) style.remove();\n}})();",
            cleanup_id, cleanup_id, cleanup_style_id
        );
        let _ = js_sys::eval(&js);
    });

    view! {
        div(id=id, class="waline-comments") {}
    }
}
