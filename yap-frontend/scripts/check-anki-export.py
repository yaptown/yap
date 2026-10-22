"""Validate APKG fixtures produced by sibling check-anki-export.mjs using official Anki.

The Node harness supplies a temporary fixture directory as the sole argument.
No account, backend, or existing Anki profile is used.
"""

import hashlib
import html
import json
import re
import sqlite3
import sys
import tempfile
import zipfile
from html.parser import HTMLParser
from pathlib import Path

from anki.collection import Collection
from anki.import_export_pb2 import ImportAnkiPackageOptions, ImportAnkiPackageRequest


class Tags(HTMLParser):
    def __init__(self, markup):
        super().__init__()
        self.tags = []
        self.feed(markup)

    def handle_starttag(self, tag, attrs):
        self.tags.append((tag, dict(attrs)))


def assert_safe(markup):
    for tag, attrs in Tags(markup).tags:
        assert tag != "script", (tag, attrs)
        assert not any(key.startswith("on") for key in attrs), (tag, attrs)
        assert attrs.get("src") != "x", (tag, attrs)


def captions(key):
    return {
        "subs-clip": '<span class="sentence" data-at="0" data-until="1000">bonjour</span>',
        "subs-clip-masked": '<span class="sentence" data-at="0" data-until="1000"><i class="gap"></i></span>',
        "subs-missing": "",
        None: "",
    }[key]


def assert_card(markup):
    tags = Tags(markup).tags
    assert sum(tag == "script" for tag, _ in tags) == 1
    script = re.search(r"<script>\s*\(\(\) => \{.*?\}\)\(\);\s*</script>", markup, re.S)
    assert script, markup
    assert sum(tag == "style" for tag, _ in tags) == 1
    style = re.match(r"\s*<style>.*?</style>\s*", markup, re.S)
    assert style and ".scene {" in style.group(), markup
    body = markup[style.end():script.start()].strip()
    assert not markup[script.end():].strip()
    assert body.startswith('<div class="yap">') and body.endswith('</div>')
    assert_safe(body)
    assert not any(tag in ("audio", "track") or "autoplay" in attrs for tag, attrs in tags)
    for tag, attrs in tags:
        if tag == "video":
            assert attrs.keys() == {"poster", "preload", "playsinline"} and attrs["preload"] == "metadata", attrs
    assert '<hr id="answer">' in body
    return body


def autoplay_sources(render, side):
    native = getattr(render, side + "_av_tags")
    tags = Tags(getattr(render, side + "_text")).tags
    return len(native) + sum(
        tag in ("audio", "video") and "autoplay" in attrs for tag, attrs in tags
    )


def import_package(collection, package, always=False):
    options = ImportAnkiPackageOptions(with_scheduling=False, with_deck_configs=False)
    if always:
        options.update_notes = 1
        options.update_notetypes = 1
    return collection.import_anki_package(
        ImportAnkiPackageRequest(package_path=str(package), options=options)
    )


def counts(collection):
    return (
        collection.db.scalar("select count(*) from notes"),
        collection.db.scalar("select count(*) from cards"),
    )


