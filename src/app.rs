use crate::components::{
    archive::Archive, list_archive::ListArchive, lists_home::ListsHome, wish_form::WishForm,
    wish_list::WishList,
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    ParamSegment, StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/wishr.css"/>
        <Title text="wishr"/>

        <Router>
            <header class="app-header">
                <div class="header-inner">
                    <a href="/" class="logo">"wishr"</a>
                    <nav>
                        <A href="/" exact=true>"Lists"</A>
                        <A href="/archive">"Archive"</A>
                    </nav>
                </div>
            </header>

            <main>
                <Routes fallback=|| view! { <h2 class="not-found">"404 — Page not found"</h2> }>
                    <Route path=StaticSegment("") view=ListsHome />
                    <Route path=(StaticSegment("list"), ParamSegment("list_id")) view=WishList />
                    <Route path=(StaticSegment("list"), ParamSegment("list_id"), StaticSegment("archive")) view=ListArchive />
                    <Route path=(StaticSegment("list"), ParamSegment("list_id"), StaticSegment("add")) view=WishForm />
                    <Route path=(StaticSegment("list"), ParamSegment("list_id"), StaticSegment("edit"), ParamSegment("id")) view=WishForm />
                    <Route path=StaticSegment("archive") view=Archive />
                </Routes>
            </main>
        </Router>
    }
}
