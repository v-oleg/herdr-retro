//! A navigate mode for agent cards, the counterpart of `prefix+w` for spaces.

use super::aggregate_navigation::{online_agent_targets, AggregateAgentTarget};
use super::*;

impl ClientShellState {
    pub(super) fn agent_navigation_active(&self) -> bool {
        self.mode == ClientShellMode::Navigate && self.navigate_agent.is_some()
    }

    fn agent_navigation_targets(&self) -> Vec<AggregateAgentTarget> {
        online_agent_targets(
            &self.endpoints,
            &self.active_endpoint_id,
            self.config.agent_panel_sort,
        )
    }

    pub(super) fn open_agent_navigation(&mut self) {
        let targets = self.agent_navigation_targets();
        let focused_pane = self.focused_pane_id();
        let Some(target) = targets
            .iter()
            .find(|target| {
                target.endpoint_id == self.active_endpoint_id
                    && focused_pane.as_deref() == Some(target.pane_id.as_str())
            })
            .or(targets.first())
            .cloned()
        else {
            self.mode = self.copy_or_terminal_mode();
            return;
        };
        self.navigate_workspace_id = self.focused_navigation_target();
        self.navigate_agent = Some(target);
        self.mode = ClientShellMode::Navigate;
    }

    fn move_navigate_agent(&mut self, delta: isize) {
        let targets = self.agent_navigation_targets();
        if targets.is_empty() {
            self.navigate_agent = None;
            self.mode = self.copy_or_terminal_mode();
            return;
        }
        let next = match targets
            .iter()
            .position(|target| Some(target) == self.navigate_agent.as_ref())
        {
            Some(current) => (current as isize + delta).rem_euclid(targets.len() as isize) as usize,
            None => 0,
        };
        self.navigate_agent = Some(targets[next].clone());
    }

    fn accept_navigate_agent(&mut self, outcome: &mut ClientShellInput) {
        let Some(target) = self.navigate_agent.clone() else {
            return;
        };
        if !self.agent_navigation_targets().contains(&target) {
            self.move_navigate_agent(0);
            return;
        }
        if self.focus_or_activate(
            target.endpoint_id,
            ClientEndpointFocusTarget::Pane(target.pane_id),
            outcome,
        ) {
            self.mode = ClientShellMode::Terminal;
            self.navigate_workspace_id = None;
            self.navigate_agent = None;
        }
    }

    pub(super) fn route_agent_navigation_key(
        &mut self,
        key: &crate::input::TerminalKey,
        outcome: &mut ClientShellInput,
    ) {
        let navigate = &self.config.keybinds.keybinds.navigate;
        if navigate.workspace_up.matches_direct_key(key) {
            self.move_navigate_agent(-1);
        } else if navigate.workspace_down.matches_direct_key(key) {
            self.move_navigate_agent(1);
        } else {
            let (code, modifiers) = crate::config::normalize_key_combo((key.code, key.modifiers));
            if code == KeyCode::Enter && modifiers.is_empty() {
                self.accept_navigate_agent(outcome);
            }
        }
        outcome.repaint = true;
    }
}