def check_sql(package, plan, directory):
    with zipfile.ZipFile(package) as archive:
        assert set(archive.namelist()) == {"collection.anki2", "media", "0", "1", "2", "3"}
        media = json.loads(archive.read("media"))
        assert set(media.values()) == {"human.ogg", "poster.jpg", "tts.wav", "word.wav"}, media
        for key in media:
            assert archive.read(key)
        archive.extract("collection.anki2", directory)

    db = sqlite3.connect(str(directory / "collection.anki2"))
    db.row_factory = sqlite3.Row
    try:
        col = db.execute("select * from col").fetchone()
        assert col["ver"] == 11
        assert "1" in json.loads(col["decks"])
        assert "1" in json.loads(col["dconf"])
        assert json.loads(col["conf"])["nextPos"] > len(plan["notes"])
        model = json.loads(col["models"])[str(plan["sentence_model_id"])]
        word_model = json.loads(col["models"])[str(plan["word_model_id"])]
        names = {
            plan["sentence_model_id"]: [field["name"] for field in model["flds"]],
            plan["word_model_id"]: [field["name"] for field in word_model["flds"]],
        }
        for flds in names.values():
            assert len(set(flds)) == len(flds), flds
        for model_type in (model, word_model):
            for template in model_type["tmpls"]:
                for side in ("qfmt", "afmt"):
                    markup = template[side]
                    body = assert_card(re.sub(r"{{[#/^][^}]*}}", "", markup))
                    tags = Tags(body).tags
                    assert sum(tag == "video" for tag, _ in tags) == 1
                    sources = [attrs for tag, attrs in tags if tag == "source"]
                    assert sources == [
                        {"src": "{{ClipUrl}}", "type": 'video/mp4; codecs="avc1.64001F, mp4a.40.2"'},
                        {"src": "{{ClipWebmUrl}}", "type": 'video/webm; codecs="vp9, opus"'},
                    ], sources
                    field = "MaskedSubtitles" if template["name"] == "Listening" and side == "qfmt" else "Subtitles"
                    assert '{{#ClipUrl}}<button type="button" class="scene" style="aspect-ratio: {{ClipAspect}}"><video' in markup
                    assert '</video><div class="subs">{{' + field + '}}</div>' in markup
                    assert '<span class="play"><svg ' in markup
                    assert '</button>{{/ClipUrl}}' in markup
                    assert '{{#Source}}<div class="source">{{Source}}</div>{{/Source}}\n<hr id="answer">' in markup
                    assert "FrontSide" not in markup
                # Revealing the answer must not move anything: both sides match
                # down to the answer line, which the badge then the answer follow.
                # The Listening prompt's blanks are the sentence's own words in
                # invisible ink, so it fills the sentence's slot.
                def slots(markup):
                    markup = re.sub(r"{{(kanji|furigana):", "{{", markup)
                    return markup.replace("{{MaskedSubtitles}}", "{{Subtitles}}").replace("{{MaskedSentence}}", "{{Sentence}}")
                front = slots(re.sub(r"^{{#Include\w+}}|{{/Include\w+}}$", "", template["qfmt"]))
                above, below = front.split('<hr id="answer">')
                assert slots(template["afmt"]).startswith(above + '<hr id="answer">'), template["name"]
                eyebrow = {"Reading": "Translate", "Listening": "Listen", "Word": "Word"}[template["name"]]
                assert below.lstrip().startswith(f'<span class="eyebrow">{eyebrow}</span>')
                assert template["afmt"].split('<hr id="answer">')[1].lstrip().startswith('<div class="answer">')
                assert 'class="eyebrow"' not in template["afmt"]
        for ordinal, template in enumerate(model["tmpls"]):
            name = ["Reading", "Listening"][ordinal]
            assert template["name"] == name and template["ord"] == ordinal
            assert template["qfmt"].startswith("{{#Include" + name + "}}")
            assert "{{Translation}}" in template["afmt"]
            assert "{{Glosses}}" in template["afmt"]
        for index, note in enumerate(plan["notes"]):
            row = db.execute("select * from notes where guid=?", (note["guid"],)).fetchone()
            raw = note["word"] if note["type"] == "Word" else note["sentence"]
            assert row["id"] == note["note_id"]
            assert row["mid"] == (plan["word_model_id"] if note["type"] == "Word" else plan["sentence_model_id"])
            assert row["sfld"] == raw
            assert row["csum"] == int(hashlib.sha1(raw.encode()).hexdigest()[:8], 16)
            assert row["tags"].split() == note["tags"]
            values = row["flds"].split("\x1f")
            assert len(values) == len(names[row["mid"]])
            fields = dict(zip(names[row["mid"]], values))
            clip = note.get("clip") or {"mp4": "", "webm": "", "poster": "", "aspect_ratio": None}
            assert html.unescape(fields["ClipUrl"]) == clip["mp4"]
            assert html.unescape(fields["ClipWebmUrl"]) == clip["webm"]
            assert html.unescape(fields["ClipPosterUrl"]) == clip["poster"]
            assert fields["ClipAspect"] == ("" if clip.get("aspect_ratio") is None else str(clip["aspect_ratio"]))
            assert_safe(row["flds"])
            assert fields["Subtitles"] == captions(note.get("subtitles"))
            source = note.get("source")
            expected_source = ""
            if source:
                if source.get("poster_filename"):
                    expected_source += f'<img src="{html.escape(source["poster_filename"])}" alt="">'
                year = f' · {source["year"]}' if source.get("year") else ""
                expected_source += f'<span>{html.escape(source["title"])}{year}</span>'
            assert fields["Source"] == expected_source
            if note["type"] == "Word":
                assert html.unescape(fields["Word"]) == note["word_furigana"]
                assert html.unescape(fields["Definition"]) == note["definition"]
                wanted_ordinals = [0]
            else:
                assert html.unescape(fields["Sentence"]) == note["sentence_furigana"]
                assert html.unescape(fields["Translation"]) == note["translation"]
                glosses = []
                for gloss in note["glosses"]:
                    term = f'<a class="term" lang="fr" href="{html.escape(gloss["url"])}">' if gloss["url"] else '<span class="term" lang="fr">'
                    end = "</a>" if gloss["url"] else "</span>"
                    glosses.append(f'<li>{term}{html.escape(gloss["text"])}{end}<span class="gloss">{html.escape(gloss["gloss"] or "")}</span></li>')
                assert fields["Glosses"] == "".join(glosses)
                assert fields["ProperNouns"] == "".join(
                    "<li>" + "".join(
                        f'<span class="name" lang="fr">{html.escape(span["text"])}</span>' if span["target_language"] else html.escape(span["text"])
                        for span in group["spans"]
                    ) + "</li>"
                    for group in note["proper_nouns"]
                )
                if index == 2:
                    assert fields["TtsBundled"] == "[sound:tts.wav]"
                if index == 4:
                    assert fields["TtsBundled"] == "" and fields["IncludeListening"] == ""
                assert fields["MaskedSubtitles"] == captions(note.get("masked_subtitles"))
                assert fields["MaskedSentence"] == (note.get("masked_sentence") or "")
                wanted_ordinals = ([0] if note["include_reading"] else []) + (
                    [1] if note["include_listening"] and index != 4 else []
                )
            rows = db.execute(
                "select id,did,ord,due,queue,type from cards where nid=? order by ord",
                (note["note_id"],),
            ).fetchall()
            assert [row["ord"] for row in rows] == wanted_ordinals
            assert all(row["id"] == note["card_id"] + row["ord"] and row["did"] == plan["deck_id"] for row in rows)
            assert all(
                row["due"] == index + 1 and row["queue"] == 0 and row["type"] == 0
                for row in rows
            )
    finally:
        db.close()


