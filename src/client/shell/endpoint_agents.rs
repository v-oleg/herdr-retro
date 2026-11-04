use super::render::put_text;
use super::*;

pub(super) fn render_collapsed(
    buffer: &mut Buffer,
    area: Rect,
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    config: &ClientShellConfig,
    hits: &mut ShellHitMap,
) {
    let rows = agent_rows(endpoints, active_endpoint_id, config);
    for (index, row) in rows.into_iter().take(area.height as usize).enumerate() {
        let rect = Rect::new(area.x, area.y + index as u16, area.width, 1);
        if row.agent.focused {
            buffer.set_style(rect, Style::default().bg(config.palette.active_row_bg));
        }
        let initial = row.machine_label.chars().next().unwrap_or('?');
        put_text(
            buffer,
            rect.x,
            rect.y,
            rect.width,
            &format!(
                "{initial}{}",
                status_icon(row.agent.status, config.status_indicators)
            ),
            Style::default()
                .fg(if row.stale {
                    config.palette.overlay0
                } else {
                    status_color(row.agent.status, &config.palette)
                })
                .add_modifier(if row.stale {
                    Modifier::DIM
                } else {
                    Modifier::empty()
                }),
        );
        hits.endpoint_agents
            .push((rect, row.endpoint_id, row.agent.pane_id));
    }
}

pub(super) fn render_expanded(
    buffer: &mut Buffer,
    area: Rect,
    agent_view_label: Option<&str>,
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    config: &ClientShellConfig,
    agent_scroll: &mut usize,
    hits: &mut ShellHitMap,
) {
    if !super::agent_sidebar::render_agent_panel_header(
        buffer,
        area,
        agent_view_label,
        config,
        hits,
    ) {
        return;
    }
    let rows = agent_rows(endpoints, active_endpoint_id, config);
    super::agent_sidebar::render_agent_list(
        buffer,
        area,
        &rows,
        agent_view_label.map(|_| " no matching agents"),
        config,
        agent_scroll,
        hits,
        |row| {
            if config.retro_sidebar {
                usize::from(super::retro::card_height(area.width))
                    + usize::from(row.group_counts.is_some())
            } else {
                row.agent.rows.len()
            }
        },
        |buffer, rect, row, hits| {
            let mut rect = rect;
            if config.retro_sidebar {
                let scene = super::retro::scene_for(row.endpoint_index);
                if let Some((busy, total)) = row.group_counts {
                    let header = Rect::new(rect.x, rect.y, rect.width, 1);
                    super::retro::render_group_header(
                        buffer,
                        header,
                        &row.machine_label,
                        scene,
                        busy,
                        total,
                    );
                    rect.y += 1;
                    rect.height = rect.height.saturating_sub(1);
                }
                let key = format!("{}/{}", row.agent.title, row.agent.pane_id);
                super::retro::render_card(buffer, rect, &row.agent, scene, &key);
            } else {
                super::agent_sidebar::render_agent_row(buffer, rect, &row.agent, config);
            }
            if row.stale {
                buffer.set_style(
                    rect,
                    Style::default()
                        .fg(config.palette.overlay0)
                        .add_modifier(Modifier::DIM),
                );
            }
            hits.endpoint_agents
                .push((rect, row.endpoint_id.clone(), row.agent.pane_id.clone()));
        },
    );
}

struct EndpointAgentRow {
    endpoint_id: ClientEndpointId,
    machine_label: String,
    stale: bool,
    endpoint_index: usize,
    /// Set on the first row of each run of agents from one machine.
    group_counts: Option<(usize, usize)>,
    agent: super::agent_sidebar::AgentRow,
}

fn agent_rows(
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    config: &ClientShellConfig,
) -> Vec<EndpointAgentRow> {
    let mut rendered_rows = endpoints
        .iter()
        .filter_map(|endpoint| {
            endpoint.snapshot.as_deref().map(|snapshot| {
                snapshot
                    .agents
                    .iter()
                    .filter_map(|agent| {
                        super::agent_sidebar::agent_row(
                            snapshot,
                            &agent.pane_id,
                            config,
                            Some(&endpoint.label),
                        )
                    })
                    .map(|agent| ((endpoint.endpoint_id.clone(), agent.pane_id.clone()), agent))
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect::<HashMap<_, _>>();

    let mut rows = super::aggregate_navigation::aggregate_agent_rows(
        endpoints,
        active_endpoint_id,
        config.agent_panel_sort,
    )
    .into_iter()
    .filter_map(|row| {
        let key = (row.endpoint.endpoint_id.clone(), row.agent.pane_id.clone());
        let mut agent = rendered_rows.remove(&key)?;
        agent.focused &= row.endpoint.endpoint_id == active_endpoint_id;
        Some(EndpointAgentRow {
            endpoint_id: row.endpoint.endpoint_id.clone(),
            machine_label: row.endpoint.label.to_owned(),
            stale: row.endpoint.stale(),
            endpoint_index: endpoints
                .iter()
                .position(|endpoint| &endpoint.endpoint_id == row.endpoint.endpoint_id)
                .unwrap_or(0),
            group_counts: None,
            agent,
        })
    })
    .collect::<Vec<_>>();

    let mut start = 0;
    while start < rows.len() {
        let endpoint_id = rows[start].endpoint_id.clone();
        let run = rows[start..]
            .iter()
            .take_while(|row| row.endpoint_id == endpoint_id)
            .collect::<Vec<_>>();
        let total = run.len();
        let busy = run
            .iter()
            .filter(|row| row.agent.status == crate::api::schema::AgentStatus::Working)
            .count();
        rows[start].group_counts = Some((busy, total));
        start += total;
    }
    rows
}
