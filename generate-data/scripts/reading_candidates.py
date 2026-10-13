#!/usr/bin/env python3
"""Cache JMdict/CC-CEDICT candidates and EDRDG KANJIDIC2 reading evidence.

Uses only the Python standard library. Shared downloads are locked because courses
run in separate processes. Candidate files are derived data, not usage inventories.
"""
import fcntl
import gzip
import json
import re
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class Reading:
    senses: list = field(default_factory=list)
    evidence: list = field(default_factory=list)

    def add(self, meanings, evidence):
        self.senses.extend(meanings)
        self.evidence.extend(evidence)


def dictionary():
    return defaultdict(lambda: defaultdict(Reading))


ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / '.cache' / 'readings'
HAN = re.compile(r'[㐀-䶿一-鿿豈-﫿\U00020000-\U000323af]')
KANA = re.compile(r'[ぁ-ゖァ-ヶー]+')
MARKS = {'a': 'āáǎà', 'e': 'ēéěè', 'i': 'īíǐì', 'o': 'ōóǒò', 'u': 'ūúǔù', 'ü': 'ǖǘǚǜ'}


def hiragana(text):
    return ''.join(chr(ord(c)-0x60) if 'ァ' <= c <= 'ヶ' else c for c in text)


def learner_lemma_reading(spelling, reading):
    # Dictionary intake is the single source of this learner spelling convention.
    return 'いう' if spelling == '言う' and reading == 'ゆう' else reading