def check_variant(root, variant):
    plan = json.loads((root / f"{variant}.json").read_text())
    package = root / f"{variant}.apkg"
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        check_sql(package, plan, directory)
        collection = Collection(str(directory / "target.anki2"))
        try:
            for iteration in range(2):
                result = import_package(collection, package)
                assert counts(collection) == (5, plan["stats"]["card_count"] - int(variant != "reading"))
                assert len(result.log.new) == (5 if iteration == 0 else 0)
                for note in plan["notes"]:
                    imported = collection.get_note(collection.db.scalar("select id from notes where guid=?", note["guid"]))
                    assert sorted(imported.tags) == sorted(note["tags"]), (imported.tags, note["tags"])
                assert len(result.log.duplicate) == (0 if iteration == 0 else 5)
                # Anki may normalize timestamp-shaped IDs; GUID identity must still merge.
                snapshot = (
                    collection.db.all("select id,mid from notes order by id"),
                    collection.db.all("select id,nid,did,ord from cards order by id"),
                )
                if iteration == 0:
                    original_snapshot = snapshot
                else:
                    assert snapshot == original_snapshot, (snapshot, original_snapshot)
                for note in plan["notes"]:
                    actual = collection.db.first("select id,mid from notes where guid=?", note["guid"])
                    expected_model = plan["word_model_id"] if note["type"] == "Word" else plan["sentence_model_id"]
                    assert actual[1] == expected_model, (actual, expected_model)
            for card_id in collection.find_cards(""):
                card = collection.get_card(card_id)
                render = card.render_output()
                note = next(note for note in plan["notes"] if note["guid"] == collection.get_note(card.nid).guid)
                is_word = note["type"] == "Word"
                # Word cards play their recording on both sides, like sentence cards.
                bundled = is_word or note["tts"] != "failed.mp3"
                assert autoplay_sources(render, "question") == int(bundled)
                assert autoplay_sources(render, "answer") == int(bundled)
                assert len(render.question_av_tags) == int(bundled)
                assert len(render.answer_av_tags) == int(bundled)
                # With readings, the front strips them and the back shows them as ruby.
                # (The Listening front shows the masked sentence; word fixture 1 has no reading.)
                text = html.escape(note["word"] if is_word else note["sentence"])
                if "仕事" in text:
                    sentence = re.compile(r'<div class="sentence(?: word)? prompt"><span lang="[^"]*">(.*?)</span>', re.S)
                    question = sentence.findall(render.question_text)
                    answer = sentence.findall(render.answer_text)
                    assert len(question) == 1 and len(answer) == 1, (question, answer)
                    if is_word or card.ord == 0:
                        assert text in question[0] and "[" not in question[0], question
                    ruby = "<ruby><rb>仕事</rb><rt>しごと</rt></ruby>"
                    expected = text.replace("仕事", ruby) if plan["furigana"] else text
                    assert expected in answer[0] and "[" not in answer[0], (expected, answer)
                for back, markup in enumerate((render.question_text, render.answer_text)):
                    body = assert_card(markup)
                    videos = [attrs for tag, attrs in Tags(markup).tags if tag == "video"]
                    assert len(videos) == int(bool(note.get("clip")))
                    if videos:
                        assert "src" not in videos[0]
                        sources = [attrs for tag, attrs in Tags(markup).tags if tag == "source"]
                        assert [source["src"] for source in sources] == [note["clip"]["mp4"], note["clip"]["webm"]]
                        assert "autoplay" not in videos[0]
                        assert videos[0]["poster"] == note["clip"]["poster"]
                    field = "masked_subtitles" if not is_word and card.ord == 1 and not back else "subtitles"
                    if not is_word and card.ord == 1 and not back:
                        assert note["masked_sentence"] in body
                    subs = re.findall(r'<div class="subs">(.*?)</div>', body, re.S)
                    assert subs == ([captions(note.get(field))] if videos else []), (note, subs)
                    assert ('<ul class="names">' in body) == bool(note.get("proper_nouns")), (note, body)
                    assert len(re.findall(r'<button type="button" class="scene" style="aspect-ratio: [\d.]*">', body)) == len(videos)
                    assert body.count('<span class="play">') == len(videos)
                    imported = collection.get_note(card.nid)
                    for value in imported.fields:
                        assert_safe(value)
                    if note.get("source"):
                        assert f'<div class="source">{imported["Source"]}</div>\n<hr id="answer">' in body
                    else:
                        assert '<div class="source">' not in body
            media = collection.media.check()
            assert not media.missing and not media.unused, media
            print(f"PASS {variant}: repeat import, SQL, escaping, AV/autoplay, media")
        finally:
            collection.close()


