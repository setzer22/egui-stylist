//! This contains all the views that are used to construct the core of the application.
use std::path::PathBuf;

use egui::{CentralPanel, Checkbox, FontDefinitions, ScrollArea, SidePanel, Style, Ui, Widget};
use egui_theme::EguiTheme;
use serde::{Deserialize, Serialize};
mod colors;
mod fonts;
mod preview;
mod shape;
mod spacing;

use preview::Preview;

use fonts::FontViewState;

/// StylistFileDialogFunction is a function callback that allows the `StylistState` to open a native filedialog and get file paths for egui.
pub type StylistFileDialogFunction =
    Box<dyn Fn(StylistFileDialog, Option<(&str, &[&str])>) -> Option<PathBuf>>;

/// This determines what kind of FileDialog is desired from within the `StylistState`
pub enum StylistFileDialog {
    Open,
    Save,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Copy)]
enum StylerTab {
    Colors,
    Fonts,
    Spacing,
    Shape,
}
/// This is the framework agnostic application state that can be easily embedded directly into any `egui` integration.
///
/// This can easily be embedded into any existing egui application by calling `ui` from within the egui context such as follows:
///
/// ```ignore
/// let state = StylistState::default():
/// egui::CentralPanel::default().show(ctx, |ui| state.ui(ui));
/// ```

#[derive(Serialize, Deserialize)]
pub struct StylistState {
    current_tab: StylerTab,
    show_stylist: bool,
    show_preview: bool,
    // This is a workaround to fonts being set only in the UI
    #[serde(skip)]
    preview_fonts: bool,
    style: Style,
    font_definitions: FontDefinitions,
    #[serde(skip)]
    font_view_state: FontViewState,
    preview: Preview,
    #[serde(skip)]
    pub file_dialog_function: Option<StylistFileDialogFunction>,
}

impl StylistState {
    pub fn default() -> Self {
        Self {
            current_tab: StylerTab::Colors,
            style: Style::default(),
            show_stylist: true,
            show_preview: true,
            preview_fonts: false,
            font_definitions: FontDefinitions::default(),
            font_view_state: FontViewState::default(),
            preview: Preview::new(Style::default()),
            file_dialog_function: None,
        }
    }
    /// Sets `file_dialog_function` with the function call that it can use to
    pub fn set_file_dialog_function(&mut self, f: StylistFileDialogFunction) {
        self.file_dialog_function = Some(f);
    }
    /// Calls the file_dialog function and returns a path if it was found.
    pub fn file_dialog(
        &self,
        kind: StylistFileDialog,
        filter: Option<(&str, &[&str])>,
    ) -> Option<PathBuf> {
        self.file_dialog_function
            .as_ref()
            .map(|f| f(kind, filter))
            .flatten()
    }

    fn tab_menu_ui(&mut self, ui: &mut Ui) {
        use egui::widgets::SelectableLabel;
        // Menu tabs
        ui.horizontal(|ui| {
            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Colors,
                    "Colors",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Colors;
            }
            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Fonts,
                    "Fonts",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Fonts;
            }
            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Spacing,
                    "Spacing",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Spacing;
            }

            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Shape,
                    "Shape",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Shape;
            }
            Checkbox::new(&mut self.show_stylist, "Show Stylist").ui(ui);
            Checkbox::new(&mut self.show_preview, "Show preview").ui(ui);
            if Checkbox::new(&mut self.preview_fonts, "Preview Font Settings")
                .ui(ui)
                .clicked()
            {
                if self.preview_fonts {
                    ui.ctx().set_fonts(self.font_definitions.clone());
                    ui.ctx()
                        .set_pixels_per_point(self.font_view_state.pixels_per_point);
                } else {
                    ui.ctx().set_fonts(FontDefinitions::default());
                    ui.ctx().set_pixels_per_point(1f32);
                }
            }
            if ui.button("Reset").clicked() {
                ui.reset_style();
                ui.ctx().set_pixels_per_point(1f32);
                ui.ctx().set_fonts(FontDefinitions::default());
            }
        });
    }
    /// Creates and displays the Stylist UI.
    /// This can be used to embed the Stylist into any application that supports it.
    pub fn ui(&mut self, ui: &mut Ui) {
        // Get the tab ui
        self.tab_menu_ui(ui);
        if self.show_stylist {
            SidePanel::left("_stylist_panel")
                .width_range(300.0..=900.0)
                .show_inside(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        // Show the content views.
                        match self.current_tab {
                            StylerTab::Colors => colors::colors_view(&mut self.style, ui),
                            StylerTab::Fonts => fonts::fonts_view(
                                &mut self.font_view_state,
                                self.file_dialog_function.as_ref(),
                                &mut self.font_definitions,
                                &mut self.style,
                                ui,
                            ),
                            StylerTab::Spacing => spacing::spacing_view(&mut self.style, ui),
                            StylerTab::Shape => shape::shape_view(&mut self.style, ui),
                        };
                    });
                });
        }
        if self.show_preview {
            CentralPanel::default().show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.preview.set_style(self.style.clone());
                    self.preview.show(ui);
                });
            });
        }
    }
    pub fn export_theme(&self) -> EguiTheme {
        EguiTheme::new(self.style.clone(), self.font_definitions.clone())
    }
    pub fn import_theme(&mut self, theme: EguiTheme) {
        let (style, font_definitions) = theme.extract();
        self.style = style;
        self.font_definitions = font_definitions;
    }
}
