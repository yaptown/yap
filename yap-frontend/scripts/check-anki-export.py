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
        assert set(archive.namelist()) == {"collection.anki2", "media", "0", "1", "2", "3", "4", "5"}
        media = json.loads(archive.read("media"))
        assert set(media.values()) == {"human.ogg", "poster.jpg", "tts.wav", "word.wav", "_yap-subs-clip.vtt", "_yap-subs-clip-masked.vtt"}, media
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
        assert len(model["flds"]) == 13
        word_model = json.loads(col["models"])[str(plan["word_model_id"])]
        assert len(word_model["flds"]) == 7
        for model_type in (model, word_model):
            assert model_type["flds"][-1]["name"] == "ClipWebmUrl"
            for template in model_type["tmpls"]:
                for side in ("qfmt", "afmt"):
                    tags = Tags(re.sub(r"{{[#/^][^}]*}}", "", template[side])).tags
                    video = next(attrs for tag, attrs in tags if tag == "video")
                    assert "src" not in video
                    sources = [attrs for tag, attrs in tags if tag == "source"]
                    assert sources == [
                        {"src": "{{ClipUrl}}", "type": 'video/mp4; codecs="avc1.64001F, mp4a.40.2"'},
                        {"src": "{{ClipWebmUrl}}", "type": 'video/webm; codecs="vp9, opus"'},
                    ], sources
                assert all("<audio" not in template[side] for side in ("qfmt", "afmt"))
        for ordinal, template in enumerate(model["tmpls"]):
            name = ["Reading", "Listening"][ordinal]
            assert template["name"] == name and template["ord"] == ordinal
            assert template["qfmt"].startswith("{{#Include" + name + "}}")
            for side in ("qfmt", "afmt"):
                markup = template[side]
                assert "FrontSide" not in markup
                # The clip, then the film (poster + title), close both sides.
                assert re.sub(r"{{[^}]*}}", "", markup).strip().endswith('</video>\n<div class="source"></div>')
                tags = Tags(re.sub(r"{{[#/^][^}]*}}", "", markup)).tags
                video = next(attrs for tag, attrs in tags if tag == "video")
                track = next(attrs for tag, attrs in tags if tag == "track")
                field = "MaskedSubtitles" if name == "Listening" and side == "qfmt" else "Subtitles"
                assert track["src"] == "{{" + field + "}}"
                assert track["kind"] == "subtitles" and "default" in track
                assert "crossorigin" not in video
                assert "autoplay" not in video and "poster" not in video
                assert video["preload"] == "metadata"
                assert "controls" in video and "playsinline" in video
                autoplay = [(tag, attrs) for tag, attrs in tags if "autoplay" in attrs]
                assert not autoplay
                assert '<hr id="answer">' in markup
            assert "Tap to reveal the answer" in template["qfmt"]
            assert "Tap to reveal the answer" not in template["afmt"]
        for index, note in enumerate(plan["notes"]):
            row = db.execute("select * from notes where guid=?", (note["guid"],)).fetchone()
            raw = note["word"] if note["type"] == "Word" else note["sentence"]
            assert row["id"] == note["note_id"]
            assert row["mid"] == (plan["word_model_id"] if note["type"] == "Word" else plan["sentence_model_id"])
            assert row["sfld"] == raw
            assert row["csum"] == int(hashlib.sha1(raw.encode()).hexdigest()[:8], 16)
            assert row["tags"].split() == note["tags"]
            fields = row["flds"].split("\x1f")
            clip = note.get("clip") or {"mp4": "", "webm": ""}
            assert html.unescape(fields[4 if note["type"] == "Word" else 6]) == clip["mp4"]
            assert html.unescape(fields[-1]) == clip["webm"]
            assert_safe(row["flds"])
            assert html.unescape(fields[0]) == raw
            if note["type"] == "Word":
                assert html.unescape(fields[1]) == note["definition"]
                assert fields[5] == ("_yap-subs-clip.vtt" if index == 0 else "")
                wanted_ordinals = [0]
            else:
                for field, key in [(1, "translation"), (2, "target_word"), (3, "target_gloss")]:
                    assert html.unescape(fields[field]) == note[key]
                assert "<img src=x" not in fields[4]
                if index == 2:
                    assert fields[7] == "[sound:tts.wav]"
                if index == 4:
                    assert fields[7] == "" and fields[9] == ""
                assert fields[10] == ("" if index == 4 else "_yap-subs-clip.vtt")
                assert fields[11] == "_yap-subs-clip-masked.vtt"
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
                # Template-only tracks must actually survive import, not just render in the HTML.
                for filename in ("_yap-subs-clip.vtt", "_yap-subs-clip-masked.vtt"):
                    assert (Path(collection.media.dir()) / filename).is_file(), filename
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
                assert_safe(render.question_text)
                assert_safe(render.answer_text)
                # Word cards play their recording on both sides, like sentence cards.
                bundled = is_word or note["tts"] != "failed.mp3"
                assert autoplay_sources(render, "question") == int(bundled)
                assert autoplay_sources(render, "answer") == int(bundled)
                assert len(render.question_av_tags) == int(bundled)
                assert len(render.answer_av_tags) == int(bundled)
                for back, markup in enumerate((render.question_text, render.answer_text)):
                    videos = [attrs for tag, attrs in Tags(markup).tags if tag == "video"]
                    assert len(videos) == int(bool(note.get("clip")))
                    if videos:
                        assert "src" not in videos[0]
                        sources = [attrs for tag, attrs in Tags(markup).tags if tag == "source"]
                        assert [source["src"] for source in sources] == [note["clip"]["mp4"], note["clip"]["webm"]]
                        assert "autoplay" not in videos[0]
                        assert "poster" not in videos[0]
                    tracks = [attrs for tag, attrs in Tags(markup).tags if tag == "track"]
                    field = "masked_subtitles" if not is_word and card.ord == 1 and not back else "subtitles"
                    filename = note.get(field)
                    expected = [] if not filename or filename == "_yap-subs-missing.vtt" else [filename]
                    assert [track["src"] for track in tracks] == expected, (note, tracks)
                    assert all(tag != "audio" for tag, _ in Tags(markup).tags)
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
                    collection.get_note(note_id).fields[8:10]
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
    for variant in ("reading", "listening", "both"):
        check_variant(root, variant)
    for always in (False, True):
        check_reexport(root, always)


if __name__ == "__main__":
    main()