def download(name, url):
    destination = CACHE / name
    with (CACHE / 'download.lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if not destination.exists():
            research = ROOT / 'experiments' / 'readings' / name
            temporary = destination.with_suffix('.part')
            if research.exists():
                shutil.copyfile(research, temporary)
            else:
                subprocess.run(['curl', '-fL', '--retry', '3', '-o', str(temporary), url], check=True)
            temporary.replace(destination)
    return destination


def tone_syllable(text):
    text = text.lower().replace('u:', 'ü').replace('v', 'ü')
    match = re.fullmatch(r'([a-zü]+)([0-5]?)', text)
    if not match:
        return None
    base, tone = match.groups()
    if not tone or tone in '05':
        return base
    position = next((base.index(v) for v in ['a', 'e'] if v in base), None)
    if position is None:
        position = base.index('o') if 'ou' in base else next((i for i in range(len(base)-1, -1, -1) if base[i] in MARKS), None)
    if position is None:
        return None
    return base[:position] + MARKS[base[position]][int(tone)-1] + base[position+1:]


# A sense list per reading keeps meaning boundaries visible to step B; the caps
# only stop a sprawling entry from dominating the prompt.
MAX_SENSES = 12
MAX_SENSE_CHARS = 120
# Classifier lists and cross-references describe other words, not a meaning.
CEDICT_NOISE = re.compile(r'(CL:|see( also)? |also written |also pr\. )')
CEDICT_PRONUNCIATION = re.compile(r'(?<!Taiwan )\b(?P<kind>colloquial|also) pr\. \[(?P<reading>[^\]]+)\]')


def senses(items):
    result = []
    for item in dict.fromkeys(i.strip() for i in items if i.strip()):
        if len(item) > MAX_SENSE_CHARS:
            item = item[:MAX_SENSE_CHARS - 1].rsplit(' ', 1)[0] + '…'
        result.append(item)
    return result[:MAX_SENSES]


def pinyin(text):
    parts = [tone_syllable(s) for s in text.split()]
    return ' '.join(parts) if parts and all(parts) else None


def cedict(path):
    result = dictionary()
    variants = []
    for line_number, line in enumerate(gzip.open(path, 'rt', encoding='utf-8')):
        match = re.match(r'(\S+) (\S+) \[([^\]]+)\] /(.*)/', line)
        if not match:
            continue
        traditional, simplified, reading, gloss = match.groups()
        reading = pinyin(reading)
        if not reading or not HAN.search(simplified):
            continue
        source = f'cedict:{line_number}'
        # Both spellings name this entry, not every entry sharing its simplified
        # form: 發 must not acquire 髮's hair reading through simplified 发.
        for surface in dict.fromkeys([simplified, traditional]):
            variant = re.match(r'(?:old |archaic )?variant of ([^/\[ ]+)(?:\[([^\]]+)\])?', gloss)
            if variant:
                target, pronunciation = variant.groups()
                variants.append((surface, reading, target.split('|')[-1],
                                 pinyin(pronunciation) if pronunciation else None, gloss, source))
                continue
            items = [item for item in gloss.split('/') if not CEDICT_NOISE.match(item)]
            evidence = [dict(source=f'{source}:{i}', spelling=surface, meaning=item,
                             primary=reading, form='primary', notes=[], priority=[])
                        for i, item in enumerate(items)]
            result[surface][reading].add(items, evidence)
            # Taiwan/name restrictions remain in their original meaning text.
            for alternative in CEDICT_PRONUNCIATION.finditer(gloss):
                pronunciation = pinyin(alternative['reading'])
                if pronunciation:
                    result[surface][pronunciation].add(items, [
                        dict(e, form=alternative['kind']) for e in evidence])
    # Resolve references after collecting every entry, including forward links
    # through other variants. Cycles without a real sense retain a short gloss.
    references = defaultdict(list)
    target_readings = defaultdict(set)
    for surface, reading, target, pronunciation, gloss, source in variants:
        references[surface, reading].append((target, pronunciation, gloss, source))
        target_readings[surface].add(reading)
    active, resolved = set(), set()

    def resolve(word, reading):
        key = word, reading
        if key not in references or key in resolved or key in active:
            return result.get(word, {}).get(reading, Reading())
        active.add(key)
        for target, pronunciation, gloss, source in references[key]:
            entries = [resolve(target, pronunciation)] if pronunciation else []
            if not any(e.senses for e in entries):
                readings = set(result.get(target, {})) | target_readings[target]
                entries = [resolve(target, r) for r in sorted(readings)]
            items = [s for e in entries for s in e.senses] or [gloss]
            evidence = [dict(e, source=f"{source}->{e['source']}", spelling=word, form='reference', notes=[gloss, *e['notes']])
                        for entry in entries for e in entry.evidence]
            if not evidence:
                evidence = [dict(source=source, spelling=word, meaning=gloss,
                                 primary=reading, form='reference', notes=[gloss], priority=[])]
            result[word][reading].add(items, evidence)
        active.remove(key)
        resolved.add(key)
        return result[word][reading]

    for word, reading in references:
        resolve(word, reading)
    return result


def conjugation(pos):
    if pos == '&vz;' or pos.startswith('Ichidan verb - zuru verb'):
        return 'zuru'
    if pos in {'&v1;', '&v1-s;'} or pos.startswith('Ichidan verb'):
        return 'ichidan'
    if pos.startswith('&v5') or pos.startswith('Godan verb'):
        if pos == '&v5k-s;' or 'Iku/Yuku' in pos:
            return 'iku'
        if pos == '&v5u-s;' or "'u' ending (special class)" in pos:
            return 'u-special'
        return 'godan'
    if pos in {'&adj-i;', '&adj-ix;'} or pos.startswith('adjective (keiyoushi)'):
        return 'adjective'
    if pos in {'&vs-s;', '&vs-i;'} or pos.startswith('suru verb -'):
        return 'suru'
    if pos == '&vk;' or pos == 'Kuru verb - special class':
        return 'kuru'
    return None


def jmdict(path):
    result = dictionary()
    stems = defaultdict(set)
    inflections = defaultdict(set)
    for _, entry in ET.iterparse(gzip.open(path), events=('end',)):
        if entry.tag != 'entry':
            continue
        if any((p.text or '') in {'proper noun', '&n-pr;'} for p in entry.findall('sense/pos')):
            entry.clear()
            continue
        positions = [p.text or '' for p in entry.findall('sense/pos')]
        entry_senses = []
        inherited_positions = []
        for sense in entry.findall('sense'):
            inherited_positions = [p.text or '' for p in sense.findall('pos')] or inherited_positions
            entry_senses.append((sense, inherited_positions))
        inflects = any(re.search(r'\bverb\b', p) or p.startswith('&v') or p.startswith('adjective (keiyoushi)') or p in {'&adj-i;', '&adj-ix;'} for p in positions)
        spellings = {k.findtext('keb'): k for k in entry.findall('k_ele')}
        for reading in entry.findall('r_ele'):
            if reading.find('re_nokanji') is not None:
                continue
            kana = reading.findtext('reb') or ''
            if not KANA.fullmatch(kana):
                continue
            for spelling in [r.text for r in reading.findall('re_restr')] or spellings:
                if spelling and HAN.search(spelling):
                    normalized = hiragana(kana)
                    normalized = learner_lemma_reading(spelling, normalized)
                    applicable_positions = set()
                    for sense_number, (sense, sense_positions) in enumerate(entry_senses):
                        readings = [r.text for r in sense.findall('stagr')]
                        surfaces = [s.text for s in sense.findall('stagk')]
                        if (readings and kana not in readings) or (surfaces and spelling not in surfaces):
                            continue
                        applicable_positions.update(sense_positions)
                        glosses = [g.text for g in sense.findall('gloss')
                                   if g.text and g.get('{http://www.w3.org/XML/1998/namespace}lang', 'eng') == 'eng']
                        if glosses:
                            meaning = '; '.join(dict.fromkeys(glosses))
                            spelling_entry = spellings[spelling]
                            notes = [n.text or '' for parent, tag in [
                                (spelling_entry, 'ke_inf'), (reading, 're_inf'),
                                (sense, 'misc'), (sense, 'dial'), (sense, 's_inf')]
                                     for n in parent.findall(tag)]
                            # Reading priorities apply only in conjunction with this
                            # spelling's own priority, never another spelling's.
                            priority = [p.text for p in spelling_entry.findall('ke_pri')
                                        if p.text in [r.text for r in reading.findall('re_pri')]]
                            evidence = dict(source=f"jmdict:{entry.findtext('ent_seq')}:{sense_number}",
                                            spelling=spelling, meaning=meaning, primary=normalized,
                                            form='primary', notes=[*sense_positions, *notes], priority=priority)
                            result[spelling][normalized].add([meaning], [evidence])
                            notes = [n.text or '' for n in sense.findall('s_inf')]
                            notes += [n.text or '' for origin in entry.findall('sense')
                                      if (not origin.findall('stagk') or spelling in [s.text for s in origin.findall('stagk')])
                                      and (not origin.findall('stagr') or kana in [r.text for r in origin.findall('stagr')])
                                      for n in origin.findall('s_inf')
                                      if re.match(r'^also [ぁ-ゖァ-ヶー]+ \(all senses\)', n.text or '')]
                            for note in dict.fromkeys(notes):
                                alternative = re.match(r'^also ([ぁ-ゖァ-ヶー]+(?:, [ぁ-ゖァ-ヶー]+)*)(?: \(all senses\))?(?:$|[;,])', note)
                                if alternative:
                                    for kana_alternative in alternative[1].split(', '):
                                        alternative_reading = learner_lemma_reading(
                                            spelling, hiragana(kana_alternative))
                                        result[spelling][alternative_reading].add(
                                            [meaning], [dict(evidence, form='also')])
                    ending = re.search(r'[ぁ-ゖァ-ヶー]+$', spelling) if inflects else None
                    if ending:
                        suffix = hiragana(ending.group())
                        if normalized.endswith(suffix) and len(normalized) > len(suffix):
                            # Only remove kana already aligned in the dictionary form;
                            # inflected surface kana are checked by the Rust validator.
                            stems[spelling[:ending.start()]].add(normalized[:-len(suffix)])
                            for pos in applicable_positions:
                                kind = conjugation(pos)
                                if kind:
                                    inflections[spelling[:ending.start()]].add((spelling, normalized, kind))
        entry.clear()
    return result, stems, inflections


def kanjidic(path):
    """EDRDG KANJIDIC2 on/kun/nanori; keep dots for okurigana alignment."""
    result = defaultdict(set)
    for _, entry in ET.iterparse(gzip.open(path), events=('end',)):
        if entry.tag != 'character':
            continue
        literal = entry.findtext('literal')
        for reading in entry.findall('reading_meaning/rmgroup/reading'):
            if reading.get('r_type') in {'ja_on', 'ja_kun'} and reading.text:
                result[literal].add(hiragana(reading.text))
        for reading in entry.findall('reading_meaning/nanori'):
            if reading.text:
                result[literal].add(hiragana(reading.text))
        entry.clear()
    return result


def write_dictionary(destination, result):
    temporary = destination.with_suffix('.tmp')
    temporary.write_text(json.dumps({word: sorted(readings) for word, readings in sorted(result.items())}, ensure_ascii=False), encoding='utf-8')
    temporary.replace(destination)


def main(language):
    CACHE.mkdir(parents=True, exist_ok=True)
    if language == 'jpn':
        source = download('JMdict_e.gz', 'http://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz')
        result, stems, inflections = jmdict(source)
        write_dictionary(CACHE / 'jpn-reading-stems.json', stems)
        write_dictionary(CACHE / 'jpn-inflections.json', inflections)
        kanji_source = download('kanjidic2.xml.gz', 'https://www.edrdg.org/kanjidic/kanjidic2.xml.gz')
        characters = kanjidic(kanji_source)
        write_dictionary(CACHE / 'jpn-kanjidic.json', characters)
        print(f'jpn: {len(characters)} KANJIDIC2 characters', file=sys.stderr)
    elif language == 'zho-hans':
        source = download('cedict.gz', 'https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz')
        result = cedict(source)
    else:
        raise ValueError(language)
    destination = CACHE / f'{language}-dictionary.json'
    result = {word: [dict(reading=reading, senses=senses(item.senses), evidence=item.evidence)
                     for reading, item in sorted(readings.items())]
              for word, readings in sorted(result.items())}
    temporary = destination.with_suffix('.tmp')
    temporary.write_text(json.dumps(result, ensure_ascii=False), encoding='utf-8')
    temporary.replace(destination)
    print(f'{language}: {len(result)} headwords, {sum(map(len, result.values()))} readings', file=sys.stderr)
    print(destination)


if __name__ == '__main__':
    main(sys.argv[1])
