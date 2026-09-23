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
        assert set(archive.namelist()) == {"collection.anki2", "media", "0", "1", "2"}
        media = json.loads(archive.read("media"))
        assert set(media.values()) == {"human.ogg", "poster.jpg", "tts.wav"}, media
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
        assert len(model["flds"]) == 11
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
                assert tags[-2][0] == "video"
                video = tags[-2][1]
                assert "autoplay" not in video and "poster" not in video
                assert video["preload"] == "metadata"
                assert "controls" in video and "playsinline" in video
                autoplay = [(tag, attrs) for tag, attrs in tags if "autoplay" in attrs]
                assert len(autoplay) == 1
                assert autoplay[0][0] == "audio" and autoplay[0][1]["src"] == "{{TtsUrl}}"
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
            fields = row["flds"].split("\x1f")
            assert_safe(row["flds"])
            assert html.unescape(fields[0]) == raw
            if note["type"] == "Word":
                assert html.unescape(fields[1]) == note["definition"]
                wanted_ordinals = [0]
            else:
                for field, key in [(1, "translation"), (2, "target_word"), (3, "target_gloss")]:
                    assert html.unescape(fields[field]) == note[key]
                assert "<img src=x" not in fields[4]
                if index == 2:
                    assert fields[7] == "[sound:tts.wav]" and fields[8] == "tts.wav"
                if index == 4:
                    assert fields[7] == "" and fields[8] == "https://mock.invalid/fail"
                wanted_ordinals = ([0] if note["include_reading"] else []) + (
                    [1] if note["include_listening"] else []
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
                assert counts(collection) == (5, plan["stats"]["card_count"])
                assert len(result.log.new) == (5 if iteration == 0 else 0)
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
                assert autoplay_sources(render, "question") == int(not is_word)
                bundled = (
                    note["audio"]["type"] == "Bundled" if is_word else
                    note["tts"]["type"] == "Bundled" and note["tts"]["filename"] == "tts.mp3"
                )
                assert autoplay_sources(render, "answer") == 1
                assert len(render.question_av_tags) == int(bundled and not is_word)
                assert len(render.answer_av_tags) == int(bundled)
                if not is_word:
                    for markup in (render.question_text, render.answer_text):
                        videos = [attrs for tag, attrs in Tags(markup).tags if tag == "video"]
                        assert len(videos) == 1 and "autoplay" not in videos[0]
                        assert "poster" not in videos[0]
                        if not bundled:
                            audio = [attrs for tag, attrs in Tags(markup).tags if tag == "audio"]
                            assert len(audio) == 1 and "autoplay" in audio[0]
                            expected_url = "https://mock.invalid/fail" if note["tts"]["type"] == "Bundled" else note["tts"]["url"]
                            assert audio[0]["src"] == expected_url
                            assert audio[0]["preload"] == "none" and "controls" in audio[0]
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
                assert counts(collection) == (5, 5 if variant == "reading" else 8)
                flags = [
                    collection.get_note(note_id).fields[-2:]
                    for note_id in collection.find_notes('note:"Yap fra-eng sentences"')
                ]
                expected = {
                    "reading": ["1", ""], "both": ["1", "1"], "listening": ["", "1"],
                }[variant]
                assert len(flags) == 3 and all(flag == expected for flag in flags), flags
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
