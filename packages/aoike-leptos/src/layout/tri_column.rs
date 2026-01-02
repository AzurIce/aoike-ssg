use leptos::prelude::*;

#[slot]
pub struct Left {
    children: Children,
}

#[slot]
pub struct Main {
    children: Children,
}

#[slot]
pub struct Right {
    children: Children,
}

#[component]
pub fn TriColumn(
    #[prop(optional)] left: Option<Left>,
    main: Main,
    #[prop(optional)] right: Option<Right>,
) -> impl IntoView {
    view! {
        <div class="flex w-full justify-center m-x-auto">
            <aside class="flex-1 flex justify-end">
                <div class="sticky top-14 h-[calc(100vh-3.5rem)] overflow-y-auto">
                    {left.map(|x| (x.children)())}
                </div>
            </aside>
            <main class="max-w-[80ch] w-full flex flex-col items-center p-8 gap-4 shrink-0">
                {(main.children)()}
            </main>
            <aside class="flex-1 flex justify-start">
                <div class="sticky top-14 h-[calc(100vh-3.5rem)] overflow-y-auto">
                    {right.map(|x| (x.children)())}
                </div>
            </aside>
        </div>
    }
}
