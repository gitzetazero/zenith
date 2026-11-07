//! The Addons screen while a world is paused: drawn over the blurred world, opened
//! from the Options tab bar, with the same registry as the title's screen.
use super::{Game, Screen};
use zenith_client::{
    hud::HudBatch,
    text::Text,
    top_bar::{self, Tab},
    ui,
};

impl Game<'_> {
    pub(super) fn addons_dir(&self) -> std::path::PathBuf {
        let path = Self::options_path();
        path.parent()
            .unwrap_or(std::path::Path::new("."))
            .to_owned()
    }
    pub(super) fn addons_account(&self) -> String {
        zenith_client::auth::current()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| self.options.player_name())
    }
    pub(super) fn draw_addons(&mut self, batch: &mut HudBatch) {
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, &self.gui.atlas, 0.0, 0.0, w, h, self.title.is_none());
        let dir = self.addons_dir();
        zenith_client::addons_ui::draw(batch, &self.gui.atlas, &self.addons, &dir, self.cursor);
        let back = [(w / 2.0).floor() - 75.0, h - 28.0, 150.0, 20.0];
        let hover = ui::hovered(self.cursor, back[0], back[1], back[2], back[3]);
        ui::button(
            batch,
            &self.gui.atlas,
            back[0],
            back[1],
            back[2],
            back[3],
            &Text::translate("gui.back", &[]),
            true,
            hover,
        );
        let account = self.addons_account();
        top_bar::draw(batch, &self.gui.atlas, Tab::Addons, &account, self.cursor);
    }
    pub(super) fn addons_click(&mut self) {
        let (w, h) = self.gui_size();
        if let Some(tab) = top_bar::hit(w, self.cursor.0, self.cursor.1) {
            self.addons_tab(tab);
            return;
        }
        let dir = self.addons_dir();
        match zenith_client::addons_ui::click(&mut self.addons, &dir, self.cursor, (w, h)) {
            zenith_client::addons_ui::Outcome::None => {}
            _ => return,
        }
        let back = [(w / 2.0).floor() - 75.0, h - 28.0, 150.0, 20.0];
        if ui::hovered(self.cursor, back[0], back[1], back[2], back[3]) {
            self.close_addons();
        }
    }
    pub(super) fn addons_tab(&mut self, tab: Tab) {
        match tab {
            Tab::Addons => {}
            Tab::Options => self.close_addons(),
            Tab::Singleplayer | Tab::Multiplayer => {
                if self.title.is_some() {
                    if let Some(title) = &mut self.title {
                        title.open_tab(tab);
                    }
                    self.screen = Screen::Playing;
                }
            }
        }
    }
    pub(super) fn close_addons(&mut self) {
        self.screen = Screen::Options;
    }
}
