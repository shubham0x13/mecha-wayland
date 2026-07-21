use crate::backend::{FileChooserOutcome, FileChooserResponse, RequestHandle};
use assets::BakedFont;
use taffy::prelude::*;
use ui::widgets::{Div, Text};
use ui::{EventCtx, Point, RenderCommand, Widget, WidgetList, WidgetTree};
use utils::Color;

use super::ChooserOptions;
use portal_core::atlas;
use portal_core::widgets::Button;

// --- Widget tree type aliases ------------------------------------------------
pub type ButtonRow = Div<(Button, Button)>;
pub type ModalDiv = Div<(Text, Text, ButtonRow)>;
pub type RootDiv = Div<(ModalDiv,)>;

// Struct representing the file chooser dialog interface (rendered inside a top-level XDG window).
pub struct FileChooserUi {
    handle: RequestHandle,
    #[allow(dead_code)]
    options: ChooserOptions,
    root: RootDiv,
    choose_rect: utils::Rect,
    cancel_rect: utils::Rect,
}

impl FileChooserUi {
    pub fn new(handle: RequestHandle, options: ChooserOptions) -> Self {
        let root = make_root(
            &atlas::UI_FONT_INTER_24,
            &atlas::UI_FONT_INTER_16,
            &handle,
            &options,
        );
        Self {
            handle,
            options,
            root,
            choose_rect: utils::Rect::ZERO,
            cancel_rect: utils::Rect::ZERO,
        }
    }
}

impl WidgetList for FileChooserUi {
    fn build_children(&mut self, tree: &mut WidgetTree) -> Vec<taffy::NodeId> {
        vec![self.root.build_tree(tree)]
    }

    fn render_children(&mut self, tree: &WidgetTree, parent_abs: Point) -> Vec<RenderCommand> {
        let commands = self.root.render_children(tree, parent_abs);
        let choose_id: u64 = self.root.children.0.children.2.children.0.node_id().into();
        let cancel_id: u64 = self.root.children.0.children.2.children.1.node_id().into();
        for cmd in &commands {
            if let RenderCommand::RegisterHitArea { id, rect } = cmd {
                if *id == choose_id {
                    self.choose_rect = *rect;
                } else if *id == cancel_id {
                    self.cancel_rect = *rect;
                }
            }
        }
        commands
    }

    fn on_event(&mut self, ctx: &mut EventCtx) {
        if self.choose_rect != utils::Rect::ZERO && ctx.interactivity().is_clicked(self.choose_rect) {
            println!("[ui] Choose File clicked.");
            ctx.dispatch(FileChooserResponse {
                handle: self.handle.clone(),
                outcome: FileChooserOutcome::Selected(vec![
                    "file:///home/shubham/Desktop/selected_file.txt".to_string(),
                ]),
            });
        } else if self.cancel_rect != utils::Rect::ZERO
            && ctx.interactivity().is_clicked(self.cancel_rect)
        {
            println!("[ui] Cancel clicked.");
            ctx.dispatch(FileChooserResponse {
                handle: self.handle.clone(),
                outcome: FileChooserOutcome::Cancelled,
            });
        }
    }

    fn wants_input(&self) -> bool {
        true
    }
}

// --- Layout ------------------------------------------------------------------
fn make_root(
    font_24: &'static BakedFont,
    font_16: &'static BakedFont,
    handle: &str,
    options: &ChooserOptions,
) -> RootDiv {
    let title_str = match options {
        ChooserOptions::OpenFile(opt) => {
            if opt.multiple.unwrap_or(false) {
                "Select Files"
            } else {
                "Select File"
            }
        }
        ChooserOptions::SaveFile(_) => "Save File",
        ChooserOptions::SaveFiles(_) => "Save Files",
    };

    let accept_str = match options {
        ChooserOptions::OpenFile(opt) => opt.accept_label.clone().unwrap_or_else(|| {
            if opt.multiple.unwrap_or(false) {
                "Choose Files".to_string()
            } else {
                "Choose File".to_string()
            }
        }),
        ChooserOptions::SaveFile(opt) => opt
            .accept_label
            .clone()
            .unwrap_or_else(|| "Save".to_string()),
        ChooserOptions::SaveFiles(opt) => opt
            .accept_label
            .clone()
            .unwrap_or_else(|| "Save All".to_string()),
    };

    let mut title = Text::new(Style::default());
    title.font = Some(font_24);
    title.text = title_str.to_string();
    title.color = Color::WHITE;
    title.z = 0.95;

    let mut handle_text = Text::new(Style::default());
    handle_text.font = Some(font_16);
    let handle_str = format!("Request: {}", handle);
    handle_text.text = if handle_str.len() > 40 {
        format!("{}...", &handle_str[..37])
    } else {
        handle_str
    };
    handle_text.color = Color::rgb(0.6, 0.6, 0.7);
    handle_text.z = 0.95;

    let mut choose_btn = Button::new(&accept_str);
    choose_btn.div.color = Color::rgb(0.2, 0.45, 0.9);
    choose_btn.div.border_color = Color::rgb(0.35, 0.6, 1.0);
    choose_btn.div.border_radius = 10.0;
    choose_btn.div.border_thickness = 1.5;
    choose_btn.div.z = 1.0;
    choose_btn.div.children.font = Some(font_16);
    choose_btn.div.children.color = Color::WHITE;
    choose_btn.div.children.z = 0.5;

    let mut cancel_btn = Button::new("Cancel");
    cancel_btn.div.color = Color::rgb(0.25, 0.25, 0.3);
    cancel_btn.div.border_color = Color::rgb(0.35, 0.35, 0.4);
    cancel_btn.div.border_radius = 10.0;
    cancel_btn.div.border_thickness = 1.5;
    cancel_btn.div.z = 1.0;
    cancel_btn.div.children.font = Some(font_16);
    cancel_btn.div.children.color = Color::WHITE;
    cancel_btn.div.children.z = 0.5;

    let row_style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        size: Size {
            width: percent(1.0_f32),
            height: length(50.0_f32),
        },
        justify_content: Some(JustifyContent::SpaceBetween),
        align_items: Some(AlignItems::Center),
        ..Default::default()
    };
    let row = Div::new(row_style, (choose_btn, cancel_btn));

    let modal_style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: Some(JustifyContent::SpaceBetween),
        align_items: Some(AlignItems::Center),
        size: Size {
            width: length(460.0_f32),
            height: length(240.0_f32),
        },
        padding: taffy::Rect {
            left: length(40.0_f32),
            right: length(40.0_f32),
            top: length(40.0_f32),
            bottom: length(40.0_f32),
        },
        ..Default::default()
    };
    let mut modal = Div::new(modal_style, (title, handle_text, row));
    modal.color = Color::rgb(0.12, 0.12, 0.15);
    modal.border_color = Color::rgb(0.25, 0.25, 0.3);
    modal.border_radius = 16.0;
    modal.border_thickness = 2.0;
    modal.z = 0.2;

    let root_style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        size: Size {
            width: percent(1.0_f32),
            height: percent(1.0_f32),
        },
        ..Default::default()
    };
    let mut root = Div::new(root_style, (modal,));
    root.color = Color::rgba(0.0, 0.0, 0.0, 0.6);
    root.z = 0.1;

    root
}
