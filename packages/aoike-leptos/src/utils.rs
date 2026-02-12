use crate::BASE_URL;

pub fn based_url(path: impl AsRef<str>) -> String {
    let path = path.as_ref().trim_start_matches('/');
    let base = BASE_URL.trim_matches('/');

    if base.is_empty() {
        format!("/{path}")
    } else {
        format!("/{base}/{path}")
    }
}

// From https://github.com/thaw-ui/thaw/blob/main/thaw_utils/src/dom/mount_style.rs
pub fn mount_style(id: &str, content: &'static str) {
    let id = format!("aoike-id-{id}");
    use leptos::prelude::document;
    let head = document().head().expect("head no exist");
    let style = head
        .query_selector(&format!("style#{id}"))
        .expect("query style element error");

    if style.is_some() {
        return;
    }

    let style = document()
        .create_element("style")
        .expect("create style element error");
    _ = style.set_attribute("id", &id);
    style.set_text_content(Some(content));

    let aoike_meta = head
        .query_selector(&format!(r#"meta[name="aoike-ui-style"]"#))
        .expect(r#"query meta[name="aoike-ui-style"] element error"#);

    if let Some(thaw_meta) = aoike_meta {
        let _ = head.insert_before(&style, Some(&thaw_meta));
    } else {
        let _ = head.prepend_with_node_1(&style);
    }
}
