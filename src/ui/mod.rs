use crate::app::{App, Tab};
use crate::config::Config;
use crate::services::actions::ActionsService;
use crate::services::cluster::ClusterService;
use crate::services::logs::LogsService;
use crate::services::overview::{OverviewData, OverviewService};
use ratatui::backend::Backend;
use ratatui::Frame;

mod layout;
mod render;

pub struct UI {
    pub overview_service: OverviewService,
    pub cluster_service: ClusterService,
    pub logs_service: LogsService,
    pub actions_service: ActionsService,
    pub config: Config,
}

impl UI {
    pub fn new(overview_service: OverviewService, cluster_service: ClusterService, logs_service: LogsService, config: Config) -> Self {
        // Create actions service using the same Patroni client as cluster service
        let actions_service = ActionsService::new(cluster_service.patroni_client.clone());

        UI {
            overview_service,
            cluster_service,
            logs_service,
            actions_service,
            config,
        }
    }

    pub fn draw_ui<B: Backend>(&self, frame: &mut Frame, app: &App) {
        let chunks = layout::create_main_layout(frame.size());

        // Render tabs
        let tabs = render::create_tabs(&app.current_tab);
        frame.render_widget(tabs, chunks[0]);

        // Render content based on selected tab
        match app.current_tab {
            Tab::Overview => self.draw_overview::<B>(frame, chunks[1]),
            Tab::Cluster => self.draw_cluster::<B>(frame, chunks[1]),
            Tab::Logs => self.draw_logs::<B>(
                frame,
                chunks[1],
                app.log_selected,
                app.log_scroll,
                app.log_focus_right,
            ),
            Tab::Actions => self.draw_actions::<B>(frame, chunks[1], app),
        }
    }

    fn draw_overview<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let data: OverviewData = self.overview_service.get_overview();
        render::draw_overview(frame, area, &data, &self.overview_service, &self.config);
    }

    fn draw_cluster<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let data = self.cluster_service.get_cluster_info();
        render::draw_cluster(frame, area, &data);
    }

    fn draw_logs<B: Backend>(
        &self,
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        selected: usize,
        scroll: u16,
        focus_right: bool,
    ) {
        let services = self.config.services_list();
        let selected_service = if selected < services.len() {
            &services[selected]
        } else {
            "unknown"
        };
        let lines = self.logs_service.read_logs(selected_service, 100);

        render::draw_logs(frame, area, &services, selected, scroll, focus_right, &lines, selected_service);
    }

    fn draw_actions<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
        let cluster_info = self.actions_service.get_cluster_info();
        render::draw_actions(
            frame, 
            area, 
            &self.actions_service,
            app.action_selected,
            app.action_confirmation,
            app.action_confirmation_yes,
            &app.action_target_node,
            &app.action_error,
            &cluster_info
        );
    }
}
