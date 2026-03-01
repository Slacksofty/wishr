use crate::server::items::*;
use leptos::prelude::*;

#[component]
pub fn Archive() -> impl IntoView {
    let archive = Resource::new(|| (), |_| get_archive());
    let stats = Resource::new(|| (), |_| get_stats());

    view! {
        <div class="page">
            <div class="page-header">
                <h1>"Purchase Archive"</h1>
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
                                    <span class="stat-label">"Active Items"</span>
                                    <span class="stat-value">{s.active_count}</span>
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
                                <p>"No purchases yet. Mark items as bought from the wishlist."</p>
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
