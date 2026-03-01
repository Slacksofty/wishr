use crate::server::items::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;
use leptos_router::hooks::{use_navigate, use_params_map};

#[component]
pub fn WishForm() -> impl IntoView {
    let params = use_params_map();
    let list_id = move || {
        params.with(|p| {
            p.get("list_id")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(1)
        })
    };
    let edit_id = move || params.with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()));
    let is_edit = move || edit_id().is_some();

    // Form fields
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let estimated_cost = RwSignal::new(String::new());
    let want_level = RwSignal::new(3i64);
    let need_level = RwSignal::new(3i64);
    let where_to_buy = RwSignal::new(String::new());
    let category = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let submitting = RwSignal::new(false);

    // Prefill form when editing
    let existing = Resource::new(
        move || edit_id(),
        |id| async move {
            match id {
                Some(id) => get_wish_item(id).await.ok(),
                None => None,
            }
        },
    );

    // Populate signals when item loads
    Effect::new(move |_| {
        if let Some(Some(item)) = existing.get() {
            name.set(item.name);
            description.set(item.description.unwrap_or_default());
            estimated_cost.set(
                item.estimated_cost
                    .map(|c| format!("{:.2}", c))
                    .unwrap_or_default(),
            );
            want_level.set(item.want_level);
            need_level.set(item.need_level);
            where_to_buy.set(item.where_to_buy.unwrap_or_default());
            category.set(item.category.unwrap_or_default());
            notes.set(item.notes.unwrap_or_default());
        }
    });

    let navigate = use_navigate();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if name.get().trim().is_empty() {
            error.set(Some("Name is required.".into()));
            return;
        }
        submitting.set(true);
        let cost = estimated_cost.get().parse::<f64>().ok();
        let navigate = navigate.clone();
        let id = edit_id();
        let lid = list_id();
        spawn_local(async move {
            let result = if let Some(id) = id {
                update_wish_item(
                    id,
                    name.get_untracked(),
                    description.get_untracked(),
                    cost,
                    want_level.get_untracked(),
                    need_level.get_untracked(),
                    where_to_buy.get_untracked(),
                    category.get_untracked(),
                    notes.get_untracked(),
                )
                .await
                .map(|_| ())
            } else {
                create_wish_item(
                    lid,
                    name.get_untracked(),
                    description.get_untracked(),
                    cost,
                    want_level.get_untracked(),
                    need_level.get_untracked(),
                    where_to_buy.get_untracked(),
                    category.get_untracked(),
                    notes.get_untracked(),
                )
                .await
                .map(|_| ())
            };

            match result {
                Ok(_) => navigate(&format!("/list/{lid}"), Default::default()),
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="page">
            <div class="page-header">
                <h1>{move || if is_edit() { "Edit Item" } else { "Add Item" }}</h1>
            </div>

            <form class="wish-form" on:submit=on_submit>
                <div class="form-group">
                    <label class="required">"Name"</label>
                    <input
                        type="text"
                        placeholder="e.g. LEGO Technic Set"
                        required
                        prop:value=name
                        on:input=move |ev| name.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label>"Estimated Cost (€)"</label>
                        <input
                            type="number"
                            step="0.01"
                            min="0"
                            placeholder="0.00"
                            prop:value=estimated_cost
                            on:input=move |ev| estimated_cost.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Category"</label>
                        <input
                            type="text"
                            placeholder="e.g. Tech, Books, Hobbies"
                            prop:value=category
                            on:input=move |ev| category.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label>"Want Level: " {want_level} "/5"</label>
                        <input
                            type="range"
                            min="1"
                            max="5"
                            prop:value=move || want_level.get().to_string()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                    want_level.set(v);
                                }
                            }
                        />
                        <div class="range-labels"><span>"1 (meh)"</span><span>"5 (really want)"</span></div>
                    </div>
                    <div class="form-group">
                        <label>"Need Level: " {need_level} "/5"</label>
                        <input
                            type="range"
                            min="1"
                            max="5"
                            prop:value=move || need_level.get().to_string()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                    need_level.set(v);
                                }
                            }
                        />
                        <div class="range-labels"><span>"1 (luxury)"</span><span>"5 (essential)"</span></div>
                    </div>
                </div>

                <div class="form-group">
                    <label>"Where to Buy"</label>
                    <input
                        type="text"
                        placeholder="URL or store name"
                        prop:value=where_to_buy
                        on:input=move |ev| where_to_buy.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Description"</label>
                    <textarea
                        placeholder="Optional description or details"
                        prop:value=description
                        on:input=move |ev| description.set(event_target_value(&ev))
                        rows="3"
                    ></textarea>
                </div>

                <div class="form-group">
                    <label>"Notes"</label>
                    <textarea
                        placeholder="Any other notes (price drops, alternatives, etc.)"
                        prop:value=notes
                        on:input=move |ev| notes.set(event_target_value(&ev))
                        rows="2"
                    ></textarea>
                </div>

                {move || error.get().map(|e| view! { <p class="error">{e}</p> })}

                <div class="form-actions">
                    <a href=move || format!("/list/{}", list_id()) class="btn">"Cancel"</a>
                    <button type="submit" class="btn btn-primary" disabled=submitting>
                        {move || if submitting.get() { "Saving…" } else if is_edit() { "Save Changes" } else { "Add to List" }}
                    </button>
                </div>
            </form>
        </div>
    }
}
