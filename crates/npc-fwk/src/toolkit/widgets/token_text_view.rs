/*
 * niepce - fwk/toolkit/widgets/token_text_view.rs
 *
 * Copyright (C) 2022 Hubert Figuière
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use gtk4::prelude::*;

glib::wrapper! {
    /// A text view that get receive a list of token.
    ///
    /// Work in progress.
    pub struct TokenTextView(
        ObjectSubclass<imp::TokenTextView>)
        @extends gtk4::TextView, gtk4::Widget;
}

impl TokenTextView {
    pub fn new() -> TokenTextView {
        glib::Object::new(&[
            ("wrap-mode", &gtk4::WrapMode::Word),
            ("accepts-tab", &false),
        ])
        .expect("Failed to create TokenTextView Widget")
    }

    /// Get the tokens from the text.
    pub fn tokens(&self) -> Vec<String> {
        let start = self.buffer().start_iter();
        let end = self.buffer().end_iter();
        let text = self.buffer().text(&start, &end, true);
        text.split(',').map(|s| s.to_string()).collect()
    }

    /// Set tht tokens.
    pub fn set_tokens(&self, tokens: &[String]) {
        let text = tokens.join(",");
        self.buffer().set_text(&text);
    }
}

impl Default for TokenTextView {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use gtk4::subclass::prelude::*;

    pub struct TokenTextView {}

    impl ObjectImpl for TokenTextView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TokenTextView {
        const NAME: &'static str = "NpcTokenTextView";
        type Type = super::TokenTextView;
        type ParentType = gtk4::TextView;

        fn new() -> Self {
            Self {}
        }
    }

    impl TextViewImpl for TokenTextView {}
    impl WidgetImpl for TokenTextView {}
}
