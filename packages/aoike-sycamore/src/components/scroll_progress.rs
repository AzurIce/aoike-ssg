use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use sycamore::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

#[derive(Clone, Copy)]
pub(crate) struct ScrollProgressContext {
    pub active: Signal<bool>,
    pub percent: Signal<f64>,
}

impl ScrollProgressContext {
    pub fn new() -> Self {
        Self {
            active: create_signal(false),
            percent: create_signal(0.0),
        }
    }
}

#[component]
pub(crate) fn ScrollProgressControls() -> View {
    let progress = use_context::<ScrollProgressContext>();

    view! {
        div(
            class=move || if progress.active.get() {
                "scroll-progress visible"
            } else {
                "scroll-progress"
            },
            aria-hidden="true",
        ) {
            div(
                class="scroll-progress-bar",
                style=move || format!(
                    "transform: scaleX({:.6})",
                    progress.percent.get().clamp(0.0, 100.0) / 100.0,
                ),
            )
        }
        div(
            class=move || if progress.active.get() {
                "scroll-progress-pill visible"
            } else {
                "scroll-progress-pill"
            },
            style=move || format!(
                "left: clamp(1rem, {:.4}%, calc(100% - 1rem))",
                progress.percent.get().clamp(0.0, 100.0),
            ),
        ) {
            (move || format!("{:.0}%", progress.percent.get().clamp(0.0, 100.0)))
        }
        button(
            class=move || if progress.active.get() && progress.percent.get() > 0.5 {
                "scroll-top visible"
            } else {
                "scroll-top"
            },
            aria-label="返回顶部",
            title="返回顶部",
            on:click=move |_| {
                if let Some(window) = web_sys::window() {
                    let options = web_sys::ScrollToOptions::new();
                    options.set_top(0.0);
                    options.set_behavior(web_sys::ScrollBehavior::Smooth);
                    window.scroll_to_with_scroll_to_options(&options);
                }
            },
        ) {
            span(class="scroll-top-icon i-lucide-arrow-up", aria-hidden="true")
        }
    }
}

#[component]
pub(crate) fn DocumentScrollProgress() -> View {
    let progress = use_context::<ScrollProgressContext>();
    let observers = Rc::new(RefCell::new(None::<DocumentProgressObservers>));

    on_mount({
        let observers = observers.clone();
        move || {
            progress.active.set(true);
            progress.percent.set(0.0);

            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(document_element) = window
                .document()
                .and_then(|document| document.document_element())
            else {
                return;
            };

            let measured_window = window.clone();
            let update_progress = Rc::new(move || {
                let Some(percent) = document_scroll_percent(&measured_window) else {
                    return;
                };
                progress.percent.set(percent);
            });
            let schedule_progress_update = schedule_animation_frame(update_progress.clone());

            let scroll_schedule = schedule_progress_update.clone();
            let scroll_closure = Closure::wrap(Box::new(move || {
                scroll_schedule();
            }) as Box<dyn FnMut()>);
            if window
                .add_event_listener_with_callback("scroll", scroll_closure.as_ref().unchecked_ref())
                .is_err()
            {
                return;
            }

            let resize_schedule = schedule_progress_update.clone();
            let resize_closure = Closure::wrap(Box::new(move || {
                resize_schedule();
            }) as Box<dyn FnMut()>);
            if window
                .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
                .is_err()
            {
                let _ = window.remove_event_listener_with_callback(
                    "scroll",
                    scroll_closure.as_ref().unchecked_ref(),
                );
                return;
            }

            let content_schedule = schedule_progress_update;
            let content_resize_closure = Closure::wrap(Box::new(move |_: js_sys::Array| {
                content_schedule();
            })
                as Box<dyn FnMut(js_sys::Array)>);
            let Ok(content_resize_observer) =
                web_sys::ResizeObserver::new(content_resize_closure.as_ref().unchecked_ref())
            else {
                let _ = window.remove_event_listener_with_callback(
                    "scroll",
                    scroll_closure.as_ref().unchecked_ref(),
                );
                let _ = window.remove_event_listener_with_callback(
                    "resize",
                    resize_closure.as_ref().unchecked_ref(),
                );
                return;
            };
            content_resize_observer.observe(&document_element);

            update_progress();
            observers.borrow_mut().replace(DocumentProgressObservers {
                content_resize_observer,
                _content_resize_closure: content_resize_closure,
                window,
                _scroll_closure: scroll_closure,
                _resize_closure: resize_closure,
            });
        }
    });

    on_cleanup({
        let observers = observers.clone();
        move || {
            observers.borrow_mut().take();
            progress.active.set(false);
            progress.percent.set(0.0);
        }
    });

    view! {}
}

struct DocumentProgressObservers {
    content_resize_observer: web_sys::ResizeObserver,
    _content_resize_closure: Closure<dyn FnMut(js_sys::Array)>,
    window: web_sys::Window,
    _scroll_closure: Closure<dyn FnMut()>,
    _resize_closure: Closure<dyn FnMut()>,
}

impl Drop for DocumentProgressObservers {
    fn drop(&mut self) {
        self.content_resize_observer.disconnect();
        let _ = self.window.remove_event_listener_with_callback(
            "scroll",
            self._scroll_closure.as_ref().unchecked_ref(),
        );
        let _ = self.window.remove_event_listener_with_callback(
            "resize",
            self._resize_closure.as_ref().unchecked_ref(),
        );
    }
}

fn schedule_animation_frame(update: Rc<dyn Fn()>) -> Rc<dyn Fn()> {
    let frame_pending = Rc::new(Cell::new(false));
    Rc::new(move || {
        if frame_pending.replace(true) {
            return;
        }
        let callback_pending = frame_pending.clone();
        let update = update.clone();
        let callback = Closure::once_into_js(move || {
            callback_pending.set(false);
            update();
        });
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(callback.unchecked_ref());
        } else {
            frame_pending.set(false);
        }
    })
}

fn document_scroll_percent(window: &web_sys::Window) -> Option<f64> {
    let scroll_y = window.scroll_y().ok()?;
    let viewport_height = window.inner_height().ok()?.as_f64()?;
    let document_height = window.document()?.document_element()?.scroll_height() as f64;
    Some(scroll_percent(scroll_y, viewport_height, document_height))
}

fn scroll_percent(scroll_y: f64, viewport_height: f64, document_height: f64) -> f64 {
    let max_scroll_y = (document_height - viewport_height).max(0.0);
    if max_scroll_y == 0.0 {
        0.0
    } else {
        (scroll_y / max_scroll_y * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::scroll_percent;

    #[test]
    fn document_progress_uses_the_actual_scrollable_distance() {
        assert_eq!(scroll_percent(0.0, 600.0, 1800.0), 0.0);
        assert_eq!(scroll_percent(600.0, 600.0, 1800.0), 50.0);
        assert_eq!(scroll_percent(1200.0, 600.0, 1800.0), 100.0);
        assert_eq!(scroll_percent(0.0, 800.0, 700.0), 0.0);
    }
}
