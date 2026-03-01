use crate::models::ItemListSummary;
use crate::server::items::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

#[component]
pub fn ListsHome() -> impl IntoView {
    let lists = Resource::new(|| (), |_| get_item_lists_with_stats());
    let new_name = RwSignal::new(String::new());
    let creating = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let on_create = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = new_name.get();
        if name.trim().is_empty() {
            return;
        }
        creating.set(true);
        error.set(None);
        spawn_local(async move {
            match create_item_list(name).await {
                Ok(_) => {
                    new_name.set(String::new());
                    lists.refetch();
                }
                Err(e) => error.set(Some(e.to_string())),
            }
            creating.set(false);
        });
    };

    view! {
        <div class="page">
            <div class="page-header">
                <h1>"My Lists"</h1>
            </div>

            <form class="new-list-form" on:submit=on_create>
                <input
                    type="text"
                    placeholder="New list name…"
                    prop:value=new_name
                    on:input=move |ev| new_name.set(event_target_value(&ev))
                />
                <button type="submit" class="btn btn-primary" disabled=creating>
                    "Create List"
                </button>
            </form>

            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}

            <Suspense fallback=|| view! { <div class="loading">"Loading lists…"</div> }>
                {move || lists.get().map(|result| match result {
                    Err(e) => view! {
                        <p class="error">"Error loading lists: " {e.to_string()}</p>
                    }.into_any(),
                    Ok(ls) if ls.is_empty() => view! {
                        <div class="empty-state">
                            <p>"No lists yet. Create your first list above."</p>
                        </div>
                    }.into_any(),
                    Ok(ls) => view! {
                        <div class="list-grid">
                            <For
                                each=move || ls.clone()
                                key=|l| l.id
                                children=|list| view! { <ListCard list=list /> }
                            />
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn ListCard(list: ItemListSummary) -> impl IntoView {
    let href = format!("/list/{}", list.id);
    view! {
        <a href=href class="list-card">
            <div class="list-card-name">{list.name}</div>
            <div class="list-card-stats">
                <span class="list-card-count">
                    {list.active_count} {if list.active_count == 1 { " item" } else { " items" }}
                </span>
                {(list.estimated_budget > 0.0).then(|| view! {
                    <span class="list-card-budget">
                        {format!("{:.2} €", list.estimated_budget)}
                    </span>
                })}
            </div>
        </a>
    }
}
