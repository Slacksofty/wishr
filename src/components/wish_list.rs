use crate::models::{ItemListSummary, WishItem};
use crate::server::items::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;
use leptos_router::hooks::use_params_map;

#[component]
pub fn WishList() -> impl IntoView {
    let params = use_params_map();
    let list_id = move || {
        params.with(|p| {
            p.get("list_id")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };

    // Returns ItemListSummary which includes estimated_budget
    let list_info = Resource::new(list_id, get_item_list_detail);
    let items = Resource::new(list_id, get_wish_items);
    let all_lists = Resource::new(|| (), |_| get_item_lists_with_stats());

    let refetch_items = move || items.refetch();
    let refetch_info = move || list_info.refetch();

    // Rename signals
    let editing_name = RwSignal::new(false);
    let draft_name = RwSignal::new(String::new());

    let on_rename_start = move |current_name: String| {
        draft_name.set(current_name);
        editing_name.set(true);
    };

    let on_rename_save = move |_: web_sys::MouseEvent| {
        let name = draft_name.get();
        if name.trim().is_empty() {
            editing_name.set(false);
            return;
        }
        let lid = list_id();
        spawn_local(async move {
            let _ = rename_item_list(lid, name).await;
            list_info.refetch();
            editing_name.set(false);
        });
    };

    view! {
        <div class="page">
            <div class="page-header">
                <div class="page-title-group">
                    <a href="/" class="back-link">"← Lists"</a>
                    <Suspense fallback=|| view! { <h1>"…"</h1> }>
                        {move || list_info.get().map(|r| {
                            let info = r.unwrap_or(ItemListSummary {
                                id: 0, name: "Unknown".into(), created_at: String::new(),
                                active_count: 0, estimated_budget: 0.0,
                            });
                            let name_for_edit = info.name.clone();
                            view! {
                                <div class="list-title-area">
                                    {move || if editing_name.get() {
                                        view! {
                                            <div class="rename-form">
                                                <input
                                                    type="text"
                                                    class="rename-input"
                                                    prop:value=draft_name
                                                    on:input=move |ev| draft_name.set(event_target_value(&ev))
                                                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                        if ev.key() == "Escape" { editing_name.set(false); }
                                                    }
                                                />
                                                <button class="btn btn-sm btn-primary" on:click=on_rename_save>"Save"</button>
                                                <button class="btn btn-sm" on:click=move |_| editing_name.set(false)>"Cancel"</button>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let n = name_for_edit.clone();
                                        view! {
                                            <div class="list-title-row">
                                                <h1>{info.name.clone()}</h1>
                                                <button
                                                    class="btn btn-sm"
                                                    on:click=move |_| on_rename_start(n.clone())
                                                >"Rename"</button>
                                            </div>
                                        }.into_any()
                                    }}
                                    {(info.estimated_budget > 0.0).then(|| view! {
                                        <span class="pending-badge">
                                            "Pending: " {format!("{:.2} €", info.estimated_budget)}
                                        </span>
                                    })}
                                </div>
                            }
                        })}
                    </Suspense>
                </div>
                <div class="page-header-actions">
                    <a href=move || format!("/list/{}/archive", list_id()) class="btn">"Archive"</a>
                    <a href=move || format!("/list/{}/add", list_id()) class="btn btn-primary">
                        "+ Add Item"
                    </a>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="loading">"Loading items…"</div> }>
                {move || {
                    let other_lists: Vec<ItemListSummary> = all_lists.get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|l| l.id != list_id())
                        .collect();

                    items.get().map(|result| match result {
                        Err(e) => view! {
                            <p class="error">"Error loading items: " {e.to_string()}</p>
                        }.into_any(),
                        Ok(list) if list.is_empty() => view! {
                            <div class="empty-state">
                                <p>"This list is empty."</p>
                                <a href=move || format!("/list/{}/add", list_id()) class="btn btn-primary">
                                    "Add your first item"
                                </a>
                            </div>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="item-grid">
                                <For
                                    each=move || list.clone()
                                    key=|item| item.id
                                    children=move |item| {
                                        let lid = list_id();
                                        let ol = other_lists.clone();
                                        view! {
                                            <WishCard
                                                item=item
                                                list_id=lid
                                                other_lists=ol
                                                on_change=move || {
                                                    refetch_items();
                                                    refetch_info();
                                                }
                                            />
                                        }
                                    }
                                />
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn WishCard(
    item: WishItem,
    list_id: i64,
    other_lists: Vec<ItemListSummary>,
    on_change: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let show_buy_modal = RwSignal::new(false);
    let item_id = item.id;
    let edit_href = format!("/list/{}/edit/{}", list_id, item.id);

    // Transfer state
    let move_target = RwSignal::new(other_lists.first().map(|l| l.id).unwrap_or(0));
    let has_other_lists = !other_lists.is_empty();

    let on_delete = {
        let on_change = on_change.clone();
        move |_| {
            let on_change = on_change.clone();
            spawn_local(async move {
                let _ = delete_wish_item(item_id).await;
                on_change();
            });
        }
    };

    let on_move = {
        let on_change = on_change.clone();
        move |_: web_sys::MouseEvent| {
            let target = move_target.get();
            if target == 0 {
                return;
            }
            let on_change = on_change.clone();
            spawn_local(async move {
                let _ = transfer_wish_item(item_id, target).await;
                on_change();
            });
        }
    };

    view! {
        <div class="wish-card">
            <div class="wish-card-header">
                <h3 class="wish-name">{item.name.clone()}</h3>
                <div class="wish-actions">
                    <a href=edit_href class="btn btn-sm">"Edit"</a>
                    <button
                        class="btn btn-sm btn-success"
                        on:click=move |_| show_buy_modal.set(true)
                    >
                        "Mark Bought"
                    </button>
                    <button class="btn btn-sm btn-danger" on:click=on_delete>"Delete"</button>
                </div>
            </div>

            <div class="wish-meta">
                {item.category.clone().map(|c| view! {
                    <span class="badge">{c}</span>
                })}
                {item.estimated_cost.map(|cost| view! {
                    <span class="cost">{format!("{:.2} €", cost)}</span>
                })}
            </div>

            <div class="wish-levels">
                <LevelBar label="Want" value=item.want_level color="want" />
                <LevelBar label="Need" value=item.need_level color="need" />
            </div>

            {item.description.clone().map(|d| view! {
                <p class="wish-description">{d}</p>
            })}

            {item.where_to_buy.clone().map(|w| view! {
                <p class="wish-store">
                    <span class="label">"Where: "</span>
                    {if w.starts_with("http") {
                        view! { <a href=w.clone() target="_blank">{w.clone()}</a> }.into_any()
                    } else {
                        view! { <span>{w}</span> }.into_any()
                    }}
                </p>
            })}

            {has_other_lists.then(move || {
                let lists = other_lists.clone();
                view! {
                    <div class="move-row">
                        <select
                            class="move-select"
                            on:change=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                    move_target.set(v);
                                }
                            }
                        >
                            {lists.iter().map(|l| {
                                let id = l.id;
                                let name = l.name.clone();
                                view! { <option value=id>{name}</option> }
                            }).collect_view()}
                        </select>
                        <button class="btn btn-sm" on:click=on_move>"Move"</button>
                    </div>
                }
            })}

            {move || show_buy_modal.get().then(|| view! {
                <BuyModal
                    item_id=item_id
                    item_name=item.name.clone()
                    estimated_cost=item.estimated_cost
                    on_close=move || show_buy_modal.set(false)
                    on_purchased={
                        let on_change = on_change.clone();
                        move || { show_buy_modal.set(false); on_change(); }
                    }
                />
            })}
        </div>
    }
}

#[component]
fn LevelBar(label: &'static str, value: i64, color: &'static str) -> impl IntoView {
    view! {
        <div class="level-bar">
            <span class="level-label">{label}</span>
            <div class="level-dots">
                {(1..=5i64).map(|i| {
                    let cls = if i <= value {
                        format!("dot dot-filled dot-{color}")
                    } else {
                        "dot".to_string()
                    };
                    view! { <span class=cls></span> }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn BuyModal(
    item_id: i64,
    item_name: String,
    estimated_cost: Option<f64>,
    on_close: impl Fn() + Send + 'static,
    on_purchased: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let cost_str = RwSignal::new(
        estimated_cost
            .map(|c| format!("{:.2}", c))
            .unwrap_or_default(),
    );
    let notes = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);

    let on_submit = {
        let on_purchased = on_purchased.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let actual_cost = cost_str.get().parse::<f64>().ok();
            let notes_val = notes.get();
            let on_purchased = on_purchased.clone();
            let error = error;
            spawn_local(async move {
                match mark_as_purchased(item_id, actual_cost, notes_val).await {
                    Ok(_) => on_purchased(),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        }
    };

    view! {
        <div class="modal-overlay">
            <div class="modal">
                <h3>"Mark as Purchased"</h3>
                <p class="modal-subtitle">{item_name}</p>

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label>"Actual cost (€)"</label>
                        <input
                            type="number"
                            step="0.01"
                            min="0"
                            placeholder="0.00"
                            prop:value=cost_str
                            on:input=move |ev| cost_str.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Notes (optional)"</label>
                        <input
                            type="text"
                            placeholder="e.g. ordered from Amazon"
                            prop:value=notes
                            on:input=move |ev| notes.set(event_target_value(&ev))
                        />
                    </div>

                    {move || error.get().map(|e| view! { <p class="error">{e}</p> })}

                    <div class="modal-actions">
                        <button type="button" class="btn" on:click=move |_| on_close()>
                            "Cancel"
                        </button>
                        <button type="submit" class="btn btn-success">"Confirm Purchase"</button>
                    </div>
                </form>
            </div>
        </div>
    }
}
