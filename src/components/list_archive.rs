use crate::models::ItemListSummary;
use crate::server::items::*;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ListArchive() -> impl IntoView {
    let params = use_params_map();
    let list_id = move || {
        params.with(|p| {
            p.get("list_id")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };

    let list_info = Resource::new(move || list_id(), |id| get_item_list_detail(id));
    let archive = Resource::new(move || list_id(), |id| get_archive_for_list(id));
    let stats = Resource::new(move || list_id(), |id| get_list_stats(id));

    view! {
        <div class="page">
            <div class="page-header">
                <div class="page-title-group">
                    <a href=move || format!("/list/{}", list_id()) class="back-link">"← List"</a>
                    <h1>
                        <Suspense fallback=|| "…">
                            {move || list_info.get().map(|r| {
                                let name = r.unwrap_or(ItemListSummary {
                                    id: 0, name: "Unknown".into(), created_at: String::new(),
                                    active_count: 0, estimated_budget: 0.0,
                                }).name;
                                format!("{} — Archive", name)
                            })}
                        </Suspense>
                    </h1>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="loading">"Loading archive…"</div> }>
                {move || {
                    let stats_view = match stats.get() {
                        None => view! { <div></div> }.into_any(),
                        Some(Err(e)) => view! {
                            <p class="error">"Stats error: " {e.to_string()}</p>
                        }.into_any(),
                        Some(Ok(s)) => view! {
                            <div class="stats-bar">
                                <div class="stat">
                                    <span class="stat-label">"Total Spent"</span>
                                    <span class="stat-value spent">{format!("{:.2} €", s.total_spent)}</span>
                                </div>
                                <div class="stat">
                                    <span class="stat-label">"Purchased"</span>
                                    <span class="stat-value">{s.purchased_count}</span>
                                </div>
                            </div>
                        }.into_any(),
                    };

                    let archive_view = match archive.get() {
                        None => view! { <div class="loading">"Loading items…"</div> }.into_any(),
                        Some(Err(e)) => view! {
                            <p class="error">"Error loading archive: " {e.to_string()}</p>
                        }.into_any(),
                        Some(Ok(items)) if items.is_empty() => view! {
                            <div class="empty-state">
                                <p>"No purchased items in this list yet."</p>
                            </div>
                        }.into_any(),
                        Some(Ok(items)) => view! {
                            <div class="archive-table-wrapper">
                                <table class="archive-table">
                                    <thead>
                                        <tr>
                                            <th>"Item"</th>
                                            <th>"Category"</th>
                                            <th>"Est. Cost"</th>
                                            <th>"Paid"</th>
                                            <th>"Purchased"</th>
                                            <th>"Notes"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || items.clone()
                                            key=|a| a.record.id
                                            children=|archived| {
                                                let item = archived.item;
                                                let record = archived.record;
                                                let date = record.purchased_at
                                                    .split('T').next()
                                                    .unwrap_or(&record.purchased_at)
                                                    .to_string();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <strong>{item.name}</strong>
                                                            {item.description.map(|d| view! {
                                                                <br/><small>{d}</small>
                                                            })}
                                                        </td>
                                                        <td>{item.category.unwrap_or_default()}</td>
                                                        <td class="number">
                                                            {item.estimated_cost
                                                                .map(|c| format!("{:.2} €", c))
                                                                .unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td class="number">
                                                            {record.actual_cost
                                                                .map(|c| format!("{:.2} €", c))
                                                                .unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td>{date}</td>
                                                        <td class="notes">{record.notes.unwrap_or_default()}</td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            </div>
                        }.into_any(),
                    };

                    view! {
                        <div>
                            {stats_view}
                            {archive_view}
                        </div>
                    }
                }}
            </Suspense>
        </div>
    }
}
