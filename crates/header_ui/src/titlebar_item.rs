use gpui::{
    actions, div, px, Action, AppContext, Element, InteractiveElement, IntoElement, Model,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, View, ViewContext,
    VisualContext, WeakView, WindowBounds,
};
use project::{Project, RepositoryEntry};
use recent_projects::RecentProjects;
use std::env;
use theme::ActiveTheme;
use ui::{h_flex, popover_menu, prelude::*, Button, ButtonLike, ButtonStyle, ContextMenu, Tooltip};
use util::ResultExt;
use vcs_menu::{build_branch_list, BranchList, OpenRecent as ToggleVcsMenu};
use workspace::{titlebar_height, Workspace};

const MAX_PROJECT_NAME_LENGTH: usize = 40;
const MAX_BRANCH_NAME_LENGTH: usize = 40;

actions!(collab, [ToggleUserMenu, ToggleProjectMenu, SwitchBranch]);

pub fn init(cx: &mut AppContext) {
    cx.observe_new_views(|workspace: &mut Workspace, cx| {
        let titlebar_item = cx.new_view(|cx| TitlebarItem::new(workspace, cx));
        workspace.set_titlebar_item(titlebar_item.into(), cx)
    })
    .detach();
}

pub struct TitlebarItem {
    project: Model<Project>,
    workspace: WeakView<Workspace>,
    _subscriptions: Vec<Subscription>,
}

impl Render for TitlebarItem {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .id("titlebar")
            .justify_between()
            .w_full()
            .h(titlebar_height(cx))
            .map(|this| {
                if matches!(cx.window_bounds(), WindowBounds::Fullscreen) {
                    this.pl_2()
                } else {
                    // Use pixels here instead of a rem-based size because the macOS traffic
                    // lights are a static size, and don't scale with the rest of the UI.
                    this.pl(px(80.))
                }
            })
            .bg(cx.theme().colors().title_bar_background)
            .on_click(|event, cx| {
                if event.up.click_count == 2 {
                    cx.zoom_window();
                }
            })
            // left side
            .child(
                h_flex()
                    .gap_1()
                    .child(self.render_project_name(cx))
                    .children(self.render_project_branch(cx)),
            )
            // right side
            .child(
                h_flex()
                    .gap_1()
                    .pr_1()
                    .child(self.render_user_menu_button()),
            )
    }
}

impl TitlebarItem {
    pub fn new(workspace: &Workspace, cx: &mut ViewContext<Self>) -> Self {
        let project = workspace.project().clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(
            cx.observe(&workspace.weak_handle().upgrade().unwrap(), |_, _, cx| {
                cx.notify()
            }),
        );
        subscriptions.push(cx.observe(&project, |_, _, cx| cx.notify()));

        Self {
            workspace: workspace.weak_handle(),
            project,
            _subscriptions: subscriptions,
        }
    }

    pub fn render_project_name(&self, cx: &mut ViewContext<Self>) -> impl Element {
        let name = {
            let mut names = self.project.read(cx).visible_worktrees(cx).map(|worktree| {
                let worktree = worktree.read(cx);
                worktree.root_name()
            });

            names.next()
        };
        let is_project_selected = name.is_some();
        let name = if let Some(name) = name {
            util::truncate_and_trailoff(name, MAX_PROJECT_NAME_LENGTH)
        } else {
            "Open recent project".to_string()
        };

        let workspace = self.workspace.clone();
        popover_menu("project_name_trigger")
            .trigger(
                Button::new("project_name_trigger", name)
                    .when(!is_project_selected, |b| b.color(Color::Muted))
                    .style(ButtonStyle::Subtle)
                    .label_size(LabelSize::Small)
                    .tooltip(move |cx| Tooltip::text("Recent Projects", cx)),
            )
            .menu(move |cx| Some(Self::render_project_popover(workspace.clone(), cx)))
    }

    pub fn render_project_branch(&self, cx: &mut ViewContext<Self>) -> Option<impl Element> {
        let entry = {
            let mut names_and_branches =
                self.project.read(cx).visible_worktrees(cx).map(|worktree| {
                    let worktree = worktree.read(cx);
                    worktree.root_git_entry()
                });

            names_and_branches.next().flatten()
        };
        let workspace = self.workspace.upgrade()?;
        let branch_name = entry
            .as_ref()
            .and_then(RepositoryEntry::branch)
            .map(|branch| util::truncate_and_trailoff(&branch, MAX_BRANCH_NAME_LENGTH))?;
        Some(
            popover_menu("project_branch_trigger")
                .trigger(
                    Button::new("project_branch_trigger", branch_name)
                        .color(Color::Muted)
                        .style(ButtonStyle::Subtle)
                        .label_size(LabelSize::Small)
                        .tooltip(move |cx| {
                            Tooltip::with_meta(
                                "Recent Branches",
                                Some(&ToggleVcsMenu),
                                "Local branches only",
                                cx,
                            )
                        }),
                )
                .menu(move |cx| Self::render_vcs_popover(workspace.clone(), cx)),
        )
    }

    pub fn render_vcs_popover(
        workspace: View<Workspace>,
        cx: &mut WindowContext<'_>,
    ) -> Option<View<BranchList>> {
        let view = build_branch_list(workspace, cx).log_err()?;
        let focus_handle = view.focus_handle(cx);
        cx.focus(&focus_handle);
        Some(view)
    }

    pub fn render_project_popover(
        workspace: WeakView<Workspace>,
        cx: &mut WindowContext<'_>,
    ) -> View<RecentProjects> {
        let view = RecentProjects::open_popover(workspace, cx);

        let focus_handle = view.focus_handle(cx);
        cx.focus(&focus_handle);
        view
    }

    pub fn render_user_menu_button(&mut self) -> impl Element {
        let user = env::var("USER").unwrap_or("".to_string());
        popover_menu("user-menu")
            .menu(|cx| {
                ContextMenu::build(cx, |menu, _| {
                    menu.action("Settings", zed_actions::OpenSettings.boxed_clone())
                        .action("Extensions", extensions_ui::Extensions.boxed_clone())
                        .action("Themes", theme_selector::Toggle.boxed_clone())
                })
                .into()
            })
            .trigger(
                ButtonLike::new("user-menu")
                    .child(
                        div().mx_2().child(
                            Label::new(format!("what's up, {}", user)).size(LabelSize::Small),
                        ),
                    )
                    .style(ButtonStyle::Subtle)
                    .tooltip(move |cx| Tooltip::text("Config", cx)),
            )
    }
}
