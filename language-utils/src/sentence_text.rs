//! The sentence text column, streamed from compressed symbols.
use std::ops::ControlFlow;

use crate::{
    Gram, Language, SpurReadingGram,
    language_pack::SentenceText,
    sentence_codec::{GramKey, Symbol, SymbolKind},
};
use lasso::{Projection, ProjectionHasher, RodeoReader, Spur};

/// Borrowed vocabulary behind a sentence's symbols. Never retained by the reader.
pub struct RenderContext<'a> {
    pub grams: &'a RodeoReader<SpurReadingGram, GramKey>,
    pub gram_rodeo: &'a RodeoReader<Gram<Spur>>,
    pub strings: &'a RodeoReader,
    pub language: Language,
}

/// One word of lookahead for whitespace prediction; no sentence-sized allocation.
/// Phrase annotations do not contribute text and are never decoded.
pub(crate) fn render(
    cx: &RenderContext<'_>,
    row: impl Iterator<Item = impl Into<Symbol>>,
    mut emit: impl FnMut(&str) -> ControlFlow<()>,
) -> ControlFlow<()> {
    let mut symbols = row.map(Into::into).peekable();
    let capitalize = !symbols
        .peek()
        .is_some_and(|symbol| symbol.kind() == SymbolKind::Lowercase);
    if !capitalize {
        symbols.next();
    }
    let atoms = symbols
        .take_while(|symbol| matches!(symbol.kind(), SymbolKind::Gram(_)))
        .enumerate()
        .flat_map(|(index, symbol)| {
            cx.gram_rodeo
                .resolve(&cx.grams.resolve(&symbol.key()).sense.gram)
                .iter()
                .map(move |atom| (index, atom.resolve(cx.strings)))
        });
    for (_, word, whitespace) in crate::render_words(atoms, capitalize, cx.language) {
        emit(&word.text)?;
        emit(whitespace.as_str())?;
    }
    ControlFlow::Continue(())
}

impl<A: lasso::SymbolTable + 'static> Projection<A> for SentenceText
where
    for<'a> A::Item<'a>: Into<Symbol>,
{
    type Context<'a> = RenderContext<'a>;
    type Query = str;

    fn hash_row<'a>(
        cx: &RenderContext<'_>,
        row: impl Iterator<Item = A::Item<'a>>,
        h: &mut ProjectionHasher,
    ) {
        let _ = render(cx, row, |piece| {
            h.write(piece.as_bytes());
            ControlFlow::Continue(())
        });
    }

    fn hash_query(q: &str, h: &mut ProjectionHasher) {
        h.write(q.as_bytes());
    }

    fn matches<'a>(
        cx: &RenderContext<'_>,
        row: impl Iterator<Item = A::Item<'a>>,
        mut q: &str,
    ) -> bool {
        render(cx, row, |piece| match q.strip_prefix(piece) {
            Some(rest) => {
                q = rest;
                ControlFlow::Continue(())
            }
            None => ControlFlow::Break(()),
        })
        .is_continue()
            && q.is_empty()
    }
}
