Stroke-order data: converted 2026-09-23 for Yap.Town.

KanjiVG, copyright Ulrich Apel (2009–2011) and contributors
https://kanjivg.tagaini.net/
https://github.com/KanjiVG/kanjivg/tree/422b5538595676da918c288a4230cb5e22a1ee7e
License: Creative Commons Attribution-ShareAlike 3.0
https://creativecommons.org/licenses/by-sa/3.0/
Canonical SVG stroke centerlines are flattened to 12 segments per cubic and
scaled by 1/109. Direction and stroke order are retained, including kana.
Derived KanjiVG data remains CC BY-SA 3.0, not the application's code license.

Make Me a Hanzi, copyright its contributors
https://github.com/skishore/makemeahanzi/tree/bddc96d41bef78427ed0e034e9f7e31d71fd1b92
Graphics derived from Arphic PL KaitiM GB and Arphic PL UKai.
AnimCJK, copyright 2016–2026 FM&SH
https://github.com/parsimonhi/animCJK/tree/ec5e17cca76c87587790bcbce5ea0b4d4fb753d6
Traditional Han graphics derived from Arphic fonts and Make Me a Hanzi.
Both graphics sources: Arphic Public License, copyright (C) 1999
Arphic Technology Co., Ltd., https://ftp.gnu.org/non-gnu/chinese-fonts-truetype/LICENSE
Medians are retained in order, converted to (x/1024, (900-y)/1024).
Derived graphics remain under the Arphic Public License, not the application's
code license. Taiwan data overrides PRC fallback; fallback retains its PRC tag.

All sources are filtered to characters used in each course. No source outlines,
numbers, dictionary entries or metadata are redistributed. License texts are
in licenses/.

Scribing korean-textbook, copyright (c) 2026 Scribing contributors
https://github.com/xiaolai/scribing/tree/fbd28a4de6bbeb3ee07ecbb0517d0054b1919791
Source: packs/generated/korean-textbook.json at the pinned commit above.
License: MIT (licenses/Scribing.txt). Original authored geometry; teaching
references inform formation choices, with no source artwork or font outlines.
We retain the default plan's stroke order and normalize coordinates by /1000.
Our syllable-block composition fits these 40 jamo into rectangular regions;
this composition is our own and is not part of the upstream pack.
Upstream describes the pack as a technical preview, not pedagogically certified.

Devanagari, Thai, Latin and Cyrillic are authored in this repository, drawn
in code in src/authored/, 2026-09-23. No published
stroke-order dataset exists for these scripts, so the centerlines are original
geometry drawn against school teaching references: Indian primary handwriting
worksheets for Devanagari (body first, shirorekha last), the Thai Ministry of
Education letterform guide for Thai (start at the head loop), ball-and-stick
print formation for Latin (single-storey a and g, accents after the letter),
and Russian print-letter (печатные буквы) formation for Cyrillic. Hindi
conjuncts are composed by our own code from the authored letters (half forms,
reph, rakar, stacks, anchored vowel signs), following the conjunct rules of
the Cambridge Introduction to Sanskrit primer; the few ligatures with a shape
of their own (क्ष त्र ज्ञ श्र द्ध द्व द्य ह्म त्त) are drawn here too.
Besides the taught form, the packs carry accepted alternative letterforms a
learner may write instead (for example double-storey a and g, open-top 4,
two-storey Cyrillic а, the full-headline अ and the older Uttara झ), drawn
here in the same way, with the published glyph images named in the module
docs as references only; composed letters and aksharas take every
combination of their letters' forms.
The Latin pack's cursive-derived print alternatives (looped ascenders, entry
strokes and exit tails, the French p, ʒ-like z, retraced-stem capitals) were
informed by the school handwriting models documented by Primarium
(https://primarium.info/handwriting-models/, CC BY-SA 4.0): Écriture A and B,
Méthode Dumont, Cuadernos Rubio and Santillana, Porto Editora, Letra
Brasileira, Corsivo tradizionale and Italica. Their images were referenced
only; no geometry was copied.
Scribing's MIT English pack informed the Latin order but no geometry was copied. Noto
fonts were used only as a visual check while drawing; no font outlines are
redistributed. These packs are covered by the application's own license and
have not been reviewed by a native teacher.
