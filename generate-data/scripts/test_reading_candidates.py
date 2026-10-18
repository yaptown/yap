"""Small source-dictionary fixtures; no downloads or corpus state required."""
import gzip
from pathlib import Path
import tempfile
import unittest

import reading_candidates as candidates


class ExtractionTests(unittest.TestCase):
    def extract(self, text, function):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'source.gz'
            with gzip.open(path, 'wt', encoding='utf-8') as file:
                file.write(text)
            return function(path)

    def test_zuru_class_is_not_collapsed_into_ichidan(self):
        self.assertEqual(candidates.conjugation('&vz;'), 'zuru')
        _, _, inflections = self.extract('''<JMdict><entry><ent_seq>1</ent_seq>
<k_ele><keb>信ずる</keb></k_ele><r_ele><reb>しんずる</reb></r_ele>
<sense><pos>Ichidan verb - zuru verb (alternative form of -jiru verbs)</pos><gloss>believe</gloss></sense>
</entry></JMdict>''', candidates.jmdict)
        self.assertIn(('信ずる', 'しんずる', 'zuru'), inflections['信'])

    def test_cedict_variants_names_and_pronunciation_notes(self):
        result = self.extract('''呵 呵 [a1] /old variant of 啊[a1]/
啊 啊 [a1] /ah!/oh!/
啊 啊 [a5] /modal particle/
朴 朴 [Piao2] /surname Piao/Korean surname Park/
那 那 [na4] /(specifier) that; the; those (colloquial pr. [nei4])/Taiwan pr. [na3]/see 哪[na3]/
帳單 帐单 [zhang4 dan1] /bill/also pr. [zhang4 dan5]/
某 某 [mou3] /variant of 无[wu2]/
''', candidates.cedict)
        self.assertEqual(result['呵']['ā'].senses, ['ah!', 'oh!'])
        self.assertEqual(result['朴']['piáo'].senses, ['surname Piao', 'Korean surname Park'])
        self.assertIn('(specifier) that; the; those (colloquial pr. [nei4])', result['那']['nèi'].senses)
        self.assertNotIn('nǎ', result['那'])
        self.assertIn('bill', result['帐单']['zhàng dan'].senses)
        self.assertEqual(result['某']['mǒu'].senses, ['variant of 无[wu2]'])
        primary = result['帐单']['zhàng dān'].evidence[0]
        alternate = result['帐单']['zhàng dan'].evidence[0]
        self.assertEqual(primary['source'], alternate['source'])
        self.assertEqual(alternate['primary'], 'zhàng dān')
        self.assertEqual(alternate['form'], 'also')
        reference = result['呵']['ā'].evidence[0]
        self.assertEqual(reference['form'], 'reference')
        self.assertIn('old variant of 啊[a1]', reference['notes'])
        self.assertEqual(reference['spelling'], '呵')

    def test_cedict_keeps_refusal_and_structural_particle_evidence(self):
        # These source entries distinguish a missing whole-headword construction
        # from a reading lost at dictionary intake.
        result = self.extract('''不了 不了 [bu4 le5] /no thanks (used to politely but informally decline)/
不了 不了 [bu4 liao3] /(as a resultative verb suffix) unable to (do sth)/(pattern: {verb} + 個|个[ge5] + ~) without end; incessantly/
得了 得了 [de2 le5] /all right!/that's enough!/
得了 得了 [de2 liao3] /(emphatically, in rhetorical questions) possible/
得 得 [de5] /structural particle: used after a verb (or adjective as main verb), linking it to following phrase indicating effect, degree, possibility etc/
了 了 [le5] /(completed action marker)/(modal particle indicating change of state, situation now)/
了 了 [liao3] /to finish/
''', candidates.cedict)
        self.assertEqual(set(result['不了']), {'bù le', 'bù liǎo'})
        self.assertIn('decline', result['不了']['bù le'].evidence[0]['meaning'])
        self.assertEqual(result['不了']['bù le'].evidence[0]['primary'], 'bù le')
        self.assertNotIn('de liǎo', result['得了'])
        self.assertIn('rhetorical', result['得了']['dé liǎo'].evidence[0]['meaning'])
        self.assertIn('structural particle', result['得']['de'].evidence[0]['meaning'])
        self.assertEqual(set(result['了']), {'le', 'liǎo'})

    def test_cedict_preserves_entry_bound_traditional_aliases(self):
        result = self.extract('''怎麼 怎么 [zen3 me5] /how?/what?/
怎麽 怎么 [zen3 me5] /variant of 怎麼|怎么[zen3 me5]/
麼 么 [me5] /suffix used in interrogatives/
麽 么 [me5] /variant of 麼|么[me5]/
麼 麽 [mo2] /tiny/insignificant/
發 发 [fa1] /to issue/to develop/
髮 发 [fa4] /hair/
''', candidates.cedict)
        self.assertIn('how?', result['怎麽']['zěn me'].senses)
        self.assertIn('me', result['麽'])
        self.assertTrue(all(e['spelling']=='怎麽' for e in result['怎麽']['zěn me'].evidence))
        self.assertEqual(set(result['发']), {'fā','fà'})
        self.assertEqual(set(result['發']), {'fā'})
        self.assertEqual(set(result['髮']), {'fà'})

    def test_jmdict_also_kana_and_restrictions(self):
        result, _, _ = self.extract('''<JMdict><entry>
<k_ele><keb>良い</keb></k_ele><k_ele><keb>善い</keb></k_ele>
<r_ele><reb>よい</reb></r_ele>
<sense><stagk>良い</stagk><stagr>よい</stagr><s_inf>also いい (all senses)</s_inf><gloss>good</gloss></sense>
<sense><stagk>良い</stagk><gloss>fine</gloss></sense>
</entry><entry><k_ele><keb>別</keb></k_ele><r_ele><reb>べつ</reb></r_ele>
<sense><stagr>ほか</stagr><s_inf>also ほか</s_inf><gloss>other</gloss></sense>
</entry></JMdict>''', candidates.jmdict)
        self.assertEqual(result['良い']['いい'].senses, ['good', 'fine'])
        self.assertNotIn('いい', result['善い'])
        self.assertNotIn('ほか', result['別'])

    def test_priority_register_and_attachment_are_spelling_bound(self):
        result, _, _ = self.extract('''<JMdict><entry><ent_seq>42</ent_seq>
<k_ele><keb>国</keb><ke_pri>news1</ke_pri></k_ele>
<k_ele><keb>故郷</keb><ke_inf>rarely-used kanji form</ke_inf></k_ele>
<r_ele><reb>くに</reb><re_pri>news1</re_pri><re_pri>ichi1</re_pri></r_ele>
<sense><misc>literary</misc><gloss>home country</gloss></sense>
</entry><entry><ent_seq>43</ent_seq><k_ele><keb>時</keb></k_ele><r_ele><reb>どき</reb></r_ele>
<sense><pos>suffix</pos><s_inf>after a noun or masu stem</s_inf><gloss>time for</gloss></sense>
</entry></JMdict>''', candidates.jmdict)
        common = result['国']['くに'].evidence[0]
        exceptional = result['故郷']['くに'].evidence[0]
        self.assertEqual(common['priority'], ['news1'])
        self.assertEqual(exceptional['priority'], [])
        self.assertIn('rarely-used kanji form', exceptional['notes'])
        self.assertIn('literary', exceptional['notes'])
        self.assertEqual(exceptional['source'], 'jmdict:42:0')
        self.assertIn('after a noun or masu stem', result['時']['どき'].evidence[0]['notes'])

    def test_iu_policy_precedes_inflection_and_preserves_other_lemmas(self):
        result, _, index = self.extract('''<JMdict><entry><k_ele><keb>言う</keb></k_ele>
<r_ele><reb>ゆう</reb></r_ele><sense><pos>Godan verb with 'u' ending</pos><gloss>say</gloss></sense>
</entry><entry><k_ele><keb>結う</keb></k_ele><r_ele><reb>ゆう</reb></r_ele>
<sense><pos>Godan verb with 'u' ending</pos><gloss>tie</gloss></sense></entry></JMdict>''', candidates.jmdict)
        self.assertEqual(set(result['言う']), {'いう'})
        self.assertEqual(index['言'], {('言う', 'いう', 'godan')})
        self.assertEqual(set(result['結う']), {'ゆう'})

    def test_forward_variant_links_and_cycles(self):
        result = self.extract('''甲 甲 [jia3] /variant of 乙[yi3]/
乙 乙 [yi3] /variant of 丙[bing3]/
丙 丙 [bing3] /third/
丁 丁 [ding1] /variant of 戊[wu4]/
戊 戊 [wu4] /variant of 丁[ding1]/
''', candidates.cedict)
        self.assertEqual(result['甲']['jiǎ'].senses, ['third'])
        self.assertEqual(result['乙']['yǐ'].senses, ['third'])
        self.assertTrue(result['丁']['dīng'].senses)
        self.assertTrue(result['戊']['wù'].senses)

    def test_conjugation_classes_keep_their_applicable_readings(self):
        _, _, index = self.extract('''<JMdict><entry>
<k_ele><keb>居る</keb></k_ele><r_ele><reb>いる</reb></r_ele><r_ele><reb>おる</reb></r_ele>
<sense><stagr>いる</stagr><pos>Ichidan verb</pos><gloss>be</gloss></sense>
<sense><stagr>おる</stagr><pos>Godan verb with 'ru' ending</pos><gloss>be (humble)</gloss></sense>
</entry></JMdict>''', candidates.jmdict)
        self.assertEqual(index['居'], {('居る', 'いる', 'ichidan'), ('居る', 'おる', 'godan')})

    def test_multiple_also_kana_alternatives(self):
        result, _, _ = self.extract('''<JMdict><entry><k_ele><keb>幅</keb></k_ele><r_ele><reb>ふく</reb></r_ele>
<sense><s_inf>also ぶく, ぷく</s_inf><gloss>scroll counter</gloss></sense></entry></JMdict>''', candidates.jmdict)
        self.assertEqual(set(result['幅']), {'ふく', 'ぶく', 'ぷく'})

    def test_sense_caps(self):
        values = candidates.senses(['word ' * 80] + [f'sense {i}' for i in range(20)])
        self.assertEqual(len(values), 12)
        self.assertTrue(all(len(s) <= 120 for s in values))


if __name__ == '__main__':
    unittest.main()
