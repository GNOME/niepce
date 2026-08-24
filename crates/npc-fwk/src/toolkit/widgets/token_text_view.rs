/*
 * niepce - fwk/toolkit/widgets/token_text_view.rs
 *
 * Copyright (C) 2022-2026 Hubert Figuière
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

use crate::base::propertyvalue::MixedString;

use crate::glib;
use crate::gtk4;
use gtk4::prelude::*;

glib::wrapper! {
    /// A text view that get receive a list of token.
    ///
    /// Work in progress.
    pub struct TokenTextView(
        ObjectSubclass<imp::TokenTextView>)
        @extends gtk4::TextView, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::ConstraintTarget, gtk4::Buildable, gtk4::Scrollable;
}

impl TokenTextView {
    pub fn new() -> TokenTextView {
        glib::Object::builder::<Self>()
            .property("wrap-mode", gtk4::WrapMode::Word)
            .property("accepts-tab", false)
            .build()
    }

    fn text_to_tokens(text: &str) -> Vec<String> {
        text.split(',')
            .filter_map(|s| {
                if s.ends_with('*') {
                    None
                } else {
                    Some(s.to_string())
                }
            })
            .collect()
    }

    fn text_to_mixed_tokens(text: &str) -> Vec<MixedString> {
        text.split(',')
            .map(|s| {
                if s.ends_with('*') {
                    MixedString::Mix(s.trim_end_matches("*").to_string())
                } else {
                    MixedString::Str(s.to_string())
                }
            })
            .collect()
    }

    /// Get the tokens from the text.
    pub fn tokens(&self) -> Vec<String> {
        let start = self.buffer().start_iter();
        let end = self.buffer().end_iter();
        let text = self.buffer().text(&start, &end, true);
        Self::text_to_tokens(&text)
    }

    /// Get the mixed tokens from the text.
    pub fn mixed_tokens(&self) -> Vec<MixedString> {
        let start = self.buffer().start_iter();
        let end = self.buffer().end_iter();
        let text = self.buffer().text(&start, &end, true);
        Self::text_to_mixed_tokens(&text)
    }

    /// Set the tokens.
    pub fn set_tokens(&self, tokens: &[MixedString]) {
        let text = tokens
            .iter()
            .map(|s| match s {
                MixedString::Str(s) => s.to_string(),
                MixedString::Mix(s) => format!("{s}*"),
            })
            .fold(String::default(), |acc, s| acc + "," + s.as_str());
        self.buffer().set_text(&text);
    }
}

impl Default for TokenTextView {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use crate::glib;
    use crate::gtk4;
    use gtk4::subclass::prelude::*;

    #[derive(Default)]
    pub struct TokenTextView {}

    #[glib::object_subclass]
    impl ObjectSubclass for TokenTextView {
        const NAME: &'static str = "NpcTokenTextView";
        type Type = super::TokenTextView;
        type ParentType = gtk4::TextView;
    }

    impl ObjectImpl for TokenTextView {}
    impl TextViewImpl for TokenTextView {}
    impl WidgetImpl for TokenTextView {}
}

#[cfg(test)]
mod test {
    use super::TokenTextView;
    use crate::MixedString;

    #[test]
    fn test_text_to_tokens() {
        let text = "keyword,image,mix*,deleted*";

        let tokens = TokenTextView::text_to_mixed_tokens(text);
        assert_eq!(tokens.len(), 4);
        assert_eq!(
            tokens,
            vec![
                MixedString::Str("keyword".into()),
                MixedString::Str("image".into()),
                MixedString::Mix("mix".into()),
                MixedString::Mix("deleted".into()),
            ]
        );
        let tokens = TokenTextView::text_to_tokens(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens, vec!["keyword".to_string(), "image".to_string()]);
    }
}