def check_reexport(root, always):
    with tempfile.TemporaryDirectory() as temporary:
        collection = Collection(str(Path(temporary) / "target.anki2"))
        try:
            for variant in ("reading", "both", "listening"):
                import_package(collection, root / f"{variant}.apkg", always)
                assert counts(collection) == (5, 5 if variant == "reading" else 7)
                flags = [
                    [collection.get_note(note_id)[name] for name in ("IncludeReading", "IncludeListening")]
                    for note_id in collection.find_notes('note:"Yap fra-eng sentences"')
                ]
                expected = {
                    "reading": ["1", ""], "both": ["1", "1"], "listening": ["", "1"],
                }[variant]
                assert len(flags) == 3, flags
                assert flags.count(expected) == (3 if variant == "reading" else 2), flags
                if variant != "reading":
                    assert flags.count([expected[0], ""]) == 1, flags
                # Anki adds newly enabled card types, but never deletes existing cards
                # when their conditional fields become empty. Narrowing Both -> Listening
                # therefore retains three Reading cards with blank-front warnings.
                # The user must remove these through Anki's Tools -> Empty Cards.
                blank = sum(
                    "front-of-card-is-blank" in collection.get_card(card_id).render_output().question_text
                    for card_id in collection.find_cards("")
                )
                assert blank == (3 if variant == "listening" else 0), blank
            policy = "always" if always else "if-newer (default)"
            print(f"PASS reexport {policy}: adds enabled cards; narrowing retains 3 blank Reading cards")
        finally:
            collection.close()


def main():
    root = Path(sys.argv[1]).resolve()
    for variant in ("reading", "listening", "both", "readings"):
        check_variant(root, variant)
    for always in (False, True):
        check_reexport(root, always)


if __name__ == "__main__":
    main()
