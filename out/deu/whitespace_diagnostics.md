# Whitespace Prediction Diagnostics

Total predictions: 3810883
Total errors: 10228
Accuracy: 99.73%

## Error Patterns (sorted by frequency)

### "“" + "–" (792 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Aber damit war die Geschichte noch nicht zu Ende.“ – „Was geschah weiter?“
  - „Aber das gab es gestern schon!“ – „Und auch morgen wird es das geben.“
  - „Alle wissen, dass ihr zusammen seid!“ – „Das sind wir nicht!“

### "–" + "„" (791 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Aber damit war die Geschichte noch nicht zu Ende.“ – „Was geschah weiter?“
  - „Aber das gab es gestern schon!“ – „Und auch morgen wird es das geben.“
  - „Alle wissen, dass ihr zusammen seid!“ – „Das sind wir nicht!“

### "„" + "Ich" (318 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist das erste Mal, dass ich Taninna „Ich liebe dich!“ sage.
  - Es ist keine Schande, „Ich weiß es nicht“ zu sagen.
  - Sie hat mich tausendmal sagen hören „Ich liebe dich“.

### "„" + "Das" (139 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sami nannte das Video „Das Spukhaus“.
  - „Alle wissen, dass ihr zusammen seid!“ – „Das sind wir nicht!“
  - „Das Bezahlen ist Sache der Familie, nicht deine“, antwortete sie.

### "„" + "Wie" (118 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es bedeutet „Wie geht es dir?“.
  - Nicholas hatte sein Buch „Wie man Belletristik schreibt“ fertiggestellt.
  - „Am Sonnabend kann ich nicht.“ – „Wie denn das? Du hast es doch versprochen.“

### "„" + "Nein" (116 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sagt „Nein!“ zum Rassismus!
  - Toms Antwort wäre sicher „Nein!“ gewesen.
  - „Befand sich in dem Zimmer eine Leiche?“ — „Nein, da war keine Leiche.“

### "„" + "Was" (111 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - So richtig „Was zum Teufel?“-groß.
  - „Aber damit war die Geschichte noch nicht zu Ende.“ – „Was geschah weiter?“
  - „Also, hast du es getan?“ – „Was getan?“– „Ihn küssen natürlich!“

### "," + """ (108 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Berüchtigt", "Leoparden küsst man nicht" und "Casablanca".
  - "Deins", "deins".
  - "Gefangen im kosmischen Strahl", "Die große Kälte", ich habe alle.

### "„" + "Ja" (105 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie antworteten alle ohne Ausnahme „Ja“.
  - „Bin ich hier ganz allein?“ – „Ja, Sir.“
  - „Bist du Moslime?“ – „Ja, und du?“ – „Ich bin Christin.“

### "—" + "„" (91 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Befand sich in dem Zimmer eine Leiche?“ — „Nein, da war keine Leiche.“
  - „Darf ich dich zum Mittagessen einladen?“ — „Ja, da würde ich mich sehr freuen.“
  - „Darf ich hier fotografieren?“ — „Nur im Außenbereich.“

### "“" + "—" (90 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Befand sich in dem Zimmer eine Leiche?“ — „Nein, da war keine Leiche.“
  - „Darf ich dich zum Mittagessen einladen?“ — „Ja, da würde ich mich sehr freuen.“
  - „Darf ich hier fotografieren?“ — „Nur im Außenbereich.“

### "„" + "Du" (68 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diese Schlampe hat mir in die Eier getreten." „Du hattest es verdient."
  - „Du beliebst zu scherzen.“ – „Keineswegs.“
  - „Du bist Engländer.“ – „Ja, stimmt, das bin ich.“

### "„" + "Wo" (68 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Da grillt jemand. Ich kann es riechen.“ – „Wo bist du denn?“
  - „Wo bist du denn jetzt, Tom?“ – „Ich bin unterwegs. Ich spreche aus dem Auto.“
  - „Wo bist du?“

### "," + "„" (66 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ach nein,“ antworteten sie, „die Bedingungen sind zu schwer.
  - Ach,“ sprachen sie, „was will der große Kriegsheld hier mitten im Frieden?
  - Amanda“, sagte Jerry, „er ist vom Topp des Großmasts gesprungen.

### "„" + "Sie" (65 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bei Marias Mutter hast du keinen Stein im Brett.“ – „Sie bei mir auch nicht.“
  - „Haben Sie mein Telefon gesehen?“ – „Sie halten es in der Hand.“
  - „Sie dürfen damit anfangen, es wieder aufzurichten“, erwiderte ich.

### "heißt" + """ (63 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "All about Eve" heißt "Alles über Eva".
  - "Spregel" heißt "Sehr erfreut" auf Schweizerisch.
  - "Tatoeba" heißt "zum Beispiel" auf Japanisch.

### "„" + "Warum" (59 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die große Frage ist „Warum?“
  - Ein Seemann taufte sein Schiff die „Warum nicht!“
  - Während die einen „Warum?“ fragen, fragen die anderen „Warum nicht?“

### """ + "ist" (55 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Aix-la-Chapelle" ist Aachen auf Deutsch.
  - "Apparat Mensch" ist ein Meisterwerk!
  - "Clausewitz" ist raus.

### "„" + "Die" (54 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich hoffe, dass die SPD keine Koalition mit der Partei „Die Linke“ bilden wird.
  - Sie gründeten eine Musikgruppe mit dem Namen „Die Eichhörnchen“.
  - Tom hat „Die Kriegskunst“ von Sūnzǐ gelesen.

### "nicht" + """ (53 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Eigentum", nicht "Besitz".
  - Aber das macht nicht "quak, quak quak"!
  - Aber sie nennen sich nicht "Cocks".

### "„" + "Es" (48 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du hast mich ohne Erlaubnis geküßt.“ – „Es war nur ein Bussi auf die Wange.“
  - „Es geht doch um die Liegegebühren, oder?“, fragte ich.
  - „Es geht nicht ums Feilschen“, erwiderte ich.

### "geht" + "'" (46 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ab geht's!
  - Ab geht's.
  - Also, auf geht's.

### "„" + "Hast" (46 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hast du Durst?“ – „Ja.“
  - „Hast du Geschwister?“ – „Nein, ich bin ein Einzelkind.“
  - „Hast du Holzbearbeitungswerkzeuge zum Tauschen dabei?“, fragte er.

### "„" + "Wir" (46 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nur, weißt du, „Wir sind nicht allein.“
  - „Oh, Marilla, was werden wir nur ohne ihn tun?“ – „Wir haben ja einander, Anne.“
  - „Wann fahrt ihr?“ – „Wir fahren übermorgen.“

### "mit" + """ (45 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Antworte mit "hier".
  - Außerdem einen Liebesbrief mit "Ja, Nein, Vielleicht".
  - Bist du weiter mit "Monte Christo"?

### "das" + """ (42 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber das "Beruhigungsmittel" hast du noch bei dir.
  - Aber das "du" ist bedeutungslos.
  - Ach so, jetzt mache ich das "Bing" mit, ich Depp.

### "“" + "„" (42 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Bist du denn in Urlaub?“ „Ja, eine Woche.“
  - „Bist du glücklich?“ „Ja, ich glaube, ich bin es.“
  - „Bist du müde?“ „Nicht wirklich.“

### "“" + "ist" (41 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Beethoovens „Ode an die Freude“ ist die offizielle Hymne der Europäischen Union.
  - Das Gefühl von „Selbst“ ist identisch mit dem, was wir Bewusstsein nennen.
  - Das Gegenteil von „aufwachen“ ist „einschlafen“.

### "ist" + """ (39 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber meine Formation ist "überholt".
  - Alles, was ich hören will, ist "ja" oder "nein".
  - Da ist "Titanic auf der Fontanka" mit Grebenstschikow.

### "„" + "Aber" (39 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber damit kannst du Notizen machen?“
  - „Aber damit war die Geschichte noch nicht zu Ende.“ – „Was geschah weiter?“
  - „Aber dann steckt dein Geld in Rubricum fest“, erwiderte er.

### """ + """ (37 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Als ich ein Kind war, redete ich wie ein Kind" "und war klug wie ein Kind."
  - "Bist du Schwede?" "Nein, Schweizer."
  - "Bist du müde?" "Nein, überhaupt nicht."

### "der" + """ (37 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ah, der "Überfall".
  - Bei der "Sylvia North Story".
  - Daher wählte ich einen Namen, der "strahlen" bedeutet.

### "„" + "Ist" (37 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gib mir etwas zum Schreiben.“ „Ist das recht?“ „Ja, das ist gut.“
  - „Ich verbringe den Abend mit Tom.“ – „Ist Tom nicht deine Katze?“
  - „Ist Familie Müller schon wieder im Urlaub?“ – „Ja, in Cuxhaven an der Nordsee.“

### "„" + "Tom" (34 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Satz enthält zwei Subjekte, nämlich „Tom“ und „Mary“.
  - Ich kann die Akte „Tom Hansen“ nicht finden.
  - Wir ermitteln in der Mordsache „Tom“.

### """ + "zu" (32 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist der einzige Weg die "Fliegenden Dolche" zu entlarven.
  - Der Inspektor ist da, um über das "Rassenmanifest" zu reden.
  - Der echte Gott würde nicht "Perle" zu meiner Fernbedienung sagen.

### "„" + "Bist" (32 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bist du Einzelkind?“ – „Nein, ich habe zwei Brüder und drei Schwestern.“
  - „Bist du Moslime?“ – „Ja, und du?“ – „Ich bin Christin.“
  - „Bist du Niederländerin?“ – „Ja.“

### "„" + "Und" (32 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber das gab es gestern schon!“ – „Und auch morgen wird es das geben.“
  - „Du bist eine schöne Frau.“ – „Und du bist ein gutaussehender Mann.“
  - „Ich habe keine Ideologie.“ – „Und ich habe keinen Akzent.“

### "„" + "Der" (31 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Apuleius schrieb „Der goldene Esel“.
  - Ich habe auf der Oberschule „Der große Gatsby“ gelesen.
  - Wer hat „Der Fürst“ geschrieben?

### """ + "und" (30 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Berüchtigt", "Leoparden küsst man nicht" und "Casablanca".
  - "FBI" und "Bonanza".
  - "Jugend forscht" und so.

### "-" + "und" (30 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als geborene Münchnerin kennt sie die Stadt in- und auswendig.
  - Backbord- und Steuerbordwachen!
  - Dann "Hals- und Beinbruch".

### "“" + "fragte" (30 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Alle haben Ruderhandschuhe?“ fragte sie und machte die letzte Kontrolle.
  - „Du hast doch nichts dagegen, dass ich dich Liisa nenne?“ fragte Markku.
  - „Eignerin?“ fragte Vicki.

### "„" + "Wer" (30 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die macht das mit Absicht.“ – „Wer macht was mit Absicht?“
  - „Er ist noch zu haben.“ – „Wer ist noch zu haben?“
  - „Es können nicht alle auf deine Feier kommen.“ — „Wer kommt denn nicht?“

### "die" + """ (29 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als Wiedergutmachung für die "Überraschung".
  - Also ihr seid also die "Welle-Faschos", ja?
  - Das ist der einzige Weg die "Fliegenden Dolche" zu entlarven.

### "ein" + """ (27 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ach ja, und er hatte da unten so ein "V."
  - Also, kann ich ein "G" für großartig bekommen?
  - An "toujours" immer noch ein "s".

### "von" + """ (27 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da sind überall Folgen von "West Wing" drauf.
  - Das ist die Definition von "nichts".
  - Der Regisseur von "Rosemaries Baby" höchstpersönlich.

### "“" + "sagte" (27 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Aber die armen Kinder dauern mich doch“ sagte der Mann.
  - „Ach, das ist eine gute Idee!“ sagte sie.
  - „Danke, dass du mich gekratzt hast!“ sagte Taninna zu ihrer Katze.

### "„" + "Danke" (27 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es geht mir auf die Nerven, wenn du vergisst, „Danke!“ zu sagen.
  - Ich wollte nur „Danke!“ sagen.
  - Kennst du das Wort „Danke“?

### """ + "von" (25 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Angst" von Joseph Turner White?
  - "Auto" von selbst und "kratein", herrschen, also Macht.
  - "Erste Liebe" von Turgenjew.

### "du" + """ (24 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber ich könnte auch sagen, weil du "auch zu verrückt" bist.
  - Bist du "pike"?
  - Du gehst du "Lymphom" und "Tuberkulose".

### "„" + "Er" (24 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Er hat Kredite in Loricum?“, sagte Amelia.
  - „Er hat aus Nordwesten allmählich zugenommen“, antwortete Jennifer.
  - „Er hat gute Arbeit geleistet.“

### "den" + """ (23 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also kannte sie den "Antillais"?
  - Bei den "Leichen" war nur das Ehepaar?
  - Chatur nannten alle den "Silencer".

### "in" + """ (23 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Daraus sahen Humphrey Bogart und Ingrid Bergman in "Casablanca".
  - Das Bates Motel in "Psycho" hatte keine separaten Bungalows.
  - Das läuft immer in "The City".

### "„" + "Ach" (23 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ach Gott, wo bin ich?“ rief es.
  - „Ach nein,“ antwortete sie, „es ist kein Riese, sondern ein garstiger Frosch.“
  - „Ach nein“, sagte Lee.

### "„" + "Von" (23 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Karte war mit „Von einem Mann, der dich attraktiv findet“ unterschrieben.
  - „Von Falschheit kann nie Gutes kommen, Mutter“, sagte sie.
  - „Von einer Papierprüfung habe ich noch nie gehört“, sagte ich.

### "sie" + """ (22 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle nannten sie "Índia", einfach "Índia".
  - Also habe ich sie "Überdosis" getauft.
  - Also war sie "Índia", und mein Vater war "der Hübsche".

### "„" + "In" (22 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gib mir, bitte, etwas Wasser.“ „In Ordnung.“
  - „Ich wohne in Rußland. Und du?“ – „In der Ukraine.“
  - „In Eastport gibt es nicht genug Geschäft für zwei Buchhandlungen“, sagte er.

### "bei" + """ (20 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bist du Juror bei "Pakistans Next Top Model"?
  - Dann bricht er sich bei "Shiloh Ranch" die Schulter, dann bin ich's geworden.
  - Die machen auch mit bei "Jugend spielt für Jugend".

### "und" + """ (20 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Auto" von selbst und "kratein", herrschen, also Macht.
  - "Berüchtigt", "Leoparden küsst man nicht" und "Casablanca".
  - "FBI" und "Bonanza".

### """ + "sagen" (19 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da kann ich nicht mehr "nein" sagen!
  - Darf ich "du" sagen?
  - Du hättest gleich am Anfang "Gratis-Europa-Reise" sagen können.

### "," + "-" (19 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber wenn er etwas hat, was wir brauchen, - lasse ich ihn länger leben.
  - Aber wie dem auch sei, - jetzt ist alles klar.
  - Die Ballerinas trugen Ballettstrumpfhosen, -röckchen und -schuhe.

### "»" + "Ich" (19 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich bin Martina Parker«, sagte sie.
  - »Ich bin tatsächlich Chiang.«
  - »Ich brauche noch einen Setzer und mehr Lettern.«

### "„" + "Hat" (19 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hat Chris noch etwas dazu gesagt, wann sein Boot fertig wird?“
  - „Hat Ihnen seine Musik gefallen?“ - „Nein, Ihnen?“
  - „Hat Markofferson sich Geld von Ihnen geliehen?“

### """ + "genannt" (18 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber Val hat ihn "Klimper-Ray" genannt.
  - Auch "Schwein" genannt.
  - Da drinnen wird sie doch von allen "herzensgute Frau Geum-Ja" genannt.

### """ + "gesagt" (18 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Aktenkoffer" gesagt.
  - Du hast "Jeffrey" gesagt.
  - Haben Sie gerade "Arschloch" gesagt?

### "„" + "Sind" (18 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sind Sie Russe?“ – „Ja.“
  - „Sind Sie Russen?“ – „Ja.“
  - „Sind Sie Russin?“ – „Ja.“

### """ + "auf" (17 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ich muss noch mal zurückkommen" auf Glorias wiederholte Fehlzeiten.
  - "R" auf der rechten Wange.
  - "Spregel" heißt "Sehr erfreut" auf Schweizerisch.

### "man" + """ (17 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf Deutsch sagt man "Zum Wohle".
  - Da sagt man "Entschuldigung".
  - Das nennt man "Methode".

### "nicht" + "“" (17 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist keine Schande, „Ich weiß es nicht“ zu sagen.
  - Was meinst du mit „Ich mag es nicht“?
  - „Das glaube ich nicht“, sagte er.

### "„" + "Keine" (17 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom geht mir auf die Nerven mit seinem ewigen „Keine Lust!“
  - „Danke für Ihre Hilfe!“ – „Keine Ursache.“
  - „Danke für deine Hilfe!“ – „Keine Ursache!“

### "Wort" + """ (16 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Apropos, wir sollten uns die Rechte an dem Wort "gut" holen.
  - Das Wort "gigantisch" gefällt mir sehr.
  - Es gibt keine englischen Sätze bei Tatoeba, in denen das Wort "Lego" vorkommt.

### "“" + "sprach" (16 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der eine war ihr zu dick, „das Weinfass!“ sprach sie.
  - Kind,“ sprach die Alte, „wie du aussiehst!
  - „Aber du, Aschenputtel,“ sprach er, „was willst du haben?“

### "„" + "Darf" (16 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darf ich Ihnen noch etwas bringen?“ – „Nein, die Rechnung bitte.“
  - „Darf ich Sie morgen anrufen?“ – „Wenn Sie wollen.“
  - „Darf ich dich fragen, wo du herkommst?“ – „Aus Bologna.“

### "„" + "Hier" (16 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hier also,“ fuhr sie fort, „werden die Belege aufbewahrt.“
  - „Hier geht es um Geld!“
  - „Hier gibt es kaum Beweise“, verkündete er.

### "mich" + """ (15 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber nenn mich "Kiwi".
  - Aber wenn sie mich "bescheuert" nennt, sagst du nichts.
  - Darum nannte sie mich "Lady".

### "„" + "Eine" (15 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eine Katze?“ fragte der alte Mann.
  - „Eine Menge“, antwortete Kat.
  - „Eine Milliarde Bücher?“

### "„" + "Ihr" (15 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ihr Name kommt mir sehr bekannt vor, Herr Holmes“, sagte er mit einem Lächeln.
  - „Ihr alle habt es perfekt gemacht!“
  - „Ihr habt Angst vor der Natur!“ „Was?“

### "„" + "So" (15 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hast du das gesehen?“ – „So klar, wie ich dich sehe.“
  - „So geht man auf Bringobble mit Geld um“, antwortete er.
  - „So hatte ich es nicht gesehen, aber du hast recht“, sagte ich.

### """ + "oder" (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Das Verbrechens-Bekämpfungs-Trio" oder "Das Lächeln des Unvermeidlichen".
  - "Verlass mich nicht" oder so was.
  - Aber glauben Sie wirklich, dass jemand Sie "vernichten" oder hinrichten will?

### "-" + "„" (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Fandst du seine Musik gut?“ - „Nein, du?“
  - „Geht Ihr Mann zu Fuß zur Arbeit?“ - „Ja, er geht gern zu Fuß.“
  - „Geht dein Mann zu Fuß zur Arbeit?“ - „Ja, er geht gern zu Fuß.“

### "für" + """ (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Khansama" ist Urdu für "Diener".
  - "Nigger" ist ein Schimpfwort für "Neger".
  - Das gleicht ja dem Zeichen für "Schwert".

### "“" + "oder" (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bist du für „City“ oder die „Rovers“?
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.
  - Haben Sie „Import“ oder „Export“ gesagt?

### "„" + "Kann" (14 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ist er verheiratet?“ – „Kann sein, dass nicht.“
  - „Kann Tom Trompete spielen?“ – „Nein, aber er ist ein ganz schön guter Sänger.“
  - „Kann er dich denn auch gut leiden?“ – „Keine Ahnung.“

### """ + "bedeutet" (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Biologisch" bedeutet, dass der Samen nicht von mir stammt.
  - "Bracelet" bedeutet auf Englisch das gleiche wie auf Französisch.
  - "Lüge nicht" bedeutet nicht unbedingt "sage die Wahrheit".

### "Das" + """ (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das "A" geht nicht.
  - Das "Chu Jia" am Hafen.
  - Das "Comando Vermelho" hat die Köpfe des ADA-Kartells erledigt.

### "Sie" + """ (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber es gab einen Grund, warum er Sie "meine Gewitterwolke" nannte.
  - Aber glauben Sie wirklich, dass jemand Sie "vernichten" oder hinrichten will?
  - Ah, gut, dann nehmen Sie "Wolf".

### "Und" + """ (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und "Bacurau"?
  - Und "Kennst du den Türcode?"
  - Und "Mama Rose"?

### "als" + """ (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Menschen mit besonderen Fähigkeiten" klingt besser als "behinderte Menschen".
  - "Yamahoozie Polka", auch bekannt als "Kiss Me Polka"?
  - Die andere These bezeichnen Sie als "unwahrscheinlich".

### "ihn" + """ (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber Val hat ihn "Klimper-Ray" genannt.
  - Der war bewaffnet, und er hat ihn "Fiesling" genannt.
  - Hinter seinem Rücken nennen sie ihn "Fahnder".

### "“" + "-" (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Fandst du seine Musik gut?“ - „Nein, du?“
  - „Geht Ihr Mann zu Fuß zur Arbeit?“ - „Ja, er geht gern zu Fuß.“
  - „Geht dein Mann zu Fuß zur Arbeit?“ - „Ja, er geht gern zu Fuß.“

### "“" + "zu" (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Denken Sie ernsthaft darüber nach, das auf „eBay“ zu verkaufen?
  - Er kam und ging, ohne auch nur „Gute Nacht!“ zu sagen.
  - Es geht mir auf die Nerven, wenn du vergisst, „Danke!“ zu sagen.

### "„" + "Auf" (13 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Geh ihm „Auf Wiedersehen“ sagen, bevor er weg ist.
  - Geh und sag ihm „Auf Wiedersehen“, bevor er wegfährt.
  - Sagt „Auf Wiedersehen!“!

### "„" + "Ein" (13 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ein Müller, lieber Vater,“ antwortete er.
  - „Ein Raumschiff ist gelandet.“
  - „Ein Signal vom Gasthaus am Ende der Welt.“

### "„" + "Nicht" (13 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bist du müde?“ „Nicht wirklich.“
  - „Hast du etwas zu tun?“ „Nicht wirklich.“
  - „Ich will mit niemandem reden.“ – „Nicht mal mit deiner Liebsten?“

### "„" + "Weil" (13 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich weiß nicht, warum ich lache.“ – „Weil du blöd bist!“
  - „Warum belegst du dir die Kekse denn mit Käse?“ – „Weil das schmeckt.“
  - „Warum bezeichnest du sie als Lügnerin?“ – „Weil sie eine ist.“

### "dem" + """ (12 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da ist nur einer, auf dem "Clips" steht!
  - Dann wird es zum reinen Geschäft mit dem "cool"-Sein.
  - Das Grab, auf dem "Unbekannt" steht, neben Arch Stanton.

### "hier" + """ (12 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nein" heißt hier "Ja".
  - Daher heißt die Krankheit hier "Baziras Fluch".
  - Und hier "Kirche".

### "“" + "rief" (12 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ach Gott, wo bin ich?“ rief es.
  - „Ach, wie wird mirs ergehen!“ rief sie, „was wird mein Mann dazu sagen!“
  - „Bist du dir dessen sicher?“ rief Lucia.

### "„" + "Haben" (12 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haben Sie Hunger?“ – „Noch nicht.“
  - „Haben Sie Kinder?“ – „Ja, ich habe einen Sohn.“
  - „Haben Sie das schon getan?“ – „Nein, noch nicht.“

### "„" + "Na" (12 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Da kommt Tom.“ – „Na bravo! Der hat mir gerade noch gefehlt.“
  - „Das wäre Betrug.“ – „Na und?“
  - „Ist das ein Witz?“ – „Na klar ist das ein Witz!“

### "„" + "Wann" (12 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe aufgehört zu rauchen.“ － „Wann?“ － „Vor zwei Tagen.“
  - „Wann bekomme ich mein Geld?“
  - „Wann bist du denn wiedergekommen?“ – „Vorgestern.“

### """ + "heißt" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "All about Eve" heißt "Alles über Eva".
  - "Auf einer Wellenlänge" heißt, auf deiner.
  - "Nein" heißt hier "Ja".

### """ + "in" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Undercover" in Morsezeichen.
  - Der Koch war früher im "Tang" in Shanghai.
  - Dr. Baker, Sie haben sicher den Namen "de Winter" in der Zeitung gelesen?

### "Sag" + "'" (11 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sag's ihr, Emily, und lass uns gehen.
  - Sag's mir doch.
  - Sag's mir nicht.

### "Was" + """ (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was "Vertrau mir"?
  - Was "alles"?
  - Was "bei uns nicht"?

### "aus" + """ (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber das ist die aus "Tal der Puppen".
  - Das ist aus "Spektakulär, Spektakulär".
  - Das ist aus "Tom & Jerry", oder?

### "bedeutet" + """ (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Muad'Dib" bedeutet "Känguru-Maus".
  - Das bedeutet "Trottel".
  - Das rote Licht an der Ampel bedeutet "Stop".

### "nur" + """ (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei Ihrer persönlichen Einschätzung steht nur "pragmatisch".
  - Die im Moment der Erregung nur "Jesus Maria" sagt.
  - Er verließ das Haus, ohne auch nur "Auf Wiedersehen" zu sagen.

### "so" + "'n" (11 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das konnte doch keiner ahnen, dass der Typ so'n hohes Tier war.
  - Die Bullen haben richtige Knarren, nicht so'n Schrott.
  - Ich mag so'n Getue nicht.

### "steht" + """ (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber hier steht "Postamt".
  - Auf dem Schild draußen steht "Louis Tenne".
  - Auf dem Schild steht "Ausgang".

### "“" + "gesagt" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fast hätte ich „Antonio“ gesagt, aber das ging ihn nichts an.
  - Haben Sie „Import“ oder „Export“ gesagt?
  - Haben Sie „drei Wochen“ oder „drei Monate“ gesagt?

### "“" + "und" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.
  - Der Satz enthält zwei Subjekte, nämlich „Tom“ und „Mary“.
  - Die Anwohner nennen diesen Fluss „Menschenfresser“ und fürchten ihn.

### "„" + "Guten" (11 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie grüßte mich mit einem „Guten Morgen“.
  - „Guten Abend, Amanda“, sagte Sparhawk.
  - „Guten Abend, Granny“, sagte Kat höflich.

### "„" + "Wessen" (11 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wessen Armbänder sind das?“ — „Majdas.“
  - „Wessen Armreife sind das?“ — „Das sind Majdas Armreife.“
  - „Wessen Bleistifte sind das?“ — „Die von Walt Disney.“

### "-" + """ (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bist du in Ordnung?" - "Ja."
  - "Bist du in Ordnung?" - "Nein."
  - "Denkst Du, dass er kommen wird?" - "Ich hoffe es nicht."

### "auf" + """ (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Anklage lautet auf "Verschwörung und versuchter Mord".
  - Einfach auf "Play" drücken.
  - Ich hatte mich vorbereitet auf ein Leben, auf "ein höheres Leben", sozusagen.

### "“" + "auf" (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Beim Übersetzen muss man vor „falschen Freunden“ auf der Hut sein!
  - Können Sie „Zum Geburtstag viel Glück“ auf französisch singen?
  - Was heißt „Liebe“ auf Französisch?

### "„" + "Kannst" (10 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kannst du Französisch?“ – „Nein, aber ich kann Englisch und Deutsch.“
  - „Kannst du Französisch?“ – „Nein.“
  - „Kannst du das verstehen?“ – „Nein.“

### "„" + "Natürlich" (10 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es macht Ihnen doch nichts aus, oder?“ — „Natürlich nicht.“
  - „Es macht euch doch nichts aus, oder?“ — „Natürlich nicht.“
  - „Könntest du mir mit meinen Englischhausaufgaben helfen?“ – „Natürlich.“

### "„" + "Nichts" (10 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Danke für deine Hilfe!“ – „Nichts zu danken!“
  - „Danke für eure Hilfe!“ – „Nichts zu danken!“
  - „Danke!“ – „Nichts zu danken!“

### "„" + "Nun" (10 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nun bist du die schönste gewesen“ sprach sie, und eilte hinaus.
  - „Nun gut, ich sag’s dir, aber nicht lachen!“ – „Ich versprech’s!“
  - „Nun, einen Eigner gibt es noch nicht“, sagte Markofferson.

### "„" + "Nur" (10 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom lebt nach der Devise „Nur der Dumme ist ehrlich“.
  - „Darf ich hier fotografieren?“ — „Nur im Außenbereich.“
  - „Her mit dem Geld!“ – „Nur über meine Leiche!“

### """ + "steht" (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Alles muss raus" steht auf dem Plakat im Schaufenster.
  - "Setsuko" steht da.
  - "Wassertank" steht da.

### "-" + "oder" (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Sphären, in denen SS- oder Gestapo-Offiziere tätig waren.
  - Ein- oder zweimal.
  - Er saß fünf- oder sechsmal in Lexington.

### "Die" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die "Akademiker" hat es geschafft.
  - Die "Akademiker" steckt im Eis fest.
  - Die "Fliegenden Dolche" haben viele Meister.

### "einen" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Sportschule erteilte einen "Schießbefehl".
  - Die meisten Leute würden annehmen, es geht um einen "Koffer".
  - Dieser Junge hat sie für einen "snuff" benutzt.

### "er" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber wenn er "Versuchen wir's noch mal" sagt, bin ich wieder bei ihm.
  - Darum wurde er "Bird" genannt.
  - Deshalb ist er "angerückt".

### "gut" + "“" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er hat den Kurs mit einem „gut“ bestanden.
  - Ich erklärte, was „gut“ bedeutete.
  - „Also gut“, sagte er zum Schulleiter.

### "ich" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - An diesem Ort sage ich "Lebe wohl".
  - Darf ich "du" sagen?
  - Ja, das Boa-Vista-Kino, in dem ich "Der weiße Hai" gesehen habe.

### "im" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bis nächste Woche um dieselbe Zeit im "Chamland"!
  - Es muss im "Van Beek" sein.
  - Ich war im "Kanada".

### "oder" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Das Verbrechens-Bekämpfungs-Trio" oder "Das Lächeln des Unvermeidlichen".
  - Alles, was ich hören will, ist "ja" oder "nein".
  - Das einzige Mittel, damit Sie sagen "ich will" oder "ich will nicht".

### "sagen" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das einzige Mittel, damit Sie sagen "ich will" oder "ich will nicht".
  - Jetzt könntest du eigentlich sagen "usw."
  - Sie sagen "Domo".

### "sagt" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er sagt "Nessa".
  - Er sagt "viel".
  - Keiner sagt "Auf Wiedersehen".

### "sagte" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als sie sagte "Ich habe dich vermisst", begann sie zu weinen.
  - Er sagte "Wahre Liebe", Max.
  - Er sagte "hallo".

### "»" + "Das" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Das Hafen-was?«, fragte ich höflich.
  - »Das Traktornetz.«
  - »Das hat ihn gerettet.« Ich musste niesen.

### "“" + "antwortete" (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mir auch recht,“ antwortete die Bäurin, „meine Äpfel will ich schon los werden.
  - O,“ antwortete der Sohn, „die ist so satt, sie mag kein Blatt.
  - „Ach nein,“ antwortete sie, „es ist kein Riese, sondern ein garstiger Frosch.“

### "“" + "gelesen" (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aus irgendeinem Grund habe ich „m“ gelesen und nicht „n“.
  - Haben Sie schon einmal „Moby Dick“ gelesen?
  - Habt ihr schon einmal „Moby Dick“ gelesen?

### "„" + "Bei" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bei Marias Mutter hast du keinen Stein im Brett.“ – „Sie bei mir auch nicht.“
  - „Bei dem Schiff waren sie Geschäftspartner“, sagte ich.
  - „Bei denen über die Rosinen aus Spencerport.“

### "„" + "Da" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Da grillt jemand. Ich kann es riechen.“ – „Wo bist du denn?“
  - „Da ist Harborview House“, sagte ich.
  - „Da ist ein Hai im Wasser!“, flüsterte sie.

### "„" + "Dann" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dann gnade Gott dem, der fällt!“ sagte Ravenswood.
  - „Dann hat jemand anders ihr alles abgekauft?“
  - „Dann kannst du dich von dem ablenken, was nicht passiert ist.“ Er nickte.

### "„" + "Möchtest" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Möchtest du dich setzen?“
  - „Möchtest du ein Ringbrötchen?“ – „Immer her damit!“
  - „Möchtest du ein Stück Kuchen zum Kaffee?“ – „Das könnte nicht schaden.“

### "„" + "was" (9 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ach,“ sprachen sie, „was will der große Kriegsheld hier mitten im Frieden?
  - Da erschrack der Mann und fragte „was fehlt dir, liebe Frau?“
  - Der König fragte verwundert „was hat das zu bedeuten?“

### """ + "-" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bist du in Ordnung?" - "Ja."
  - "Bist du in Ordnung?" - "Nein."
  - "Denkst Du, dass er kommen wird?" - "Ich hoffe es nicht."

### """ + "hat" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Enchante" hat den Vertrag gekündigt.
  - "Fettes Schwein" hat sie sie genannt.
  - "Hallo" hat der Deutsche gesagt.

### "," + "»" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dies frühzeitige Aufstehen«, dachte er, »macht einen ganz blödsinnig.
  - Gregor,« rief es – es war die Mutter –, »es ist dreiviertel sieben.
  - Herr Samsa,« rief nun der Prokurist mit erhobener Stimme, »was ist denn los?

### "-" + "bis" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er pflegte zwei- bis dreimal im Jahr in dieser Stadt zu weilen.
  - Es gibt fünfzehn- bis zwanzigtausend Zuschauer.
  - Ich bin schon drei- bis viermal pro Woche in der Schule.

### "Der" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der "Antillais" hat Ihnen meine Anweisungen mitgeteilt.
  - Der "Antillais" ist der Kopf eines Prostitutionsrings.
  - Der "Antillais" versorgt hauptsächlich den Großraum Paris.

### "Nein" + "“" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nein“, antwortete Agatha.
  - „Nein“, antwortete Tom entschlossen.
  - „Nein“, antwortete ich.

### "einfach" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle nannten sie "Índia", einfach "Índia".
  - Das Fahrrad heißt in der Schweiz einfach "Velo".
  - Er hatte einfach "Alles gut" gerufen, und wir konnten ruhig schlafen.

### "es" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann ist es "dein Standpunkt".
  - Er wollte uns weismachen, daß es "ein einfacher Trick" ist.
  - Ich nenne es "meine Tüte der Hingabe".

### "sind" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das nächste Thema sind "moderne Legenden".
  - Sie sind "berufen"!
  - Und wer sind "die"?

### "und" + "-" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Ballerinas trugen Ballettstrumpfhosen, -röckchen und -schuhe.
  - Du mußt mehr lernen und weniger herummachen und -tun.
  - Ihr müßt mehr lernen und weniger herummachen und -tun.

### "war" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also war sie "Índia", und mein Vater war "der Hübsche".
  - Auf seinem Visumantrag war "ärztliche Behandlung" als Reisegrund angegeben.
  - Das Wort war "Schlammblut".

### "»" + "Du" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Du hast Jim gerettet«, sagte sie.
  - »Du hast gesehen, wie sie mich mitgenommen haben?«, fragte ich.
  - »Du hast nicht zufällig ein ganzes Nutan dabei?«

### "“" + "bedeutet" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er weiß nicht, was das Wort „Angst“ bedeutet.
  - Wissen Sie überhaupt, was „ironisch“ bedeutet?
  - Wisst ihr überhaupt, was „ironisch“ bedeutet?

### "„" + "Doch" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bist doch wohl kein Kommunist, oder?“ – „Doch.“
  - „Du bist keine Kommunistin, oder?“ – „Doch.“
  - „Du kannst mich nicht so rumkommandieren.“ „Doch, das kann ich und werde ich.“

### "„" + "Für" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Für Feuerfische nehme ich alles“, sagte er.
  - „Für Reparaturen in der Stadt.“
  - „Für dich.“ Sie wandte den Blick ab.

### "„" + "Hallo" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie sah mich an und sagte „Hallo!“
  - „Hallo! Wie geht’s?“ – „Es geht mir gut.“
  - „Hallo!“, rief ich.

### "„" + "Klar" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hast du das Fußballspiel gestern gesehen?“ – „Klar!“
  - „Hast du einen Onkel in Amerika?“ – „Klar, alle haben doch Verwandte dort.“
  - „Ihr macht ein Spiel? Kann ich mitmachen?“ – „Klar.“ – „Super!“

### "„" + "Mein" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Brennholz kommt von dort“, sagte sie.
  - „Mein Halsband“ sagte das Mädchen.
  - „Mein Hund hat keine Nase.“ – „Wie riecht er denn?“ – „Furchtbar.“

### "„" + "Mir" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sage nicht „Mir geht es gut", wenn es dir nicht gut geht!
  - „Das gehört mir nicht!“ – „Mir ebensowenig.“
  - „Er hat mir gesagt, dass er mich liebe.“ – „Mir hat er das gleiche gesagt!“

### "„" + "Noch" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haben Sie Hunger?“ – „Noch nicht.“
  - „Hast du Hunger?“ – „Noch nicht.“
  - „Hast du schon einmal Austern gegessen?“ – „Noch nie.“

### "„" + "Wem" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wem gehören diese Armreifen?“ — „Sie gehören Majda.“
  - „Wem gehören diese Bücher?“ „Alice.“
  - „Wem gehören diese Hunde?“ — „Sie gehören meinen Nachbarn.“

### "„" + "und" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bugleinen liegen“, meldete Paulson, „und sind eingeholt.
  - Das Wort „aber“ ist viel wichtiger als das Wort „und“.
  - Ich stelle Leute ein“, sagte ich, „und bezahle sie.

### "„" + "wer" (8 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der dritte „wer hat von meinem Brötchen genommen?“
  - Der erste sprach „wer hat auf meinem Stühlchen gesessen?“
  - Der fünfte „wer hat mit meinem Gäbelchen gestochen?“

### """ + "nicht" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er kann "Ranger" nicht mal buchstabieren!
  - Ich kenne das Wort "unmöglich" nicht.
  - Ich kenne die Bedeutung des englischen Wortes "gerund" nicht.

### """ + "sagt" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Du vögelst den Milchmann," sagt er.
  - "Was soll das?" sagt er.
  - "Whacken" sagt man nicht.

### """ + "sagte" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Dieb" sagte ich nie.
  - "Eine strahlende Zukunft als Jungfer" sagte der große Dichter.
  - "Ich bin nicht der Schreckliche Pirat Roberts" sagte er, "Mein Name ist Ryan."

### """ + "wäre" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Entscheidend" wäre übertrieben.
  - "Haus Leichtgläubig" wäre treffender gewesen.
  - "Informiert" wäre zu viel gesagt.

### "Ja" + "“" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja“, sagte ich, „solange derjenige, der uns dreizehn schuldet, auch zahlt.
  - Sie antworteten alle ohne Ausnahme „Ja“.
  - „Bist du ein Engel?“ – „Ja“, antwortete der Engel.

### "Lied" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da ist irgendeiner, der will sich immer wieder das Lied "Sad Letter" anhören.
  - Das Lied "Canto general" ertönt.
  - Das Lied "Ich vermisse dein Gesicht", aus deinem Tagebuch, ist das Zeichen.

### "Sag" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sag "Ah".
  - Sag "Ja".
  - Sag "Papa".

### "Träne" + "n" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er hat Tränen in den Augen.
  - Mir kamen die Tränen.
  - Mit ihnen kamen die überfließenden Tränen.

### "an" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bring sie an "einen anderen Ort".
  - Das Schild zeigt an "Montags geschlossen".
  - Die Soldaten der Dunkelheit hießen von nun an "Wächter des Tages".

### "auf" + "'" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die sitzt bei mir auf'm Schoß.
  - Es ist als ob ich auf'm Planeten der Affen arbeite.
  - Es ist so heiß wie auf'm Grill.

### "dich" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als ich dich "Marnie" gerufen habe, bist du erschrocken.
  - Darf ich dich "Lehrmeister" nennen?
  - Deshalb nenne ich dich "Ochse".

### "immer" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Leute nennen ein lrrenhaus immer "einen anderen Ort".
  - Ho Po-Wing sagt immer "Versuchen wir's noch mal."
  - Ihre Antworten waren immer "Ich weiß nicht" oder "Ich erinnere mich nicht".

### "kein" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist kein "Heil".
  - Das ist kein "beurre en motte"!
  - Du bist doch kein "Hubert".

### "was" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist, was "Bis zum Schmerz" bedeutet.
  - Ich glaube, ich weiß, was "Wenn du es baust, wird er kommen" bedeuten soll.
  - In Tschechien gibt es eine Stadt namens Loket, was "Ellenbogen" bedeutet.

### "wir" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ab jetzt heißen wir "die Welle".
  - Deine Erscheinung nennen wir "Rest-Selbstbildnis".
  - Dort bauen wir eine Allee mit Statuen, die wir "Allee mit Statuen" nennen.

### "»" + "Was" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Was glaubst du, womit sie die Kaffee-Zimt-Scones aufpeppt?«, fragte er.
  - »Was haben sie erzählt?«
  - »Was ist das?«, rief er.

### "“" + "genannt" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Slowakei wird auf Slowakisch „Slovensko“ genannt.
  - Er mochte Zeichentrickfilme japanischer Art, auch „Anime“ genannt.
  - Im Gefängnis wurde Tom der „Chirurg“ genannt.

### "„" + "Also" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Also gehört er immer noch dir?“
  - „Also gut“, sagte er zum Schulleiter.
  - „Also verkaufen wir es zum Buchpreis?“, fragte ich.

### "„" + "Bitte" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „Bitte“ auf Wolof?
  - „Bitte nehmen Sie Platz und erzählen Sie mir, wonach Sie suchen.“
  - „Bitte schön“, sagt die Bordbedienung.

### "„" + "Einen" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einen Anteil?“, fragte ich.
  - „Einen Anteil“, sagte Johanna.
  - „Einen Augenblick, Schiffsherrin“, sagte Chiang.

### "„" + "Geht" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Geht Ihr Mann zu Fuß zur Arbeit?“ - „Ja, er geht gern zu Fuß.“
  - „Geht dein Mann zu Fuß zur Arbeit?“ - „Ja, er geht gern zu Fuß.“
  - „Geht es dir besser?“ – „Nein, schlechter.“

### "„" + "Lass" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte doch „Lass es sein!“!
  - „Der Motor springt nicht an.“ – „Lass mich da mal ran!“
  - „Ich sehe mal nach, was da drüben los ist.“ – „Lass es!“

### "„" + "Mit" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eine Portion Pommes frites bitte.“ – „Mit Salz und Essig?“ – „Ja, bitte.“
  - „Mit etwas Glück werdet ihr bald Fisch bekommen“, antwortete ich.
  - „Mit wem sollten wir sprechen?“

### "„" + "Oh" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dieses Handtuch gefällt mir wirklich sehr.“ – „Behalt’s!“ – „Oh, danke schön!“
  - „Es ist nicht mein Baby.“ – „Oh doch, ist es.“
  - „Ich habe keine Angst vor Maria.“ – „Oh, das solltest du aber!“

### "„" + "Welche" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe die Bilder gesehen.“ — „Welche Bilder?“
  - „Welche Fremdsprache hast du genommen?“ – „Englisch.“
  - „Welche Fremdsprache lernst du?“ – „Ich lerne Französisch.“

### "„" + "Willst" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die kleinen Welpen sind ja herzallerliebst!“ – „Willst du einen haben?“
  - „Willst du etwas zu essen haben?“ – „Ach nein, ich habe gerade gegessen.“
  - „Willst du mich heiraten?“ – „Ja, ich will!“

### "„" + "Wohin" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wohin fährt Tom?“ – „Nach München.“
  - „Wohin gehen wir?“ – „An einen sicheren Ort.“
  - „Wohin hast du denn das Auto gestellt?“ – „Auf den Parkplatz.“

### "„" + "das" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Spielmann antwortete „das ist mein und dein Haus, wo wir zusammen wohnen.“
  - Der eine war ihr zu dick, „das Weinfass!“ sprach sie.
  - Für „das kleinere Übel“ entschieden.

### "„" + "ich" (7 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Amanda“, sagte Keith, „ich bin im Moment nicht dein Leibwächter.
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.
  - Es ist leicht, „ich bin“ in jede Sprache zu übersetzen.

### """ + "an" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das wichtigste Wort der Nachricht fängt mit einem "T" an?
  - Der "Mord" an ihr.
  - Er hängte ein Schild "Geschlossen" an die Eingangstür des Geschäftes.

### """ + "für" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "S" für Simón.
  - Also, kann ich ein "G" für großartig bekommen?
  - Ich habe ja "Palmers" für Sie.

### """ + "heißen" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du willst also "David" heißen?
  - Er muss "Chicken On A Stick" heißen.
  - Hör mal, Akira, was soll denn hier bitte schön "schon wieder" heißen?

### """ + "mit" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Wenn der Postmann zweimal klingelt" mit John Garfield.
  - Ach so, jetzt mache ich das "Bing" mit, ich Depp.
  - Da ist "Titanic auf der Fontanka" mit Grebenstschikow.

### """ + "nennt" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber wenn sie mich "bescheuert" nennt, sagst du nichts.
  - Das, was man so "Liebe" nennt.
  - Eine "politische Aktion" nennt er das.

### "Amanda" + "“" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Amanda“, sagte Jerry, „er ist vom Topp des Großmasts gesprungen.
  - Amanda“, sagte Keith, „ich bin im Moment nicht dein Leibwächter.
  - „Aber ich habe das Geld wirklich nicht, Amanda“, sagte Dom höflich.

### "Geld" + "“" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „Geld“ oder „Gold“ gesagt?
  - „Als er noch lebte, schuldete er Ihnen Geld“, berichtigte Brunel.
  - „Chris schuldet mir mein Geld“, sagte Williamson noch einmal.

### "Mit" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit "A" kann ja jeder schreiben.
  - Mit "Absicht" meinte ich sehr koordiniert.
  - Mit "Denken" kommst du nicht ans Ziel.

### "Wieso" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wieso "Godefroy"?
  - Wieso "Granados"?
  - Wieso "Seb's"?

### "also" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das nennst du also "Schatten loswerden", hä?
  - Die Antwort lautet also "nein"?
  - Die Tote war ein Mädchen, also "sie".

### "auch" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bin ich auch "ganz bezaubernd"?
  - Die Beegeum-Spitze wurde früher auch "Ölberg" genannt.
  - Er könnt ihn auch "Henry" oder "Parkplatz" nennen.

### "aus" + "“" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich weiß nicht, ob dieser Lichtschalter auf „an“ oder „aus“ steht.
  - „Das sieht ziemlich interessant aus“, sagt Hiroshi.
  - „Dories halten einiges aus“, sagte Christopher.

### "dein" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Behalt dein "Hey da!"
  - Lass dein "Haha"!
  - Lass dein "Was ist denn"!

### "deine" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - An deine "Wiederkehr"?
  - Behalt deine "drei danach".
  - Du musst deine "Oyster card" aufladen.

### "dich" + "“" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie hat mich tausendmal sagen hören „Ich liebe dich“.
  - Sie ist schüchtern und wagt deshalb nicht zu sagen „Ich liebe dich“.
  - Weltflucht heißt gewöhnlich nicht „Tischlein, deck dich“.

### "doch" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da steht doch "Löffel"!
  - Das war doch "Honey West".
  - Es waren doch "unfehlbare Praktiken".

### "einem" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf Lateinisch beginnt Jehova mit einem "I".
  - Bei seinem Gesicht fällt einem "Das Schweigen der Lämmer" ein.
  - Das wichtigste Wort der Nachricht fängt mit einem "T" an?

### "hat" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Cinemathek hat "Sabor" restauriert.
  - Er hat "Das Manifest" gelesen.
  - Ja, er hat "lebenslänglich" verdient.

### "wie" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Kyo" wie "Tokyo".
  - Dieser Mistkerl sieht haargenau so aus wie "Das Ding".
  - Filme wie "Meine Ehefrau Pamela" werden nicht mehr gemacht.

### "zum" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf zum "Terceiro Comando".
  - Cao Cao hat sich zum "Premierminister" ernannt.
  - Die Spieler zogen ihn damit auf und so wurde er zum "Schuhlosen Joe".

### "«" + "rief" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gregor,« rief es – es war die Mutter –, »es ist dreiviertel sieben.
  - Herr Samsa,« rief nun der Prokurist mit erhobener Stimme, »was ist denn los?
  - Weg muss es,« rief die Schwester, »das ist das einzige Mittel, Vater.

### "“" + "sagen" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da kann ich nur „Donnerwetter!“ sagen.
  - Du könntest wenigstens „danke“ sagen.
  - Geh ihm „Auf Wiedersehen“ sagen, bevor er weg ist.

### "„" + "Bis" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Alles klar, bis morgen!“ – „Bis morgen!“
  - „Bis dahin sitzen wir hier fest“, sagte Vincent.
  - „Bis das fertig ist, wird es eine Weile dauern.“

### "„" + "Diese" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diese Blumen sind für dich, Maria.“ – „Wie süß! Danke!“
  - „Diese Schlampe hat mir in die Eier getreten." „Du hattest es verdient."
  - „Diese Schuldscheine sind so gut wie wertlos“, antwortete ich.

### "„" + "Gibt" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gibt es da einen Witz?“
  - „Gibt es eigentlich etwas, was Tom besser kann als Maria?“ – „Faulenzen.“
  - „Gibt es heute keinen Nachtisch?“ – „Heute nicht. Iss einen Apfel!“

### "„" + "Habt" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Habt ihr Angst?“ – „Kein bißchen!“
  - „Habt ihr Bücher darüber?“
  - „Habt ihr am Wochenende ferngesehen?“ – „Nein.“

### "„" + "Im" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bist du fertig?“ – „Im Gegenteil. Ich habe noch nicht einmal angefangen.“
  - „Bist du soweit?“ – „Im Gegenteil. Ich habe noch nicht einmal angefangen.“
  - „Im Ausland isst man keinen rohen Fisch.“ ist eine Lüge.

### "„" + "Komm" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Komm herab zu uns,“ sagten sie, „wir wollen dir nichts zu Leid tun.“
  - „Komm und sag mir, wer du bist“, sprach jene, „und wie du hergekommen bist.“
  - „Komm und sieh es dir an!“ sagte Philip.

### "„" + "Man" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kann man diesen Pilz essen?“ – „Man kann alles essen – mindestens einmal.“
  - „Man kann Leute nicht nach ihrem Aussehen beurteilen“ ist gut gesagt, oder?
  - „Man muss sie beobachten“, sagte der Grafschaftsbibliothekar.

### "„" + "Meine" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Höre!“ rief der Geist. „Meine Zeit ist fast um.“
  - „Meine Frau besitzt mehrere Hackmesser“, ergänzte Antonio.
  - „Meine Glückwünsche zu Ihrer erfolgreichen Reise“, sagte ich.

### "„" + "Nach" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nach allem, was ihr getan habt, habt ihr das alles verdient.“
  - „Sie hat nach Ihnen gefragt.“ – „Nach mir?“
  - „Sie hat nach dir gefragt.“ – „Nach mir?“

### "„" + "Tut" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du mußt Eintritt zahlen.“ – „Ich dachte, das sei umsonst.“ – „Tut mir leid!“
  - „Haben Sie ein Streichholz?“ – „Tut mir leid, aber ich rauche nicht.“
  - „Kann ich bitte Tom sprechen?“ – „Tut mir leid, er ist gerade rausgegangen.“

### "„" + "Woher" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ist das normal?“ – „Woher soll ich das wissen? Wirke ich auf dich normal?“
  - „Woher bekommt man ein Stiftungskapital?“, fragte ich.
  - „Woher hast du meine Nummer?“ – „Tom hat mir die gegeben.“

### "„" + "drei" (6 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „drei Wochen“ oder „drei Monate“ gesagt?
  - Haben Sie „drei Wochen“ oder „drei Monate“ gesagt?
  - Habt ihr „drei Wochen“ oder „drei Monate“ gesagt?

### """ + "als" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf seinem Visumantrag war "ärztliche Behandlung" als Reisegrund angegeben.
  - Du siehst "Wahrheit" als etwas Absolutes.
  - Hast du kein besseres "Heil mir" als das?

### """ + "ein" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei seinem Gesicht fällt einem "Das Schweigen der Lämmer" ein.
  - Es ist gar nicht albern, dass das "I" in "Wilder" ein Zauberstab ist.
  - Geben Sie mal bei Wikipedia "Vollpfosten" ein.

### """ + "hier" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Yaku" hier, arme Indios dort, und jetzt?
  - Brennt ihm das Ausreißer-"R" hier auf die Wange.
  - Eben sah ich einen "Bufeo" hier.

### """ + "nennen" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Darf ich dich "Lehrmeister" nennen?
  - Dort bauen wir eine Allee mit Statuen, die wir "Allee mit Statuen" nennen.
  - Du sollst mich nicht "Vater" nennen.

### """ + "war" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Apparat Mensch" war oberflächlich.
  - Bei "Unknown Pleasures" war alles perfekt.
  - Bei den "Leichen" war nur das Ehepaar?

### "Eine" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine "Kawasaki".
  - Eine "politische Aktion" nennt er das.
  - Eine "sauer-scharfe" Suppe.

### "Frieden" + "“" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich bin fast fertig mit „Krieg und Frieden“.
  - Ich habe auch nie „Krieg und Frieden“ gelesen.
  - Ich habe „Krieg und Frieden“ auch noch nicht gelesen.

### "Gradisca" + "!" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alles Gute, Gradisca !
  - Da ist Gradisca !
  - Ein Hurra für Gradisca !

### "Monate" + "“" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „drei Wochen“ oder „drei Monate“ gesagt?
  - Habt ihr „drei Wochen“ oder „drei Monate“ gesagt?
  - Hast du „drei Wochen“ oder „drei Monate“ gesagt?

### "Und" + "…" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und … Eines Tages werde ich alt genug sein.
  - Und … Nick?
  - Und … au!

### "aus" + "'" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Da schmeißt du mich aus'm Auto.
  - Die Exporteure, die Frank aus'm Geschäft gedrängt hat.
  - Die haben Sie aus'm Geschäft gekickt.

### "denn" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was bedeutet denn "Woo-Sung" überhaupt?
  - Was heißt denn "für die Reichen"?
  - Was heißt denn "normal"?

### "du" + "?" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gehst du ?
  - Lügst du ?
  - Sieht die nicht aus wie du ?

### "eine" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das war definitiv eine "Desert Eagle".
  - Ist das so eine "Mühe"?
  - War bestimmt wieder so eine "Oma-Geschichte".

### "gerade" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er schreibt dir bestimmt gerade "Mami chula, du fehlst mir."
  - Haben Sie gerade "Arschloch" gesagt?
  - Hast du gerade "ernsthafter Musiker" gesagt?

### "habe" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe "Apocalypse Now" im Kino gesehen.
  - Ich habe "Bounty Law" hier gedreht.
  - Ich habe "Elle" abonniert.

### "haben" + "“" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du wirst dieses riesige Haus haben“, sagte sie.
  - „Ich werde jede Menge Platz haben“, sagte ich.
  - „Ihr müsst ein anderes Wort dafür haben“, sagte Arthur.

### "ich" + "“" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.
  - „Das habe ich“, erwiderte ich.
  - „Der bin ich“, antwortete Aristotle.

### "ja" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber das Gericht heißt ja "Alte Tofu-Frau".
  - Da steht ja "wahrer" Apache.
  - Das ist ja "Das Jüngste Gericht".

### "keine" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber das ist keine "Andere", sondern Jess.
  - Das sind keine "Wesen".
  - Es tut mir leid, wir bieten keine "Entschuldigungs-Dienste" mehr an.

### "mal" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er ging aus dem Haus und sagte nicht mal "Auf Wiedersehen".
  - Können Sie mal "Hi" sagen für mein Instagram?
  - Okay, ich schreibe einfach mal "Sport" dazu.

### "meine" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf meine "In welchen Club gehen wir?"
  - Bist du meine "Fair Lady"?
  - Ich meine "auch verrückt".

### "sein" + "“" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das müssen Gespenster gewesen sein“, sagte ich.
  - „Die Aktionäre müssen begeistert sein“, sagte Clark Laneson.
  - „Ich muss der Bescheidenheit halber ehrlich sein“, sagte Saunders.

### "spielt" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er spielt "Sound of da Police" von KRS-One.
  - Er spielt "Stille Nacht".
  - Er spielt "World of Warcraft".

### "unter" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ihr kennt sie unter "Shaffer", aber ihr wahrer Name lautete Brittle.
  - Ja, unter "Ehemann".
  - Schlagen wir unter "weiß" nach.

### "«" + "sagte" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das »morgen« sagte er mit einem gespielten Knurren.
  - Ich habe ja Appetit,« sagte sich Gregor sorgenvoll, »aber nicht auf diese Dinge.
  - »Mutter, Mutter,« sagte Gregor leise und sah zu ihr hinauf.

### "“" + "lautet" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Genitiv Singular von „rosa“ lautet „rosae“.
  - Der Plural von „Kaktus“ lautet „Kakteen“.
  - Der Plural von „Pferd“ lautet „Pferde“.

### "“" + "steht" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich weiß nicht, ob dieser Lichtschalter auf „an“ oder „aus“ steht.
  - Im Restaurant hängt ein Schild, auf dem „Rauchen verboten!“ steht.
  - Schilder, auf denen „Hochspannung“ steht, sind häufig gelb.

### "„" + "Alle" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Alle Festmacher loswerfen!“, rief ich.
  - „Alle Kanadier sollten, soweit es geht, zu Hause bleiben“, sagte Trudeau.
  - „Alle haben Ruderhandschuhe?“ fragte sie und machte die letzte Kontrolle.

### "„" + "Alles" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Alles Betty!“ ist eine der beliebtesten Fernsehsendungen der Welt.
  - „Alles klar, bis morgen!“ – „Bis morgen!“
  - „Alles steht!“, rief Vickisdotr schließlich vom Bug.

### "„" + "Amanda" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Amanda, Liebes“, begann sie, „du hast Bindfaden Nummer vier gekauft.“
  - „Amanda, bitte stell dich in die Kreuzung der beiden Gänge.“
  - „Amanda, ich freue mich auch, dich zu sehen“, erwiderte er.

### "„" + "Den" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es standen im Park viele Schilder mit der Aufschrift „Den Rasen nicht betreten!“
  - „Den Rausschmeißer des Verbands.“
  - „Den Trick mit dem Ruder würde ich nicht jeden Tag versuchen.“

### "„" + "Gar" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Spüren Sie etwas?“ – „Gar nichts.“
  - „Was empfindest du für Tom?“ – „Gar nichts. Er ist mir egal.“
  - „Was haben Sie getan?“ – „Gar nichts.“

### "„" + "Gib" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gib mal her!“ sagte Maria und nahm Tom das Fernglas ab.
  - „Gib mir etwas zum Schreiben.“ „Ist das recht?“ „Ja, das ist gut.“
  - „Gib mir noch eine Chance!“ – „Nein, es ist endgültig aus zwischen uns.“

### "„" + "Gut" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gut“, sagte sie.
  - „Versuch, vorsichtiger zu sein!“ – „Gut, ich versuch’s.“
  - „Wer da?“ – „Gut Freund!“

### "„" + "Kein" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Danke für deine Hilfe!“ – „Kein Thema!“
  - „Habt ihr Angst?“ – „Kein bißchen!“
  - „Hörst du es, wenn wir bei uns den Fernseher laufen haben?“ – „Kein bisschen.“

### "„" + "Krieg" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich bin fast fertig mit „Krieg und Frieden“.
  - Ich habe auch nie „Krieg und Frieden“ gelesen.
  - Ich habe „Krieg und Frieden“ auch noch nicht gelesen.

### "„" + "Mama" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir singen einen Monat lang in der Galaxis die „Mama“, und er ist unser.
  - „Mama ist heute das Essen angebrannt“, erzählte Fritzchen seiner Oma.
  - „Mama, wo sind die Wäscheklammern?“ – „Auf dem Tisch.“

### "„" + "Schon" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darf ich kurz stören?“ – „Schon passiert. Was gibt’s denn?“
  - „Ich bedanke mich recht herzlich.“ – „Schon wieder?“
  - „Schon erledigt“, verkündete sie.

### "„" + "Soll" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Soll ich das Fenster zumachen?“ – „Ja, bitte.“
  - „Soll ich den Tee machen?“ – „Ja, mach! Du kannst das besser als ich.“
  - „Soll ich dir ein Bier holen?“ – „Nein, ich trinke besser Mineralwasser.“

### "„" + "Sprichst" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sprichst du Deutsch?“ – „Nein, ich bin Französin.“
  - „Sprichst du Deutsch?“ – „Nein.“
  - „Sprichst du Englisch?“ – „Noch nicht, das ist meine erste Lektion!“

### "„" + "War" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „War beim Abendessen alles in Ordnung, Ma’am?“, fragte sie.
  - „War da eine Frage dabei?“, sagte ich.
  - „War das Darlehen nicht für das Schiff bestimmt?“, fragte Brunel.

### "„" + "Weiß" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin zum Islam übergetreten.“ – „Weiß ich.“
  - „Ist das normal?“ – „Weiß ich nicht.“
  - „Was hast du, Maria?“ – „Weiß nicht.“

### "„" + "Wenn" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darf ich Sie morgen anrufen?“ – „Wenn Sie wollen.“
  - „Hilft Cola gegen Durchfall?“ – „Wenn die Flasche steckenbleibt, ja.“
  - „Kann ich Bedenkzeit haben?“ – „Wenn Sie sich denn noch bedenken müssen.“

### "„" + "Wird" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wird bereits gesattelt“, sagte Victorson.
  - „Wird das Projekt pünktlich fertiggestellt sein?“ – „Das kann ich garantieren.“
  - „Wird er bald wieder auf den Beinen sein?“ – „Ich hoffe es.“

### "„" + "Zum" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Können Sie „Zum Geburtstag viel Glück“ auf französisch singen?
  - Tom sang für Maria „Zum Geburtstag viel Glück“.
  - Tom stimmte „Zum Geburtstag viel Glück!“ an.

### "„" + "Zwei" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was war das?“ – „Zwei Katzen haben sich angefaucht.“
  - „Wie lange wird die Versammlung dauern?“ — „Zwei Stunden.“
  - „Zwei Dutzend?“, fragte Jim.

### """ + "am" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Zwang" am Arsch!
  - Das "Chu Jia" am Hafen.
  - Das große Aquarium "Ocean Park" am Tigerstrand in Dalian ist bereits eröffnet.

### """ + "aus" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Spricht man das nicht "Caleb Dakota" aus?
  - Und wie sieht es bei "Sylvester und Tweety" aus?
  - Weißt du, in Großbritannien spricht man Craig "Creig" aus.

### """ + "gelesen" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat "Das Manifest" gelesen.
  - Hast du nie "Moidodyr" gelesen?!
  - Ich habe das Buch "Die Göttliche Komödie" gelesen.

### """ + "sagst" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich mag, wie du "lecker" sagst.
  - Wenn du "Willkommen" sagst, musst du ihn ansehen.
  - Wenn du noch einmal "Junge" sagst, schlitze ich dich auf, von unten bis oben!

### """ + "sein" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also, Charlie, was soll "rot werden" sein?
  - Es muss im "Van Beek" sein.
  - Ich möchte keine "Couch-Potato" sein.

### """ + "sind" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Extras" sind so geduldig, sitzen und warten.
  - "Fruit of Islam" sind diszipliniert.
  - "Wie heisst du" sind nur drei Worte.

### """ + "—" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bist du Schwede?" — "Nein, ich bin Schweizer."
  - "Hast du deine Aufgabe erledigt?" — "Fast."
  - "Wer ist das?" — "Das ist Jim."

### "." + "„" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ha, ha!“ lachte der Neffe von Scrooge. „Ha, ha, ha!“
  - „Höre!“ rief der Geist. „Meine Zeit ist fast um.“
  - „Tu’s nicht, Anne!“ flehte Diana. „Du fällst herunter und stürzt dich zu Tode!“

### "Ein" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein "A".
  - Ein "Ding".
  - Ein "Volvo" steht dort.

### "Freund" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das ist ein hebräisches Wort, und es bedeutet „Freund“.
  - Keith hatte ich als meinen „guten Freund“ dabei.
  - „Auf den abwesenden Freund“, erwiderten die anderen.

### "Hm" + "…" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hm … Eingeladen?
  - Hm … Metalleinband.
  - Hm … das Tagebuch ist im abgesperrten Bereich.

### "Ich" + "…" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich … die Dinge, die wir auf keinen Fall kaufen.
  - Ich … helf eurer Freundin.
  - „Ich …“ Ich biss mir auf die Zunge.

### "Im" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Im "Deux Moulins", ich weiß.
  - Im "Grandhotel London".
  - Im "Headquarters"!

### "Namen" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle unter dem Namen "Emily Rugburn".
  - Die Einheimischen verpassten ihnen den Namen "Die Baek Lady Killers".
  - Dr. Baker, Sie haben sicher den Namen "de Winter" in der Zeitung gelesen?

### "Nicht" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nicht "Cousin Hubert"?
  - Nicht "Praveen", "Parveen".
  - Nicht "angeblich", Dr. Jones.

### "Problem" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Daran gibt es nur ein kleines Problem“, sagte Plaitonos Euripideson.
  - „Ich habe ein Problem“, sagte er.
  - „Ich sehe kein Problem“, sagte ich.

### "Spencerport" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „In Spencerport“, warf Kat ein.
  - „Man sieht nicht viele Leute aus Spencerport“, sagte Piotr.
  - „Spencerport“ stand eindeutig nicht auf der Liste.

### "Tatoeba" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du schon einmal etwas von „Tatoeba“ gehört?
  - Was heißt „Tatoeba“?
  - „Tatoeba“ bedeutet auf Japanisch „zum Beispiel“.

### "Versuch" + "'" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Versuch's mal ohne mich.
  - Versuch's mal, Arturo.
  - Versuch's mal.

### "Vor-" + "und" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle Schriftsysteme haben Vor- und Nachteile.
  - Diese Prozedur hat Vor- und Nachteile.
  - Es gibt immer Vor- und Nachteile.

### "an" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich weiß nicht, ob dieser Lichtschalter auf „an“ oder „aus“ steht.
  - „Der Fischverkauf zieht an“, verkündete Dietrich.
  - „Sternfahrer tun den Menschen manchmal schreckliche Dinge an“, sagte sie.

### "auch" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich erzähle den Leuten auch“, fuhr Clyde fort, „was ich hier gehört habe.
  - „Das sollte er auch“, sagte ich.
  - „Diese beiden auch“, sagte Keith.

### "des" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich bin die Geliebte des "Antillais".
  - Matías hatte den Anführer des "Comando Vermelho" getötet.
  - Vorsicht bei der Abfahrt des "Waterford Huskie Express"!

### "dir" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gefällt dir "Marathon im Herbst"?
  - Leider muss ich dir "Lebe wohl" sagen.
  - Schaust du dir "Tom und Jerry" an?

### "für" + "'n" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was für'n Tag ist heute?
  - Was hast'n du für'n Traum?
  - Was ist jetzt wieder für'n Scheiß, Reznik?

### "nein" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ach nein“, sagte Lee.
  - „Nun, nein“, antwortete Nicholas.
  - „Oh nein“, antwortete sie.

### "noch" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Führer soll heute noch "Clausewitz" ausgeben.
  - Schreib noch "Jude" dazu.
  - Solang ich noch "Lebwohl" sagen kann.

### "recht" + "“" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die junge Dame hat recht“, erklärte Maurice.
  - „Du hast recht“, sagte Dietrich.
  - „Ja, sie hat recht“, warf Jill ein.

### "sagten" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie sagten "Nebenwirkungen".
  - Sie sagten "Waffenkammer".
  - Sie sagten "Wurm".

### "vom" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du lässt mich aussehen wie die Irre vom "Aquarius".
  - Ich rede vom "Ranger Assessment Selection Program".
  - Judy Brown vom "Portobello Express".

### "wie" + "'n" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Collignon, dumm wie'n Floh!
  - Collignon, dumm wie'n Floh.
  - Du siehst wie'n Stück Scheiße aus.

### "wird" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das wird "Clinch" genannt.
  - Es wird "Hastings" ausgesprochen.
  - Guinea wird "Guinée" auf Französisch genannt.

### "zu" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Angenommen, ich ginge zu "Denny's".
  - Der muss zu "Saturday Night Live".
  - Er hat einen Link zu "Mimas Zimmer" gelegt.

### "»" + "Aber" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Aber meine Fische warten auf mich«, sagte ich.
  - »Aber was wird aus meinem Geschäft?«, fragte er.
  - »Aber wir haben ihnen doch das ganze Geld gegeben …« Meine Stimme verebbte.

### "»" + "Ja" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ja!«
  - »Ja, natürlich ist das Loricum.«
  - »Ja.« Laneson sah nicht aus, als erwarte er Geld.

### "»" + "Und" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Und du brauchst auch einen Autodoc.«
  - »Und jetzt habe ich eine Bitte«, verkündete Michael.
  - »Und jetzt?« fragte sich Gregor und sah sich im Dunkeln um.

### "»" + "Wenn" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Wenn die Tür sich öffnet, starte die Sanduhr.«
  - »Wenn es keinen Käufer gibt, gibt es kein Geld, um irgendjemanden auszuzahlen.«
  - »Wenn ich jetzt zahle, ist die Sache dann ganz sicher erledigt?«

### "—" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bist du Schwede?" — "Nein, ich bin Schweizer."
  - "Hast du deine Aufgabe erledigt?" — "Fast."
  - "Wer ist das?" — "Das ist Jim."

### "“" + "Ich" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ein …“ Ich biss mir auf die Zunge und dachte sehr, sehr schnell nach.
  - „Ich …“ Ich biss mir auf die Zunge.
  - „Ja!“ Ich strahlte und deutete darauf.

### "“" + "heißt" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Wort „chat“ heißt auf französisch „Katze“.
  - „Arktos“ heißt auf Griechisch Bär.
  - „Hai“ heißt auf Lettisch „haizivs“.

### "„" + "Ahoi" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ahoi!“, rief Rudolph dem Mann zu, der am Ende des Piers angelte.
  - „Ahoi, an Deck!“
  - „Ahoi, ihr da bei den Zelten!“

### "„" + "Dafür" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dafür gibt es doch Anleger“, antwortete er.
  - „Dafür gibt es nicht viel Nachfrage“, antwortete er.
  - „Dafür sind Freunde doch da.“

### "„" + "Dieses" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dieses Boot hat Segel“, sagte Nicholas.
  - „Dieses Handtuch gefällt mir wirklich sehr.“ – „Behalt’s!“ – „Oh, danke schön!“
  - „Dieses Lied geht mir nicht mehr aus dem Kopf.“ – „Welches?“

### "„" + "Einspruch" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einspruch!“ – „Abgelehnt!“
  - „Einspruch!“ — „Abgelehnt!“
  - „Ich erhebe Einspruch!“ – „Einspruch abgelehnt!“

### "„" + "Fast" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fast fünfzehn Teile auf zehn investierte“, antwortete er.
  - „Fast jedes“, antwortete er.
  - „Fast vier Pfund“, sagt Bob.

### "„" + "Ganz" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ganz richtig“, sagte ich.
  - „Ganz und gar nicht“, sagte ich.
  - „Tom.“ „John.“ „Ist mir eine Freude.“ „Ganz meinerseits.“

### "„" + "Gefällt" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Für dieses Bild haben viele mit „Gefällt mir!“ gestimmt.
  - Verpasst meiner Seite auf Facebook doch bitte ein „Gefällt-Mir“.
  - Wir würden uns über ein „Gefällt mir!“ bei Facebook freuen!

### "„" + "Gern" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Danke für deine Hilfe!“ – „Gern geschehen!“
  - „Danke für die Hilfe!“ – „Gern geschehen!“
  - „Herzlichen Dank für alles.“ – „Gern geschehen.“

### "„" + "Gerne" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Danke für Ihre Hilfe!“ – „Gerne!“
  - „Danke für deine Hilfe!“ – „Gerne!“
  - „Danke für eure Hilfe!“ – „Gerne!“

### "„" + "Gott" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gott ist in seinem Himmel, mit der Welt ist alles gut“, flüsterte Anne leise.
  - „Gott segne Sie!“ – „Ich bin Atheist.“
  - „Gott segne dich!“ – „Ich bin Atheist.“

### "„" + "Habe" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Habe ich das wirklich getan?“ – „Wissen Sie das denn nicht mehr?“
  - „Habe ich dich geweckt?“ – „Ja, das hast du!“
  - „Habe ich dich schon mal belogen?“ – „Ja, schon mehrmals.“

### "„" + "Kaffee" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Guten Morgen!“ – „Morgen!“ – „Wie hast du geschlafen?“ – „Kaffee!“
  - „Kaffee, Tee, Cognac, Whisky?“ – „Ein Glas Wasser.“
  - „Kaffee?“ – „Instant?“ – „Ja.“ – „Dann nicht.“

### "„" + "Können" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Können Sie Französisch?“ – „Nein.“
  - „Können Sie mir den Zucker reichen?” „Bitte sehr.”
  - „Können wir ein Gespräch führen?“ – „Du kannst mich mal!“

### "„" + "Maria" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Maria weiß nichts von meinen Gefühlen.“ – „Bist du dir dessen sicher?“
  - „Maria, ich liebe dich!“ – „Ich weiß.“
  - „Maria, ist das wirklich passiert oder nur in deiner Phantasiewelt?“ fragte Tom.

### "„" + "Morgen" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Guten Morgen!“ – „Morgen!“ – „Wie hast du geschlafen?“ – „Kaffee!“
  - „Morgen früh“, sagte ich.
  - „Morgen gibt's bei uns eine Party.“ – „Und wer kommt?“

### "„" + "Tatoeba" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du schon einmal etwas von „Tatoeba“ gehört?
  - Was heißt „Tatoeba“?
  - „Tatoeba“ bedeutet auf Japanisch „zum Beispiel“.

### "„" + "Vater" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lasst uns „Vater, Mutter, Kind“ spielen!
  - Wir spielten oft „Vater, Mutter, Kind“ im Park.
  - „Vater, das beste, was ich mitgebracht habe, ist das Tischchen.“

### "„" + "Vielleicht" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vielleicht kommt ja immer noch irgendwann der Tag“, erwiderte er.
  - „Vielleicht noch ein Zweimaster?“, fragte er.
  - „Vielleicht trägt sie den Makel der Sterne“, sagte Lanceson düster.

### "„" + "Warst" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gestern habe ich Tom geküsst.“ – „Warst du da betrunken?“
  - „Warst du bei Marias Hochzeit?“ – „Ja, ich war ein Zaungast.“
  - „Warst du das?“ – „Nein, Mama, das war ich nicht.“

### "„" + "Weißt" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weißt du Bescheid?“ – „Nein. Worum geht es eigentlich?“
  - „Weißt du, was heute für ein Tag ist?“ — „Unser Hochzeitstag!“
  - „Weißt du, wo Lappland ist?“, fragte sie das Rentier.

### "„" + "Wieso" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bist du mir böse?“ – „Wieso sollte ich dir böse sein?“
  - „Räume bitte dein Zimmer auf!“ „Wieso denn, es ist doch eh ganz sauber!“
  - „Wieso bist du nicht gekommen?“ „Ich hatte keine Wahl.“

### "„" + "Wirst" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wirst du das für mich tun, Tom?“ – „Mit Wonne, mein lieber Schatz!“
  - „Wirst du heute Nachmittag lernen?“ „Ja.“
  - „Wirst du ihn heiraten?“ – „Nein.“

### "„" + "Zu" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ihr heiratet besser nicht.“ – „Zu spät! Wir sind bereits verheiratet.“
  - „Wie bist du gekommen?“ „Zu Fuß.“
  - „Wie seid ihr gekommen?“ „Zu Fuß.“

### "„" + "die" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ach nein,“ antworteten sie, „die Bedingungen sind zu schwer.
  - O,“ antwortete der Sohn, „die ist so satt, sie mag kein Blatt.
  - Zorin“, sagte ich, „die Rechnung und ein Trinkgeld für Margrethe.

### "„" + "du" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich heiße nicht „du“, sondern Ricardo.
  - James“, sagte ich, „du gehst direkt zur Akademie.
  - Jill“, sagte ich, „du bist mit Marika befreundet, nicht wahr?

### "„" + "wie" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kind,“ sprach die Alte, „wie du aussiehst!
  - Neue Erkenntnisse wie „wie man ein Raumschiff baut“?
  - Sie waren aber freundlich und fragten „wie heißt du?“

### """ + "der" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eddie Murphy war in "Shrek" der Esel.
  - In der Familie "Elektriker" der Großvater.
  - Wenn es ein Verb namens "nachen" gäbe, wäre "Nacht!" der Imperativ davon.

### """ + "die" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann bricht er sich bei "Shiloh Ranch" die Schulter, dann bin ich's geworden.
  - In der Familie "Kameramann" die Mutter.
  - Ist "Rank" die Abkürzung für "Franklin"?

### """ + "drauf" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da sind überall Folgen von "West Wing" drauf.
  - Da steht "Sarah Grant" drauf.
  - Ich sagte, schreiben Sie "Bahnhof" drauf.

### """ + "fragte" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Kennen Sie die internationale Vorwahl für den Libanon?" fragte Dima.
  - "Warst nicht du dort?" fragte Herr Jordan.
  - "Was ist los mit dir?" fragte sie.

### """ + "gesehen" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Habt ihr die Fernsehwerbung für das "Kleine Paris im Osten" gesehen?
  - Hat gesagt, er kommt, wenn er "seine Paro" gesehen hat.
  - Ja, das Boa-Vista-Kino, in dem ich "Der weiße Hai" gesehen habe.

### """ + "höchstpersönlich" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Fredy Krüger" höchstpersönlich.
  - Der Regisseur von "Rosemaries Baby" höchstpersönlich.
  - Na, wenn das nicht "Die Kobra" höchstpersönlich ist.

### """ + "im" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Damit ist "Double Bind" im Kasten.
  - Ich habe "Apocalypse Now" im Kino gesehen.
  - Steht "gewandt" im Wörterbuch?

### """ + "kann" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Rückwärts" kann ich nicht.
  - "Stehen" kann er gut.
  - Mit "A" kann ja jeder schreiben.

### """ + "mehr" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt kein "wir" mehr, Madame Després.
  - Es tut mir leid, wir bieten keine "Entschuldigungs-Dienste" mehr an.
  - Was "musst du" mehr als Familie?.

### """ + "wie" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Kyo" wie "Tokyo".
  - War es "T" wie ein Trottel, der eine wichtige Nachricht verliert?
  - Weshalb benutzt du "romantisch" wie ein Schimpfwort?

### """ + "wird" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "B" wird gesucht, Gruppe B!
  - "Etwas" wird nicht genug sein.
  - Das Verb "sein" wird in allen Sprachen unregelmäßig konjugiert.

### "-" + "Ei" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ei- Ei- Ei- Ei!
  - Ei- Ei- Ei- Ei!
  - Ei- Ei- Ei- Ei!

### "-" + "das" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die ewige Stadt - das ist Rom; und die immerwährende Baustelle - das ist Berlin.
  - Die ewige Stadt - das ist Rom; und die immerwährende Baustelle - das ist Berlin.
  - Was kam zu erst - das Ei oder das Huhn?

### "-" + "die" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es wurde ein neues Kommunikationsmittel entwickelt - die Eisenbahn.
  - Ich habe dich - die ganze Zeit gesucht.
  - Ich habe die Wohnung verkauft, - die sie geerbt hat.

### "-" + "ich" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe gesagt, - ich glaube dir.
  - Ich-- ich habe keinen.
  - Nun gut, aber- ich verstehe es noch nicht richtig.

### "-" + "ist" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für den Vater, Abou Tarek, - ist eine Sterbeurkunde nicht aufzufinden.
  - Ich dachte, der Ring - ist vielleicht fertig.
  - Sich so entspannt zu unterhalten wie mit Ihnen, - ist ein seltenes Vergnügen.

### "Babino" + "t" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Babinot, Beeilung!
  - Babinot, die Ferien sind vorbei.
  - Babinot, was soll das?

### "City" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bist du für „City“ oder die „Rovers“?
  - „City“ hat gegen „United“ verloren.
  - „City“ ist gegen „United“ früh in Führung gegangen.

### "Den" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Den "Nagetieren Ungewöhnlicher Größe"?
  - Den "Sommer".
  - Den "Starbury One".

### "Export" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „Import“ oder „Export“ gesagt?
  - Habt ihr „Import“ oder „Export“ gesagt?
  - Hast du „Import“ oder „Export“ gesagt?

### "Facebook" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich halte zu meinen Freunden und Verwandten über „Facebook“ Kontakt.
  - Wie viele Freunde hast du bei „Facebook“?
  - Wie viele „Facebook“-Freunde hast du?

### "Hause" + "!" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bringe ihn sofort nach Hause !
  - Du bleibst zu Hause !
  - Geht nach Hause !

### "Herrin" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Herrin“, keuchte James Victorson.
  - „Ich bin die Leibwache der Eignerin-Herrin“, sagte er.
  - „Ich bin wie geplant hinausgegangen, Herrin“, sagte Chiang.

### "Ihre" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ihre "Feuerwehrübung!"
  - Ihre "wissenschaftliche" Theorie.
  - Sie müssen Ihre "Oyster card" aufladen.

### "Import" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „Import“ oder „Export“ gesagt?
  - Habt ihr „Import“ oder „Export“ gesagt?
  - Hast du „Import“ oder „Export“ gesagt?

### "Julia" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Habt ihr „Romeo und Julia“ gelesen?
  - Hast du „Romeo und Julia“ gelesen?
  - Wer hat „Romeo und Julia“ geschrieben?

### "Kapitänin" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bist Kapitänin“, sagte Clorinda.
  - „Guten Abend, Kapitänin“, stammelte er.
  - „Hundert, Kapitänin“, beendete Henrison die Zählung.

### "Korrek" + "t" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Korrekt von euch.
  - Korrekt!
  - Korrekt.

### "Long" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Long'er, du bist bald am Ziel.
  - Long'er, reicht das Haus der Xu aus, um Fuguis Schulden zu begleichen?
  - Long'er, was ist mit Ihrer Truppe?

### "Maul-" + "und" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der hat die Maul- und Klauenseuche.
  - Die Maul- und Klauenseuche ist hochansteckend.
  - Es geht nicht um Maul- und Klauenseuche.

### "Pferd" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Pferd“ lautet „Pferde“.
  - Die Mehrzahl von „Pferd“ lautet „Pferde“.
  - „Pferde“ ist der Plural von „Pferd“.

### "Pferde" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Pferd“ lautet „Pferde“.
  - Die Mehrzahl von „Pferd“ lautet „Pferde“.
  - „Pferde“ ist der Plural von „Pferd“.

### "Schild" + "e" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Schilde hoch!
  - Schilde hoch.
  - Schilde zusammen!

### "So" + "'n" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - So'n Mist, hier geht's auch nicht weiter.
  - So'n Quatsch.
  - So'n bescheuerter Vogel namens Errol.

### "Titel" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist ihr Titel "Transmission".
  - Ein Programm mit dem Titel "Die Heldin von morgen".
  - Eines Tages mach ich einen Film mit dem Titel "Immer Liebeskummer".

### "Tom" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Satz enthält zwei Subjekte, nämlich „Tom“ und „Mary“.
  - Wir ermitteln in der Mordsache „Tom“.
  - „Wer sind die Leute da?“ – „Das ist ein Spiegel, Tom“, sagte Maria.

### "Verstehe" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verstehe“, antwortete er.
  - „Verstehe“, brachte ich hervor.
  - „Verstehe“, sagte ich.

### "Wie" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie "Franklin Delano Roosevelt" oder so was?
  - Wie "Pocholo"?
  - Wie "was"?

### "Wochen" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „drei Wochen“ oder „drei Monate“ gesagt?
  - Habt ihr „drei Wochen“ oder „drei Monate“ gesagt?
  - Hast du „drei Wochen“ oder „drei Monate“ gesagt?

### "auf" + "!" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich gebe auf !
  - Mach auf !
  - Steh auf !

### "bin" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist leicht, „ich bin“ in jede Sprache zu übersetzen.
  - „Sie fürchten mich, weil ich hässlich bin“, sagte er.
  - „Vielleicht, wenn ich das nächste Mal hier bin“, antwortete ich.

### "can" + "'t" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Even if we can't go to Sylt.
  - That can't be correct!
  - That can't be correct.

### "dann" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Erst ging sie nicht ran, dann "Teilnehmer ist nicht zu erreichen."
  - Und dann "Warum?".
  - Wenn ich dann "Eye of the Tiger" singe, verliert ihr eure Hemmungen.

### "dieses" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Doch dieses "Morgen" kam für die Prinzessin und Sutemaru niemals.
  - Was bedeutet dieses "Junior"?
  - Was soll dieses "mm-mm"?

### "dir" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist sehr großzügig von dir“, sagte er.
  - „Das war sehr aufmerksam von dir“, erwiderte ich.
  - „Ich habe Geld bei dir“, sagte ich.

### "drei-" + "oder" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich gehe drei- oder viermal im Monat ins Kino.
  - Tom spielt drei- oder viermal im Monat Golf.
  - Wir verbrauchen drei- oder viermal so viele Kalorien.

### "ein" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich stelle Leute ein“, sagte ich, „und bezahle sie.
  - „Die ziehe ich ein“, sagte er.
  - „Mir fallen mehrere Möglichkeiten ein“, antwortete der Bibliothekar.

### "einmal" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er ging nach Hause und sagte noch nicht einmal "Auf Wiedersehen".
  - Mit einmal "Aufzeichnen & Melden".
  - Wenn du noch einmal "Junge" sagst, schlitze ich dich auf, von unten bis oben!

### "es" + "…" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich muss es … einfach beweisen.
  - Okay, es … es ist mir egal, dass wir unterschiedlich sind.
  - Zu der Zeit war es … eine Show zur Geschlechtsidentität.

### "fragen" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Also werden wir euch zuerst nach eurer Vergangenheit fragen“, sagte ich.
  - „Das musst du ihn fragen“, sagte Piotr.
  - „Ich muss meine Klienten fragen“, sagte er.

### "gehen" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ein grünes Licht bedeutet „gehen“.
  - „Es ist ganz sicher Zeit, dass wir gehen“, sagte ich zu ihm.
  - „Ich werde wieder in die Schule gehen“, verkündete sie.

### "gesagt" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Besser gesagt "bis morgen".
  - Hat der grade gesagt "ein solider Fund"?
  - Hat gesagt "Beatrice Russo".

### "gesehen" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das habe ich gesehen“, sagte ich lächelnd.
  - „Ich habe sie nicht gesehen“, sagte Sparhawk.
  - „Ich habe sie vor fünf Tagen gesehen“, sagte er.

### "hast" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hast "Black Irish"?
  - Du hast "Jeffrey" gesagt.
  - Du hast "Pech" zu ihm gesagt.

### "heisst" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was heisst "Wieso nicht".
  - Was heisst "ich weiss es am besten"?
  - Was heisst "nichts"?

### "heißen" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber das sind doch keine Nikes, die heißen "Nice".
  - Die heißen "Kachori".
  - Was soll das heißen "Nein, danke"?

### "hieß" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein Freund von mir hieß "Hinterlader".
  - Er hieß "Der Ray Harris".
  - Es hieß "Reisen".

### "kann" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Typ nimmt, was er kriegen kann“, sagte Maria.
  - „Ich fange, was ich kann“, antwortete ich.
  - „Weil man sie beim Einkaufen mitnehmen kann“, antwortete er.

### "m" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Aus irgendeinem Grund habe ich „m“ gelesen und nicht „n“.
  - Dann tippte ich auf das „m“.
  - Es erklang ein kaum hörbares Klicken, und hinter dem „a“ erschien ein „m“.

### "machen" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn wir es noch einmal machen“, sagte Kat, „sollten wir es besser angehen.
  - „Das könnte ich machen“, sagte ich.
  - „Das lässt sich leicht machen“, sagte Vincent.

### "mehr" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da kann ich nicht mehr "nein" sagen!
  - Noch mehr "gute" Nachrichten?
  - Nur heißt es nicht mehr "Liebe", sondern "Weihnachten".

### "mir" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hast mir "aufwachen" ins Ohr gebrüllt.
  - Endlich scheint mir "Meine Ehefrau Pamela" auf dem richtigen Weg zu sein.
  - Heute muss ich mir "FBI" ansehen.

### "nach" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cut nach "sind wir uns einig".
  - Es riecht hier nach "Grüner Apfel".
  - Kommt donnerstags nach "Thomas".

### "namens" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er fiel im Krieg und hinterließ ihr ein Schwert namens "Fliegender Schnee".
  - Ich traf eine sehr nette junge Dame namens "Granate".
  - Wenn es ein Verb namens "nachen" gäbe, wäre "Nacht!" der Imperativ davon.

### "sag" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja, Naina, sag's ihm.
  - Ja, sag's mir.
  - Komm, sag's mir.

### "sage" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Geh rüber und sage "Idiot" zu ihm.
  - Ich sage "du kannst es", aber ich habe dir nie etwas beigebracht.
  - Ich sage "sexy" einfach nicht mehr.

### "schon" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Da war ich schon“, sagte ich.
  - „Das kennen wir schon“, sagte Vincent.
  - „Ich verstehe schon“, sagte ich.

### "sei" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er trank keinen einzigen Schluck, und doch sagten alle PK sei "blau".
  - Er trank nie einen Schluck und doch sagten alle PK sei "blau".
  - Herr Kongming meint, deine Formation sei "etwas zu überholt".

### "sich" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es nennt sich "Weltenspringen".
  - Ich weiß nur, dass er sich "Asterix" nennt.
  - Wir suchen einen Freund, der sich "Asterix" nennt.

### "sie" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.
  - „Eigentlich sie“, korrigierte Keith.
  - „Ja, das war sie“, sagte Williamson.

### "soll" + "'n" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was soll'n der Scheiß jetzt wieder heißen?
  - Was soll'n wir mit dem Verräter machen?
  - Wie soll'n das geh'n?

### "sprechen" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber ich kann mit meinem Rechtsberater sprechen“, fuhr ich fort.
  - „Ich wollte ohnehin mit dir darüber sprechen“, sagte sie.
  - „Wir müssen direkt mit ein paar Bauern sprechen“, sagte Kat.

### "tun" + "“" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bist eine junge Frau und hast zweifellos anderes zu tun“, fuhr sie fort.
  - „Ich will’s tun“, sagte die kleine Meerjungfrau totenbleich.
  - „Ja, ich hielt immer den Mund und ließ dich das Richtige tun“, antwortete ich.

### "wollte" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er wollte "rot werden", bevor wir gehen.
  - Ich schwör's dir, ich wollte "starkes Ego" sagen.
  - Sie wollte "Der Blick auf Qin Huai" spielen, und deshalb musste sie sterben.

### "«" + "Ich" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Das hat ihn gerettet.« Ich musste niesen.
  - »Jetzt.« Ich öffnete meinen Geldbeutel und begann zu zählen.
  - »Waren, die wir transportieren.« Ich hatte inzwischen Konkurrenz.

### "«" + "fragte" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Nun?« fragte Herr Samsa.
  - »Und jetzt?« fragte sich Gregor und sah sich im Dunkeln um.
  - »Wo ist das Frühstück?« fragte der mittlere der Herren mürrisch die Bedienerin.

### "»" + "Die" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Die Armbrüste sind nicht gespannt«, antwortete ich.
  - »Die Bellers haben eine ganze Reihe meiner Freunde umgebracht«, sagte Rudolph.
  - »Die erste Ladung kommt aus dem Ofen, bevor du losmusst«, verkündete sie.

### "»" + "Sie" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Sie haben nach mir geschickt, Hafenmeister?«, fragte ich.
  - »Sie und Ihr Rechtsberater müssen sich darum kümmern«, stimmte Laneson zu.
  - »Sie wurden im Voraus bezahlt«, unterbrach ich ihn.

### "»" + "Wo" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Wo immer Sie auch sind.«
  - »Wo ist das Frühstück?« fragte der mittlere der Herren mürrisch die Bedienerin.
  - »Wo war das Hafen-Traktornetz?«, fragte sie.

### "“" + "Er" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Dann kannst du dich von dem ablenken, was nicht passiert ist.“ Er nickte.
  - „Die Stadt hat also sechs weitere Reader an Bord?“ Er nickte.
  - „Mal sehen.“ Er knabberte an einem Scone und nickte zufrieden.

### "“" + "bezeichnet" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Mafia wird auch als die „Cosa Nostra“ bezeichnet.
  - Mein Freund hat mich als „Meister der durchgemachten Nächte“ bezeichnet.
  - Mein Freundin hat mich als „Meisterin der durchgemachten Nächte“ bezeichnet.

### "“" + "ein" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In der heutigen Zeit ist „variabel“ ein Schimpfwort.
  - Maria hat bei „Pinterest“ ein Bild von ihren Keksen hochgeladen.
  - „Michael“ ist ein männlicher, aber „Michaela“ ein weiblicher Vorname.

### "“" + "einen" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die „Löwen“ haben gegen die „Falken“ einen leichten Sieg errungen.
  - Kann man in „Kalles Speisewagen“ einen Tisch reservieren?
  - Tom hat sich über „Open Table“ einen Tisch in dem Restaurant reservieren lassen.

### "“" + "hat" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die „Flammenatmung“ hat eine lange Geschichte.
  - „City“ hat gegen „United“ verloren.
  - „Kalles Speisewagen“ hat dichtgemacht.

### "“" + "kommt" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Wort „Astronaut“ kommt aus dem Lateinischen und bedeutet „Sternenschiffer“.
  - Das Wort „Golem“ kommt aus dem Hebräischen.
  - Das Wort „ja“ kommt ihm nie über die Lippen.

### "“" + "nennen" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist das, was wir ein „otoshidama“ nennen.
  - Sie wollte ihn „Ehrliche Amandas Universalwarenhaus“ nennen.
  - „Soccer“ nennen die Europäer „football“.

### "”" + "„" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Können Sie mir den Zucker reichen?” „Bitte sehr.”
  - „Welchen Namen bevorzugst du?” „Sie sind beide schön.”
  - „Würden Sie mir bitte den Zucker geben?” „Hier, bitte.”

### "„" + "Am" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Am Sonnabend kann ich nicht.“ – „Wie denn das? Du hast es doch versprochen.“
  - „Bis wann brauchen Sie die Informationen?“ – „Am besten bis gestern.“
  - „Wann sagen wir es Tom am besten?“ – „Am besten jetzt gleich.“

### "„" + "An" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Glaubst du noch?“ – „An Gott? Natürlich!“
  - „Wohin gehen wir?“ – „An einen sicheren Ort.“
  - „Woran denken Sie?“ – „An die Mittagspause.“

### "„" + "Aus" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aus reinem Kupfer gewebt wäre sie billiger“, stöhnte Yu.
  - „Darf ich dich fragen, wo du herkommst?“ – „Aus Bologna.“
  - „Woher kommst du?“ – „Aus Algier.“ – „Ach, du bist Algerierin.“

### "„" + "Bin" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bin gerade zurückgekommen“, antwortete ich.
  - „Bin ich hier ganz allein?“ – „Ja, Sir.“
  - „Warum bist du mir böse?“ – „Bin ich doch gar nicht!“

### "„" + "Boote" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Boote draußen!“, rief Paulson.
  - „Boote zu den Steuerbordbojen, Leinen festsetzen!“, rief ich.
  - „Boote, Bugleinen ausbringen.“

### "„" + "Chris" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Chris Markofferson hat sein Geld nicht hier aufbewahrt.“ Auch das war Yus Werk.
  - „Chris Markofferson“, antwortete ich.
  - „Chris schuldet mir mein Geld“, sagte Williamson noch einmal.

### "„" + "City" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bist du für „City“ oder die „Rovers“?
  - „City“ hat gegen „United“ verloren.
  - „City“ ist gegen „United“ früh in Führung gegangen.

### "„" + "Darum" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darum ging es also bei den Scherzen auf der Hochzeit“, sagte ich.
  - „Darum ist es ja ein Schnäppchenfest“, antwortete er.
  - „Warum?“ – „Darum!“

### "„" + "Dass" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dass ich dich besser hören kann.“
  - „Dass ich dich besser packen kann.“
  - „Dass ich dich besser sehen kann.“

### "„" + "Deine" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Deine Mannschaft hat schon wieder verloren!“ – „Egal. Ich trag’s mit Fassung.“
  - „Deine Welt ist nicht wie meine“, sagte Arthur.
  - „Wo sind meine Zeitungen?“ – „Deine Zeitungen liegen auf dem Tisch.“

### "„" + "Drei" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ist es bis zur Grenze noch weit?“ – „Drei Kilometer.“
  - „Wie viel Uhr ist es jetzt?“ „Drei Uhr zwanzig.“
  - „Wie viele Kürbisse willst du?“ – „Drei, wenn’s beliebt.“

### "„" + "Dreißig" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie lange haben Sie dafür gebraucht?“ – „Dreißig Minuten.“
  - „Wie lange habt ihr dafür gebraucht?“ – „Dreißig Minuten.“
  - „Wie lange hast du dafür gebraucht?“ – „Dreißig Minuten.“

### "„" + "Dürfen" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dürfen wir am Anleger festmachen?“
  - „Dürfen wir dir einige Fragen stellen?“ – „Schieß los!“
  - „Dürfen wir dir einige Fragen stellen?“ – „Schießt los!“

### "„" + "Ei" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ei, Großmutter, was hast du für große Augen!“
  - „Ei, Großmutter, was hast du für große Hände!“
  - „Ei, Großmutter, was hast du für große Ohren!“

### "„" + "Export" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „Import“ oder „Export“ gesagt?
  - Habt ihr „Import“ oder „Export“ gesagt?
  - Hast du „Import“ oder „Export“ gesagt?

### "„" + "Facebook" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich halte zu meinen Freunden und Verwandten über „Facebook“ Kontakt.
  - Wie viele Freunde hast du bei „Facebook“?
  - Wie viele „Facebook“-Freunde hast du?

### "„" + "Geld" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „Geld“ oder „Gold“ gesagt?
  - „Geld her, oder es passiert was!“ drohte der Räuber.
  - „Geld“, antwortete sie.

### "„" + "Glaubst" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Frag Tom, wo Maria ist!“ – „Glaubst du, der weiß das?“
  - „Glaubst du an Gott, mein Sohn?“ – „Ja, mein Vater.“
  - „Glaubst du noch?“ – „An Gott? Natürlich!“

### "„" + "Ha" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ha! Ich war schneller als du!“ – „Du Frechdachs!“
  - „Ha, ha!“ lachte der Neffe von Scrooge. „Ha, ha, ha!“
  - „Ha, ha!“ lachte der Neffe von Scrooge. „Ha, ha, ha!“

### "„" + "Heute" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gibt es heute keinen Nachtisch?“ – „Heute nicht. Iss einen Apfel!“
  - „Heute ist doch Dienstag, oder?“ – „Ja.“ – „Warum ist es nur so voll?“
  - „Heute oder morgen?“ – „Ob heute oder morgen, das ist mir ganz egal.“

### "„" + "Import" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie „Import“ oder „Export“ gesagt?
  - Habt ihr „Import“ oder „Export“ gesagt?
  - Hast du „Import“ oder „Export“ gesagt?

### "„" + "Jawohl" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Jawohl!“ erwiderte Maria entschlossen.
  - „Jawohl, Ma’am.“
  - „Sei in Zukunft vorsichtiger!“ – „Jawohl.“ – „Nun gut, du kannst gehen.“

### "„" + "Jeder" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es heißt „Jeder für sich!“.
  - „Jeder hat Fehler.“ – „Ich nicht.“
  - „Jeder macht Fehler.“ – „Besonders Vollidioten!“

### "„" + "Kennst" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kennst du Tom schon länger?“ – „Nein, ich habe ihn eben erst kennengelernt.“
  - „Kennst du Tom?“ – „Ich weiß von ihm.“
  - „Kennst du ihn wirklich nicht?“ — „Keineswegs!“

### "„" + "Kommt" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kommt er auch?“ – „Ich hoffe es.“
  - „Kommt er?“ — „Nein, ich glaube nicht.“
  - „Kommt vor“, sagte ich.

### "„" + "Könnten" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Könnten Sie wohl das Fenster öffnen?“ – „Ja, natürlich.“
  - „Könnten wir deinen UniComp benutzen, Nicholas?“, fragte Vincent.
  - „Könnten wir zu den Verrätern zurückkehren?“, fragte Saunders.

### "„" + "Könntest" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Könntest du den Leuten erklären, wie man ein Segelschiff baut?“
  - „Könntest du mir den Zucker reichen?“ – „Hier, bitte!“
  - „Könntest du mir mit meinen Englischhausaufgaben helfen?“ – „Natürlich.“

### "„" + "Liebe" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was heißt „Liebe“ auf Französisch?
  - „Liebe“ und „Hass“ sind Gegensätze.
  - „Philosophie“ ist ein griechisches Wort, das „Liebe zur Weisheit“ bedeutet.

### "„" + "Liebst" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Liebst du mich?“ – „Aber natürlich liebe ich dich!“
  - „Liebst du mich?“ – „Natürlich!“
  - „Liebst du mich?“ – „Über alles!“

### "„" + "Möchten" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Möchten Sie eine Nachricht hinterlassen?“ – „Nein, danke.“
  - „Möchten Sie etwas zu trinken haben?“ – „Ich muss dankend ablehnen.“
  - „Möchten Sie noch ein Stück Kuchen?“ „Ja, gerne!“

### "„" + "O" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das weißt du doch nicht.“ – „O doch!“
  - „O Tannenbaum“ ist ein traditionelles deutsches Volkslied.
  - „O,“ antwortete der Sohn, „die ist so satt, sie mag kein Blatt.“

### "„" + "Pferd" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Pferd“ lautet „Pferde“.
  - Die Mehrzahl von „Pferd“ lautet „Pferde“.
  - „Pferde“ ist der Plural von „Pferd“.

### "„" + "Pferde" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Pferd“ lautet „Pferde“.
  - Die Mehrzahl von „Pferd“ lautet „Pferde“.
  - „Pferde“ ist der Plural von „Pferd“.

### "„" + "Romeo" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Habt ihr „Romeo und Julia“ gelesen?
  - Hast du „Romeo und Julia“ gelesen?
  - Wer hat „Romeo und Julia“ geschrieben?

### "„" + "Sag" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sag ihm bitte nichts!“ – „Ich werde schweigen!“
  - „Sag ihr bitte nichts.“ – „Ich werde schweigen.“
  - „Sag ihr das.“

### "„" + "Sei" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sei bitte pünktlich!“ — „Das bin ich doch immer!“
  - „Sei in Zukunft vorsichtiger!“ – „Jawohl.“ – „Nun gut, du kannst gehen.“
  - „Sei still“, flüsterte er.

### "„" + "Um" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hast du Tom mein Geheimnis verraten?“ – „Um Gottes willen! Natürlich nicht!“
  - „Um wieviel Uhr verließ Mr. Drebber Ihr Haus, um zum Zug zu gehen?“ fragte ich.
  - „Wozu brauchst du das Auto?“ – „Um in die Stadt zu fahren.“

### "„" + "Verstehe" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verstehe“, antwortete er.
  - „Verstehe“, brachte ich hervor.
  - „Verstehe“, sagte ich.

### "„" + "Vielen" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vielen Dank für die Auskunft.“ – „Keine Ursache!“
  - „Vielen Dank! Du hast mir sehr geholfen!“ — „Immer wieder gern!“
  - „Vielen Dank.“ Piotr verbeugte sich und trat rückwärts zur Tür hinaus.

### "„" + "Welcher" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Welcher Tag ist heute?“ – „Mittwoch.“
  - „Welcher Wochentag ist heute?“ – „Mittwoch.“
  - „Welcher klassische Komponist gefällt dir am besten?“ – „Debussy.“

### "„" + "Wieviel" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wieviel schulde ich dir?“ – „Fünf Taler.“
  - „Wieviel verdienst du hier?“ – „Das tut nichts zur Sache.“
  - „Wieviel würde das kosten?“ – „Viertausend Dinar.“

### "„" + "Wirklich" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ist Tom hier?“ – „Nein, er ist gerade gegangen.“ – „Wirklich?“
  - „Wirklich?“ „Ja, wirklich.“
  - „Wirklich?“, fragte ich.

### "„" + "Wollen" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe Hunger.“ – „Ich auch.“ – „Wollen wir was essen?“ – „Ja.“
  - „Wollen wir bis morgen warten?“ – „Gut, aber nur bis mittags.“
  - „Wollen wir einen trinken?“ – „Ja, trinken wir einen!“

### "„" + "Worüber" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Denke bitte gut darüber nach!“ — „Worüber denn?“
  - „Ich habe gerade mit ihm gesprochen.“ – „Worüber denn?“
  - „Worüber habt ihr beiden euch unterhalten?“ – „Über nichts weiter.“

### "„" + "Wovon" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich muss dich davon abhalten.“ – „Wovon?“
  - „Wann hast du davon erfahren?“ – „Wovon erfahren?“
  - „Wovon redest du?“ – „Du weißt ganz genau, wovon ich rede.“

### "„" + "aber" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Beginne einen Satz nicht mit „aber“!
  - Das Wort „aber“ ist viel wichtiger als das Wort „und“.
  - „Das habe ich gemacht“, beschwerte sie sich, „aber da war kein Wasser drin!“

### "„" + "gut" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Da kam eine Bauersfrau die Straße herab und rief „gut Mus feil! gut Mus feil!“
  - Er hat den Kurs mit einem „gut“ bestanden.
  - Ich erklärte, was „gut“ bedeutete.

### "„" + "in" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Captain“, rief Jennifer, „in den Booten sitzen ausschließlich Männer.
  - Die andern kamen gelaufen und riefen „in meinem hat auch jemand gelegen.“
  - Es ist unter den jungen Leuten jetzt „in“, gebrauchte Kleidung zu tragen.

### "„" + "m" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Aus irgendeinem Grund habe ich „m“ gelesen und nicht „n“.
  - Dann tippte ich auf das „m“.
  - Es erklang ein kaum hörbares Klicken, und hinter dem „a“ erschien ein „m“.

### """ + "Apache" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da steht ja "wahrer" Apache.
  - Da steht ja gar nicht "echter" Apache.

### """ + "Gerechtigkeit" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht ist es nicht "die" Gerechtigkeit, aber es ist "eine" Gerechtigkeit.
  - Vielleicht ist es nicht "die" Gerechtigkeit, aber es ist "eine" Gerechtigkeit.

### """ + "Qureshi" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Hitler" Qureshi.
  - Doch aus Angst vor "Hitler" Qureshi hat er ihn nie abgeschickt.

### """ + "Wohnung" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Diese" Wohnung ist die Wohnung, in der ihr groß geworden seid.
  - "Diese" Wohnung liegt am Strand von Boa Viagem.

### """ + "auch" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich wusste nicht, dass das "Gebrüll der Löwin" auch noch verstärkt werden kann!
  - Wenn wir "Lazlo" auch außerhalb seiner Räume aufklären können.

### """ + "aufladen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du musst deine "Oyster card" aufladen.
  - Sie müssen Ihre "Oyster card" aufladen.

### """ + "auseinanderzuhalten" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist mitunter schwierig "richtig" und "falsch" auseinanderzuhalten.
  - Ich denke, es ist wichtig, "Meinung" und "Tatsache" auseinanderzuhalten.

### """ + "das" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht ist "Bewusstsein" das bessere Wort.
  - Wenn du mit "goldenen Filmzeiten" das Fernsehen vor acht Jahren meinst, dann ja.

### """ + "dazu" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Okay, ich schreibe einfach mal "Sport" dazu.
  - Schreib noch "Jude" dazu.

### """ + "ertönt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Lied "Canto general" ertönt.
  - Das spanische Lied "canto general" ertönt.

### """ + "fragt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ritter von Nazareth?" fragt die Lehrerin.
  - "Und wo sind die Hemden?" fragt sie.

### """ + "gegen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du verwendest "Bonettis Verteidigung" gegen mich, huh?
  - Jetzt beginnt gleich die Partie "White Bears" gegen "Kenzan Fighters".

### """ + "geht" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das "A" geht nicht.
  - Ohne "Kanone" geht's nicht!

### """ + "gehört" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du schon mal von der Idee der "acht Millionen Götter" gehört?
  - Ich habe "Spider, Spider" gehört.

### """ + "gemeint" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das hast du mit "Hühnchen-Machen" gemeint?
  - Und wer ist da mit "Idiot" gemeint, Sie oder ich?

### """ + "gerufen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als ich dich "Marnie" gerufen habe, bist du erschrocken.
  - Er hatte einfach "Alles gut" gerufen, und wir konnten ruhig schlafen.

### """ + "geschrieben" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er wusste nicht, dass Shakespeare "Romeo und Julia" geschrieben hat.
  - Wo hast du "Love Thing" geschrieben?

### """ + "gibt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Krass, Alter" gibt es nicht mehr.
  - "Krass, Alter" gibt es nicht mehr?

### """ + "heraus" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Herkules O'Brien fordert die "Kranken Ostasiens" heraus.
  - Ich bringe den "Waterford Sentinel" heraus.

### """ + "kam" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Doch dieses "Morgen" kam für die Prinzessin und Sutemaru niemals.
  - Kannst du nachsehen, ob was von "Pizza Generation" kam?

### """ + "klingt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Alles über Eva" klingt komisch.
  - "Menschen mit besonderen Fähigkeiten" klingt besser als "behinderte Menschen".

### """ + "machen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Genau, du musst ein "Atonement" machen.
  - Sie müssen "Schützengräben" machen.

### """ + "meine" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit "heim" meine ich ein Bundesgefängnis.
  - Mit "wir" meine ich nicht dich.

### """ + "nach" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schau mal bei "McDonald's" nach.
  - Schlagen wir unter "weiß" nach.

### """ + "reden" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wer hier von "ergeben" reden sollte, dem geht es wie diesem Tisch!
  - Wenn du wieder auf die Beine kommst, können wir mit "Enchante" reden.

### """ + "spielen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will aber trotzdem "Ich sehe was, was du nicht siehst" spielen.
  - Sie wollte "Der Blick auf Qin Huai" spielen, und deshalb musste sie sterben.

### """ + "steckt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die "Akademiker" steckt im Eis fest.
  - In "Sehschlange" steckt das Wort sehen.

### """ + "vor" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Wir" vor "Mir"!
  - Er stellte mich als "John" vor, aber ich heiße "Jack".

### """ + "waren" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Dinosaurier in dem Film "Jurassic Park" waren lebensecht.
  - Wahrscheinlich ist sie angepisst, weil wir nicht für "die Veränderer" waren.

### """ + "was" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bitte" was?
  - "Wie lange" was?

### """ + "werden" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Filme wie "Meine Ehefrau Pamela" werden nicht mehr gemacht.
  - In den "Gelben Seiten" werden Sie keinen Exorzisten finden.

### """ + "würde" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Herr Hitler" würde mir genügen.
  - "Schneesturm" würde besser passen.

### """ + "zum" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Sugar Babies" zum Beispiel.
  - Nimm "Bingo Martin" zum Beispiel.

### "-" + "er" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieser Tisch ist gut bis auf einen Aspekt - er passt nicht in mein Zimmer.
  - Er ist-- er ist einer von denen Sir.

### "-" + "es" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Folgende anzuhören ist kaum erträglich - es zu sagen ist noch schwerer.
  - Sie wissen, - es geht zu Ende, seit die USA in den Krieg gezogen sind.

### "-" + "in" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du wärst - in Schwierigkeiten ohne mich.
  - Ist es Herrn Yis Idee, - in das Viertel der Japaner zu fahren?

### "-" + "nicht" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich kann darüber jetzt - nicht reden.
  - Nichts hält ewig in dieser verhexten Welt - nicht mal unsere Sorgen.

### "-" + "sind" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Menschen, mit denen ich verkehre, - sind wichtige Leute der Gesellschaft.
  - Staatsstreiche, Mafia, Korruption - sind das nicht Ihre Kunden?

### "-" + "sondern" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Krieg bestimmt nicht, wer Recht hat - sondern nur, wer am Leben bleibt.
  - Wenn ich du wäre, würde ich mir das nicht zwei- sondern dreimal überlegen.

### "-" + "war" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die letzte Medizin - war schnell verkauft.
  - Dieser Stoff - war Ihrer Frau zu auffällig.

### "-" + "zu" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tsao gehört also nicht - zu den Vertrauten.
  - Zu eindeutigen Lösungen für eindeutige Problemstellungen - zu kommen.

### "." + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Also wirklich", grinste Dima. "Das soll wohl ein Witz sein!"
  - "Keine Sorge", versuchte Dima die Frau zu beruhigen. "Es ist nicht meines."

### "Aber" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber … wir sind noch am Leben.
  - »Aber … was ist das?«

### "Abgemacht" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Abgemacht“, sagte er.
  - „Abgemacht“, sagte ich.

### "Also" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also "meine Tochter".
  - Also "nein".

### "Anteil" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einen Anteil“, sagte Johanna.
  - „Ich verkaufe dir gern einen Anteil“, sagte Christopher.

### "Arschloch" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ein Fremder schreibt nicht „Arschloch“.
  - Er schreibt „Arschloch“.

### "Außerdem" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Außerdem“, fügte ich hinzu, „habe ich etwas Geld zur Verfügung.
  - Außerdem“, sagte ich, „wurden sie verfolgt.

### "Baum" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Onkel Teo ist auf dem Baum !
  - Onkel Teo ist auf einem Baum !

### "Begriff" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich werde nicht den Begriff "Intelligenz" benutzen.
  - Und wie ist meine eindeutige Haltung zum Begriff "Schicksal"?

### "Bei" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei "Location" gibst du die URL ein.
  - Bei "Unknown Pleasures" war alles perfekt.

### "Beispiel" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tatoeba“ bedeutet auf Japanisch „zum Beispiel“.
  - „Tatoeba“ bedeutet „zum Beispiel“ auf Japanisch.

### "Beziehung" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das meinte ich mit „Beziehung“.
  - Was meinst du mit „Beziehung“?

### "Binde" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Deine Krawatte oder die Anarchisten-Binde ?
  - Was für eine Binde ?

### "Braut" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle hinter die Braut !
  - Eine feuchte Braut ist eine glückliche Braut !

### "Brücke" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Brücke“ ist übertrieben.
  - „Unter der Brücke“, antwortete sie.

### "Buch" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe das Buch "Die Göttliche Komödie" gelesen.
  - Liest du gerade das Buch "Der Graf von Monte Christo"?

### "Bücher" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Vater hatte solche Bücher“, sagte Kat.
  - „Neue Bücher“?

### "Captain" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Captain“, rief Jennifer, „in den Booten sitzen ausschließlich Männer.
  - „Captain“, sagte Saunders, „seht euch die Tischlampe genau an.“

### "Danke" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kennst du das Wort „Danke“?
  - „Danke“, sagte ich.

### "Dann" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann "Hals- und Beinbruch".
  - Dann "Wo wohnst du?"

### "Das" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das … wenn du schlafen gehst, begib dich in ein Klassenzimmer.
  - Das … würde ich mir gern für mein Therapietier aufheben.

### "Dieses" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieses "Sie" lässt mich im fünften Kreis der Hölle versinken.
  - Dieses "Wunder" war nötig, um mir wieder Mut zu machen.

### "Eichhörnchen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie gründeten eine Musikgruppe mit dem Namen „Die Eichhörnchen“.
  - Toms Lieblingslied ist „Durchgedreht“ von der Musikgruppe „Die Eichhörnchen“.

### "Ein" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein … ein unmöglicher Preis.
  - „Ein …“ Ich biss mir auf die Zunge und dachte sehr, sehr schnell nach.

### "Ende" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist das Ende“, murmelte mein Vater, bevor er bewusstlos wurde.
  - „Das ist das Ende“, murmelte mein Vater, bevor er das Bewusstsein verlor.

### "Erdschiff" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom und Maria haben ein „Erdschiff“ gebaut.
  - Tom und Maria wohnen in einem „Erdschiff“.

### "Familie" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In der Familie "Elektriker" der Großvater.
  - In der Familie "Kameramann" die Mutter.

### "Film" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Dinosaurier in dem Film "Jurassic Park" waren lebensecht.
  - Ein Freundschaftsfoto von uns, inspiriert vom Film "Human Centipede".

### "Flugobjekt" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - UFO bedeutet „unidentifiziertes Flugobjekt“.
  - UFO steht für „unidentifiziertes Flugobjekt“.

### "Freude" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Beethoovens „Ode an die Freude“ ist die offizielle Hymne der Europäischen Union.
  - Das war's dann mit „Freude, Freude, Freude“.

### "Freunde" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Diese „Freunde“ üben einen schlechten Einfluss auf dich aus.
  - „Freunde“, am Arsch.

### "Geld" + "«" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Das ist Geld«, sagte ich.
  - »Mit Geld«, antwortete ich.

### "Geschichte" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eigentlich bin ich Experte für ihre Geschichte“, sagte Nicholas.
  - „Eine lange Geschichte“, sagte Plaitonos.

### "Geschmacks-" + "und" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle vier verloren ihren Geschmacks- und Geruchssinn.
  - Geschmacks- und Geruchsirritation, verzögerte Wahrnehmung.

### "Geschäft" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gefahr gehört zu meinem Geschäft“, bemerkte ich.
  - „Klingt für mich nach einem guten Geschäft“, sagte ich.

### "Glück" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Können Sie „Zum Geburtstag viel Glück“ auf französisch singen?
  - Tom sang für Maria „Zum Geburtstag viel Glück“.

### "Golem" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „Golem“ kommt aus dem Hebräischen.
  - Das Wort „Golem“ stammt aus dem Hebräischen.

### "Granny" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Granny“, rief ich.
  - „Guten Abend, Granny“, sagte Kat höflich.

### "Haupt" + "grund" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das ist der Hauptgrund, weswegen ich mir kein neuen Auto zulegen will.
  - Das ist nicht der Hauptgrund, warum Tom das lassen sollte.

### "Haus" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie schreibt sich „Haus“ auf Arabisch?
  - „Wie bei einem Haus“, sagte ich.

### "Hin-" + "und" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hin- und Rückfahrt?
  - Ihm wurde ein Hin- und Rückflug nach Genf auf das Konto des Papstes ausgestellt.

### "Hoch-" + "und" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er ist weitergeleitet worden an die Hoch- und Tiefbauabteilung.
  - Hoch- und Tiefbauabteilung.

### "Ich" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich“, sagt der Stier, weil ich schießen kann.
  - „Ich“, sagte die Drossel, die sich auf einen Busch setzte.

### "In" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In "Bounty Law".
  - In "Sehschlange" steckt das Wort sehen.

### "Ins" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ins "Sevilla".
  - Ins "Shu Yu".

### "Jahres" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieser Wagen wurde zum „Auto des Jahres“ gewählt.
  - Rosch Haschana bedeutet wörtlich „Kopf des Jahres“.

### "Jeff" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jeff "Babey".
  - Stimm-Styling von Jeff "Bebay".

### "Kakteen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Kaktus“ lautet „Kakteen“.
  - Die Mehrzahl von „Kaktus“ lautet „Kakteen“.

### "Kaktus" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Kaktus“ lautet „Kakteen“.
  - Die Mehrzahl von „Kaktus“ lautet „Kakteen“.

### "Karin" + "as" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Karinas Party hat dir gut getan, was?
  - Karinas Party war Wahnsinn, oder?

### "Katze" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe die Katze "Tama" genannt.
  - Ich nannte die Katze "Tama".

### "Kein" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kein "Mist", kein "Scheiße" all die Jahre.
  - Kein "uchebe"?

### "Kind" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lasst uns „Vater, Mutter, Kind“ spielen!
  - Wir spielten oft „Vater, Mutter, Kind“ im Park.

### "Liebe" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was heißt „Liebe“ auf Französisch?
  - „Liebe“ und „Hass“ sind Gegensätze.

### "Mann" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Er ist ein guter Mann“, sagte Agatha.
  - „Ich habe einen ganz reizenden Mann“, sagte Maria.

### "Marsfahrzeug" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das „Neugier-Marsfahrzeug“ ist auf dem Mars gelandet.
  - Das „Neugier-Marsfahrzeug“ ist im Gale-Krater gelandet.

### "Ma’am" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist viel mehr als nötig, Ma’am“, antwortete er.
  - „Ja, Ma’am“, antwortete Henrison.

### "Meer" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieses Wort bedeutet „Meer“.
  - „Wie die Fische im Meer“, erwiderte Vincent.

### "Minecraft" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hör auf, „Minecraft“ zu spielen!
  - Höre auf, „Minecraft“ zu spielen!

### "Moby Dick" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie schon einmal „Moby Dick“ gelesen?
  - Habt ihr schon einmal „Moby Dick“ gelesen?

### "Morgen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie grüßte mich mit einem „Guten Morgen“.
  - „Guten Morgen“, sagte Tom lächelnd.

### "Nachrichten" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Nachrichten“, sagte ich.
  - „Nachrichten“, antwortete ich.

### "Name" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was bedeutet der Name "Gal"?
  - Was ist mit dem Name "Jesus"?

### "Natürlich" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Natürlich“, sagte Gunther.
  - „Natürlich“, sagte ich.

### "Nächte" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Freund hat mich als „Meister der durchgemachten Nächte“ bezeichnet.
  - Mein Freundin hat mich als „Meisterin der durchgemachten Nächte“ bezeichnet.

### "OV" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der letzte Tagesbericht des OV "Lazlo".
  - Noch läuft ja der OV "Lazlo".

### "Oder" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Oder "die Feenwesen".
  - Oder "zahnstocherndes Hähnchen"?

### "Oh" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Oh … Das Schloss, ja.
  - Oh … Ich spür wohl Ärger im Paradies.

### "Papa" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mach das Kreuz, Papa !
  - Papa !

### "Preis" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Preis“, sagte sie.
  - „Zum selben Preis“, antwortete ich.

### "Protokoll" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Akustisches Protokoll "Ruhe" befolgen.
  - Protokoll "Festung".

### "Sagt" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sagt "Freundschaftsfüßler."
  - Sagt "Mann und Frau"!

### "Schiff" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist das Schiff "La Cenepa".
  - Sie nannten das Schiff "Mayflower".

### "Schiff" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schiff“, verbesserte Sparhawk sie.
  - „Schönes Schiff“, sagte er.

### "Schiffsherrin" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einen Augenblick, Schiffsherrin“, sagte Chiang.
  - „Schiffsherrin“, brachte er hervor.

### "Schreib" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Schreib's auf meinen Deckel, bis ich wiederkomme.
  - Schreib's auf.

### "Sie" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ach … Sie … Sie hat's nicht überstanden.
  - Sie … muss Nick finden.

### "Slovensko" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Slowakei heißt auf Slowakisch „Slovensko“.
  - Die Slowakei wird auf Slowakisch „Slovensko“ genannt.

### "Speisewagen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kann man in „Kalles Speisewagen“ einen Tisch reservieren?
  - „Kalles Speisewagen“ hat dichtgemacht.

### "Sport" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich lese alle Rubriken außer die Rubrik „Sport“.
  - „Ernsthafter Sport“, schrieb George Orwell, „ist Krieg ohne das Schießen.“

### "United" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „City“ hat gegen „United“ verloren.
  - „City“ ist gegen „United“ früh in Führung gegangen.

### "Vater" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wirfst Steine gegen deinen alten Vater ?
  - Ärgerst du immer noch deinen Vater ?

### "Versprich" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Versprich's mir.
  - Versprich's.

### "Vogel" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „Vogel“ auf Tibetisch?
  - Wie schreibt man auf tibetisch „Vogel“?

### "Was" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was'n los?
  - Was'n?

### "Was" + "'n" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was'n Outfit, Junge!
  - Was'n schräger Vogel!

### "Wegen" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wegen "des Lehrplans"?
  - Wegen "des Rankings".

### "Welt" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist Ihre Welt“, erklärte Schiffskapitän Gunther.
  - „Willkommen im Gasthaus am Ende der Welt“, sagte Chiang feierlich.

### "Wiedersehen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Geh ihm „Auf Wiedersehen“ sagen, bevor er weg ist.
  - Geh und sag ihm „Auf Wiedersehen“, bevor er wegfährt.

### "Woche" + "»" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie in der «Woche» gelesen?
  - In der «Woche».

### "Wortes" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da kommt eine Definition des Wortes "Vollpfosten".
  - Ich kenne die Bedeutung des englischen Wortes "gerund" nicht.

### "Zweimaster" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du brauchst noch einen Zweimaster“, sagte sie.
  - „Wir brauchen noch ein paar Zweimaster“, sagte Rudolph.

### "a" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es erklang ein kaum hörbares Klicken, und hinter dem „a“ erschien ein „m“.
  - Vorsichtig tippte ich auf das „a“.

### "aber" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Beginne einen Satz nicht mit „aber“!
  - Das Wort „aber“ ist viel wichtiger als das Wort „und“.

### "abgeschlossen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Kurzwellenscan des Gebäudes ist abgeschlossen“, antwortete er.
  - „Die Notreparaturen in Spencerport sind abgeschlossen“, sagte Jill.

### "alles" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum passiert mir das alles ?
  - Was fragst du da alles ?

### "an" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schaut euch die neuen Töchter an !
  - Zündet eine Kerze an !

### "aus-" + "und" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du solltest dir die nassen Sachen aus- und etwas Trockenes anziehen.
  - Sie sollten sich die nassen Sachen aus- und etwas Trockenes anziehen.

### "befriedigend" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe in Englisch ein „befriedigend“ bekommen.
  - Warum habe ich ein „befriedigend“ bekommen?

### "bisschen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ein bisschen“ und „ein kleines bisschen“ – das ist fast das gleiche.
  - „Ein bisschen“ und „ein kleines bisschen“ – das ist fast das gleiche.

### "bitten" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ab einem bestimmten Alter will niemand seine Eltern um Hilfe bitten“, sagt Li.
  - „Wir müssen dich um einen Gefallen bitten“, sagte Clorinda schließlich.

### "bleiben" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe für „bleiben“ gestimmt.
  - „Alle Kanadier sollten, soweit es geht, zu Hause bleiben“, sagte Trudeau.

### "can" + "not" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - As two neighbors cannot come.
  - Without glasses I cannot read the digits.

### "danke" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du könntest wenigstens „danke“ sagen.
  - Wie sagt man „danke“ auf Japanisch?

### "das" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was ist das ?
  - Wer ist das ?

### "das" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Natürlich hast du das“, sagte Sparhawk.
  - „Natürlich kannst du das“, antwortete er.

### "denn" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum weinst du denn ?
  - Was begehrst du denn ?

### "dich" + "-" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe dich - die ganze Zeit gesucht.
  - Ich habe nur Augen für dich - ehrlich.

### "diese" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Manchmal sehe ich sowas vor mir, wie diese "ewige Wiederkehr".
  - War diese "Entscheidung" einvernehmelich?

### "diesen" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Reden wir über diesen "Unfall".
  - Schaffen Sie diesen "Itzick" hier raus.

### "dieser" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber ich denke, dieser "Thibault" hebt "Capo Ferro" auf, du nicht?
  - Und wir stehen da mit dieser "Wahrheit".

### "du" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich heiße nicht „du“, sondern Ricardo.
  - „Genauso viel wie du“, erwiderte ich.

### "eben" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du eben "Unfug" gesagt?
  - Ich vertausche eben "er" und "sie".

### "eigentlich" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wer ist eigentlich "Dada"?
  - Wieso eigentlich "Phil"?

### "ein-" + "oder" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ja, sie hatten mich ein- oder zweimal geweckt.
  - Tom kommt ein- oder zweimal im Jahr nach Boston.

### "er" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was schreit er ?
  - Wie ist er ?

### "erfinden" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Verwende nicht „entdecken“, wenn du eigentlich „erfinden“ meinst.
  - Was sollte es heißen, die Null zu „erfinden“?

### "es" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich weiß es“, sagte Granny.
  - „So viele Fundamente gibt es“, antwortete ich.

### "fangen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bald werde ich erheblich mehr Fisch fangen“, sagte ich.
  - „So viel kann ich vormittags nicht fangen“, antwortete ich.

### "fest" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber dann steckt dein Geld in Rubricum fest“, erwiderte er.
  - „Bis dahin sitzen wir hier fest“, sagte Vincent.

### "gegessen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du hast oft genug im Pavilion At The Shore gegessen“, sagte ich.
  - „Ich habe den Keks nicht gegessen“, entgegnete Tom mit Krümeln am Mund.

### "gehen" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wohin alle diese Leute gehen ?
  - Wohin sie mit Herzklopfen gehen ?

### "gehört" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Davon habe ich Gerüchte gehört“, sagte Piotr.
  - „Von einer Papierprüfung habe ich noch nie gehört“, sagte ich.

### "gemacht" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was habe ich gemacht ?
  - Wie hat er das gemacht ?

### "gemacht" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das habe ich gemacht“, beschwerte sie sich, „aber da war kein Wasser drin!“
  - „Das hat mich wahnsinnig gemacht“, antwortete sie.

### "genannt" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Quell aller Phänomene, auch genannt "die Akasha-Chronik".
  - Maria Aparecida dos Santos, genannt "Índia".

### "gerade" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich dachte gerade“, sagte ich.
  - „Ich erzähle es Brunel gerade“, sagte Sparhawk.

### "gerettet" + "«" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Du hast Jim gerettet«, sagte sie.
  - »Ich habe einen von ihnen vor dem Ertrinken gerettet«, sagte ich.

### "gewesen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nun bist du die schönste gewesen“ sprach sie, und eilte hinaus.
  - „So krank bin ich noch nie gewesen“, sagte Tom.

### "hier" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mit „hier“ meinte ich Australien.
  - „Ich bin hier“, sagte ich leise.

### "ich" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das sehe ich … etwas anders.
  - „Wie kann ich …“, begann ich.

### "immer" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mache ich immer“, antwortete ich.
  - „War er schon immer“, antwortete der Fischer.

### "ironisch" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wissen Sie überhaupt, was „ironisch“ bedeutet?
  - Wisst ihr überhaupt, was „ironisch“ bedeutet?

### "is" + "'n" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja, gut, aber das is'n Ferrari.
  - Was is'n jetzt?

### "ist" + "'n" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er ist'n Nazi!
  - Ich find, er ist'n bisschen jung.

### "ist" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe gehört, er hat bezahlt, bevor er abgereist ist“, sagte ich.
  - „Vicki, bring uns flussaufwärts, sofern es nicht zu flach ist“, antwortete ich.

### "ist" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist … Ich versteh das nicht.
  - Was wir suchen, ist … älter.

### "ja" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „ja“ kommt ihm nie über die Lippen.
  - Die Antwort war „ja“.

### "jemand" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hat jemand "Harry" gesagt?
  - Im Übrigen hat jemand "Kopf abnehmen" neben mein Mikrofon geschrieben.

### "jetzt" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lass mich alleine, geh jetzt !
  - So, genug jetzt !

### "jetzt" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In seinen Augen kann man schon jetzt "Amerika" lesen.
  - Okay, und jetzt "Ich sehe dich."

### "kann" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er kann "Ranger" nicht mal buchstabieren!
  - Sie kann "Papa" sagen!

### "kannst" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du kannst "Die Abhängigkeit" aufführen.
  - Du kannst "Whiplash" auswendig?

### "kennenzulernen" + "«" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich freue mich auch, dich kennenzulernen«, erwiderte sie.
  - »Ich freue mich sehr, dich endlich kennenzulernen«, sagte ich.

### "kennt" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jeder hier kennt "Das Licht der Wüste".
  - Niemand kennt "die Richtige", nicht mal ich.

### "könnte" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe nichts mehr, das ich geben könnte“ antwortete das Mädchen.
  - „Mir wurde gesagt, dass eine Nacht genügen könnte“, antwortete Saunders.

### "lautet" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Befehl den die Soldaten erhalten haben lautet "kämpft um zu töten".
  - Die Antwort lautet "ja".

### "lebenslänglich" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Mann bekam „lebenslänglich“.
  - Tom steht „lebenslänglich“ bevor.

### "leid" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das tut mir leid“, sagte ich.
  - „Tut mir leid“, seufzte der Junge.

### "mal" + "»" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - «Verdammt noch mal»!
  - «Verdammt noch mal».

### "muss" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er muss "Chicken On A Stick" heißen.
  - Ich muss "Räuber" fertig lesen.

### "nicht" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist ein hübsches Ei, nicht ?
  - Er ist gewachsen, nicht ?

### "oft" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine total geordnete Menge wird oft "Kette" genannt.
  - Warum schreibst du so oft "Frieden"?

### "ohne" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ah, ja, ohne "wertschätzen".
  - Sie ging weg, ohne "Auf Wiedersehen" zu sagen.

### "rauf" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich geh rauf !
  - Schicke mich rauf !

### "räum" + "en" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Brücke räumen.
  - Hank, kannst du das Atelier räumen?

### "schon" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist schon "speziell".
  - Hast du schon "Four Swords" gespielt?

### "sehen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Amanda, ich freue mich auch, dich zu sehen“, erwiderte er.
  - „Das würde ich gern sehen“, sagte ich.

### "seine" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Man nannte sie seine "Diamantenhunde".
  - Scheiß auf Herzog und seine "Motivation"!

### "seiner" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hast die Meisterschaft mit seiner "schlechten" Technik gewonnen.
  - Ich hab ihn so satt, diesen Stepanytsch mit seiner "Niwa".

### "sie" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie ist sie ?
  - Wo ist sie ?

### "so" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nick ist ja nicht so … redselig.
  - Und so … und so noch.

### "sogar" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da hassen die sogar "Forrest Gump", den besten Film der Welt.
  - Eine sagte sogar "sexy".

### "sogenannte" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist die sogenannte "List der Schönheit"!
  - Der sogenannte "Kunsthändler"!

### "soll" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also, Charlie, was soll "rot werden" sein?
  - Gesungen werden soll "Non nobis" und "Te Deum".

### "trinken" + "?" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Möchtest du auf den Sieg des Faschismus trinken ?
  - Warum sollte ich das trinken ?

### "um" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Zeit ist gekommen, um "Leb wohl!" zu sagen.
  - Wenn, dann um "ja" zu sagen.

### "um" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich seh mich um“, sagte sie.
  - „So geht man auf Bringobble mit Geld um“, antwortete er.

### "unbedingt" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Lüge nicht" bedeutet nicht unbedingt "sage die Wahrheit".
  - Als ich klein war, wollte ich unbedingt "Der Weiße Hai" sehen.

### "und" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „aber“ ist viel wichtiger als das Wort „und“.
  - Tom beginnt jeden Satz mit „und“.

### "uns" + "!" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fall nicht über uns !
  - Sie verlässt uns !

### "versuchen" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich kann es versuchen“, sagte ich.
  - „Ich werde es versuchen“, sagte ich.

### "viel" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Hälfte ist ganz schön viel“, sagte ich.
  - „Du redest zu viel“, sagte er.

### "vor" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Wir" vor "Mir"!
  - Doch aus Angst vor "Hitler" Qureshi hat er ihn nie abgeschickt.

### "wegen" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nur wegen "Vorfahrt gewähren".
  - Von wegen "Missverständnis"!

### "weiter" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Preise für die Schuldscheine sinken immer weiter“, sagte sie.
  - „Sie sinken immer weiter“, sagte sie.

### "wieder" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sag nicht wieder "Mookie arbeitet nicht".
  - Schon wieder "glauben", na ja.

### "wieder" + "finden" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du wirst mir sagen, was du weißt, ansonsten wirst du dich im Knast wiederfinden.
  - Ich habe mein Mobiltelefon verloren und kann es nicht wiederfinden.

### "wir" + "“" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.
  - „Hauptsächlich wir“, sagte Missy Marky.

### "wo" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da, wo "Unterschrift" steht.
  - Ich weiß nicht mal, wo "hier" ist.

### "wäre" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn es ein Verb namens "nachen" gäbe, wäre "Nacht!" der Imperativ davon.
  - Wie wäre "Goldlöckchen und die drei Bären" aus der Sicht der Bären?

### "zur" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Guten Abend, meine Damen und Herren, zur "Aktuellen Kamera".
  - Nein, ich wurde zur "Auffälligsten" gewählt.

### "zwei-" + "oder" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich kaue nicht öfter als zwei- oder dreimal.
  - Tom lässt sich gewöhnlich nur zwei- oder dreimal im Jahr die Haare schneiden.

### "»" + "Also" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Also bin ich unserem trockenen Flusslauf stromaufwärts gefolgt.« Keith kam mit.
  - »Also, was ist passiert?«

### "»" + "Amanda" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Amanda, kannst du mir bitte sagen, was du zu Markofferson gesagt hast?«
  - »Amanda«, begrüßte mich Antonio.

### "»" + "Auf" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Auf Nova Capricornis vier«, verkündete er, »bin ich beinahe ertrunken.«
  - »Auf dein Wort als Schiffsmeisterin?«

### "»" + "Da" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Da ist noch ein Zimmer«, sagte der Kerl.
  - »Da muss es noch einen Haken geben«, erwiderte Khan.

### "»" + "Dann" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Dann verlieren wir die Liegegebühren.«
  - »Dann versuchen Sie mal, jemanden zu finden, der Weißfische fängt«, sagte sie.

### "»" + "Eine" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Eine Maschine, die denkt.«
  - »Eine lange Geschichte, selbst für eine lange Fahrt.«

### "»" + "Er" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Er führt den Laden.«
  - »Er ist nicht verletzt, aber sein Büro ist jetzt in seinem Haus.«

### "»" + "Es" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Es tut mir leid, Ihnen Ihre Zeit stehlen zu müssen«, sagte er.
  - »Es waren sogar mehr als acht«, sagte ich.

### "»" + "Für" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Für Freundinnen«, sagte ich.
  - »Für mich?«, fragte er.

### "»" + "Ganz" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ganz genau«, donnerte Chris.
  - »Ganz unter uns?«, fragte er.

### "»" + "Hallo" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Hallo!«, rief er.
  - »Hallo«, brachte ich hervor.

### "»" + "Hast" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Hast du gelernt zu träumen?«, fragte sie.
  - »Hast du vor, es zu kaufen?«, fragte er.

### "»" + "Immer" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Immer zusammen!« kam es laut aus dem Raum zurück.
  - »Immer zusammen!«, rief er.

### "»" + "Jetzt" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Jetzt kommt der geheime Teil«, sagte ich.
  - »Jetzt.« Ich öffnete meinen Geldbeutel und begann zu zählen.

### "»" + "Kein" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Kein Gott, kein Land, kein Meister«?
  - »Kein Käufer«, sagte Brunel.

### "»" + "Mutter" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Mutter, Mutter,« sagte Gregor leise und sah zu ihr hinauf.
  - »Mutter?« rief die Schwester von der anderen Seite.

### "»" + "Nummer" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Nummer drei«, sagte ich.
  - »Nummer zwei«, fuhr ich fort.

### "»" + "Oh" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Oh nein.« Ihre Stimme hellte sich auf.
  - »Oh, gut, du bist wach.«

### "»" + "Wir" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Wir tun das für dich, Chris«, hielt ich dagegen.
  - »Wir wissen, dass sie heute ausziehen muss.« Stille.

### "»" + "was" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Herr Samsa,« rief nun der Prokurist mit erhobener Stimme, »was ist denn los?
  - »Gregor, Gregor,« rief er, »was ist denn?«

### "Über" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Über "Sabor".
  - Über "den" Kerl.

### "äh" + "…" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie, äh … haben ihn geschnappt.
  - Äh … äh … was?

### "über" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieses Zeug über "Lebenskräfte stehlen"?
  - Schreibt einen Aufsatz über "Freundschaft".

### "“" + "(" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gérard de Nerval schrieb „Voyage en Orient“ (Reise im Orient).
  - „Clair de lune“ (Mondschein) wurde von Claude Debussy komponiert.

### "“" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Motto für den Antiextremismus-Tag war „bunt statt braun“ .
  - Sagen Sie „bitte“ .

### "“" + "Keith" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Landungstrupp, antreten.“ Keith hatte ausgewählt, wer mit uns kommen sollte.
  - „Was?“ Keith wirkte ein wenig verwirrt.

### "“" + "bekommen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe in Englisch ein „befriedigend“ bekommen.
  - Warum habe ich ein „befriedigend“ bekommen?

### "“" + "erwiderte" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Jawohl!“ erwiderte Maria entschlossen.
  - „Weißt du, wo Tom ist?“ – „Ich bin Tom!“ erwiderte Tom.

### "“" + "flüsterte" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ich liebe dich!“ flüsterte er ihr ins Ohr.
  - „Ich liebe dich!“ flüsterte er.

### "“" + "geschrieben" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer hat „Der Fürst“ geschrieben?
  - Wer hat „Romeo und Julia“ geschrieben?

### "“" + "gesehen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du den Film „Fang mich, wenn du kannst!“ gesehen?
  - Wie oft hast du „Matrix“ gesehen?

### "“" + "gestimmt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für dieses Bild haben viele mit „Gefällt mir!“ gestimmt.
  - Ich habe für „bleiben“ gestimmt.

### "“" + "im" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Gebrauch von „oh“ im Englischen ist ziemlich kompliziert.
  - Wir spielten oft „Vater, Mutter, Kind“ im Park.

### "“" + "nicht" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich kann die Akte „Tom Hansen“ nicht finden.
  - Leider gibt es das Wort „leiderlicherweise“ nicht.

### "“" + "sagten" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Aha!“ sagten jene.
  - „Komm herab zu uns,“ sagten sie, „wir wollen dir nichts zu Leid tun.“

### "“" + "schreibt" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Den Mexikanern gefällt es nicht, wenn man „Méjico“ anstatt „México“ schreibt.
  - Dorgan wusste nicht, wie man „Dackel“ schreibt.

### "“" + "schrie" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Nein!“ schrie Maria, als Tom ihr Glas voll Milch verschüttete.
  - „Wo in aller Welt bist du gewesen?“ schrie Toms Mutter.

### "“" + "sind" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Am „Silizium-Kreisel“ sind zahlreiche junge IT-Unternehmen ansässig.
  - „Liebe“ und „Hass“ sind Gegensätze.

### "“" + "spielen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe Tom mit seinem kleinen Kindchen „Kuckuck!“ spielen sehen.
  - Lasst uns „Vater, Mutter, Kind“ spielen!

### "“" + "sprachen" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ach,“ sprachen sie, „was will der große Kriegsheld hier mitten im Frieden?
  - „Wie bist du in unser Haus gekommen?“ sprachen weiter die Zwerge.

### "“" + "stand" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Obwohl auf dem Schild an der Tür „Betreten verboten!“ stand, ging Tom hinein.
  - „Spencerport“ stand eindeutig nicht auf der Liste.

### "“" + "von" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom hat „Die Kriegskunst“ von Sūnzǐ gelesen.
  - Toms Lieblingslied ist „Durchgedreht“ von der Musikgruppe „Die Eichhörnchen“.

### "“" + "war" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Buch „Englischunterricht im Ausland“ war für mich nicht von Nutzen.
  - Der „Dreifaltigkeitsplatz“ war das ursprüngliche Zentrum von Sankt Petersburg.

### "“" + "waren" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Begriffe „Senat“ und „Zoll“ waren mir beide neu.
  - „Arbeit“ und „Familie“ waren für sie Synonyme.

### "“" + "－" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ich habe aufgehört zu rauchen.“ － „Wann?“ － „Vor zwei Tagen.“
  - „Ich habe aufgehört zu rauchen.“ － „Wann?“ － „Vor zwei Tagen.“

### "„" + "A" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Relation „A liebt B“ ist weder reflexiv noch transitiv noch symmetrisch.
  - „A“ ist der erste Buchstabe des Alphabets.

### "„" + "Abgelehnt" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einspruch!“ – „Abgelehnt!“
  - „Einspruch!“ — „Abgelehnt!“

### "„" + "Abgemacht" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Abgemacht“, sagte er.
  - „Abgemacht“, sagte ich.

### "„" + "Adasdotr" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Adasdotr?“, fragte ich.
  - „Adasdotr“, antwortete er.

### "„" + "Ah" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sag „Ah!“
  - „Ah“ ist eine Interjektion.

### "„" + "Als" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Als er noch lebte, schuldete er Ihnen Geld“, berichtigte Brunel.
  - „Als ich also die Flagge auf deinem Boot sah, bin ich hergekommen.“

### "„" + "Anson" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Anson Barlow, zu Ihren Diensten.“
  - „Anson ist hinten, Amanda“, begrüßte mich jemand freundlich.

### "„" + "Arbeit" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arbeit hilft am besten gegen Vergrämtheit, mein lieber Watson“, sagte er.
  - „Arbeit“ und „Familie“ waren für sie Synonyme.

### "„" + "Arschloch" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ein Fremder schreibt nicht „Arschloch“.
  - Er schreibt „Arschloch“.

### "„" + "Außer" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Außer auf Bringobble.“
  - „Außer, ihr stoßt auf einen Engpass.“

### "„" + "Behalt" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dieses Handtuch gefällt mir wirklich sehr.“ – „Behalt’s!“ – „Oh, danke schön!“
  - „Nimm meine Sachen.“ – „Behalt sie.“ – „Aber ich will, dass du sie bekommst.“

### "„" + "Betreten" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - An der Tür hängt ein Schild mit der Aufschrift „Betreten nur auf eigene Gefahr!“
  - Obwohl auf dem Schild an der Tür „Betreten verboten!“ stand, ging Tom hinein.

### "„" + "Beziehung" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das meinte ich mit „Beziehung“.
  - Was meinst du mit „Beziehung“?

### "„" + "Bleib" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte doch „Bleib zurück!“!
  - „Bleib noch ein bisschen.“ – „Nein, ich muss nach Hause gehen.“

### "„" + "Brunel" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Brunel, Sie dürfen eine Zeugin nicht niederwalzen.“
  - „Brunel, nicht einer von den Einheimischen.“

### "„" + "Bäumchen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bäumchen, rüttel dich und schüttel dich wirf Gold und Silber über mich.“
  - „Bäumchen, rüttel dich und schüttel dich, wirf Gold und Silber über mich.“

### "„" + "Danach" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Danach habe ich gesucht“, erklärte er.
  - „Danach wird es viel einfacher, die anderen zwei Dutzend Sägewerke zu bauen.“

### "„" + "Darüber" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darüber habe ich nachgedacht“, sagte ich.
  - „Darüber muss ich nachdenken“, sagte er.

### "„" + "Dieser" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dieser Mann hat versucht, mich umzubringen.“
  - „Dieser Wagen ist zu teuer.“ – „ Stimmt, er ist nicht billig.“

### "„" + "Diesmal" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diesmal haben wir nur eine Reihe von Einlagenkonten“, sagte Kat.
  - „Diesmal kommst du nicht davon“, verkündete er.

### "„" + "Dir" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dir auch einen guten Abend, Jerry“, sagte ich.
  - „Dir wird von den Massenmedien das Gehirn gewaschen!“ – „Was redest du da?“

### "„" + "Durch" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie hast du unsere Sprache gelernt?“ — „Durch Zuhören!“
  - „Wie ist er hereingekommen?“ „Durch das Fenster.“

### "„" + "Eigentlich" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eigentlich bin ich Experte für ihre Geschichte“, sagte Nicholas.
  - „Eigentlich sie“, korrigierte Keith.

### "„" + "Enter" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dann zähle ich bis drei und drücke auf „Enter”.
  - Ähm, ich muss hauptsächlich nur auf „Enter” drücken.

### "„" + "Erdschiff" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom und Maria haben ein „Erdschiff“ gebaut.
  - Tom und Maria wohnen in einem „Erdschiff“.

### "„" + "Erst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erst das Geschäft, dann das Vergnügen“, erwiderte ich.
  - „Wann ist er gestorben?“ – „Erst vor kurzem.“

### "„" + "Fang" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du den Film „Fang mich, wenn du kannst!“ gesehen?
  - „Fang damit an, alle deine größeren Barsalden zu begleichen“, schlug ich vor.

### "„" + "Frag" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Frag Tom, wo Maria ist!“ – „Glaubst du, der weiß das?“
  - „Wo ist denn Tom?“ – „Frag mich nicht, Kumpel! Ich habe keine Ahnung.“

### "„" + "Fragen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lesen Sie unsere „Fragen und Antworten“.
  - „Ist das wirklich ein natürlicher Ausdruck?“ – „Fragen wir Herrn Google!“

### "„" + "Frau" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Frau Königin, ihr seid die schönste im Land.“
  - „Wer unterrichtet dich in Englisch?“ – „Frau Yamada.“

### "„" + "Freunde" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Diese „Freunde“ üben einen schlechten Einfluss auf dich aus.
  - „Freunde“, am Arsch.

### "„" + "Früher" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Früher mochtest du doch Äpfel.“ – „Früher schon, jetzt aber nicht mehr.“
  - „Früher mochtest du doch Äpfel.“ – „Früher schon, jetzt aber nicht mehr.“

### "„" + "Golem" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „Golem“ kommt aus dem Hebräischen.
  - Das Wort „Golem“ stammt aus dem Hebräischen.

### "„" + "Gute" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er kam und ging, ohne auch nur „Gute Nacht!“ zu sagen.
  - „Gute Nacht.“ „Ich liebe dich auch.“

### "„" + "Haltet" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haltet den Mund, alle!“ sagte er.
  - „Haltet ihr uns für Narren, die so eine absurde Behauptung glauben würden?“

### "„" + "Hatten" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hatten Sie denn irgendwelche Probleme für mich?“
  - „Hatten Sie eine bestimmte Farbe im Sinn?“, fragte sie.

### "„" + "Hauptsächlich" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hauptsächlich geht es mir um diese drei Orte hier.“
  - „Hauptsächlich wir“, sagte Missy Marky.

### "„" + "Heißt" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heißt du etwa Rumpelstilzchen?“
  - „Warum nennt Tom mich Mary?“ – „Heißt du denn nicht Mary?“ – „Nein, Maria.“

### "„" + "Herr" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom wurde von seinen Freunden „Herr Geldsack“ genannt.
  - „Herr Doktor, wie geht es ihm?“ – „Er kommt durch.“

### "„" + "Hol" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom spielt mit seinem Hund „Hol das Stöckchen!“.
  - „Hol erst einmal Luft“, sagte ich.

### "„" + "Höre" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Höre ich mir gern an.“
  - „Höre!“ rief der Geist. „Meine Zeit ist fast um.“

### "„" + "Hörst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hörst du es, wenn wir bei uns den Fernseher laufen haben?“ – „Kein bisschen.“
  - „Hörst du im Kopf Stimmen?“ – „Ja, dämonische.“

### "„" + "Immer" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Möchtest du ein Ringbrötchen?“ – „Immer her damit!“
  - „Vielen Dank! Du hast mir sehr geholfen!“ — „Immer wieder gern!“

### "„" + "Isst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Isst du jeden Tag Foufou?“ – „Ich esse es fast jeden Tag.“
  - „Isst du nichts?“ – „Ich habe keinen Hunger.“

### "„" + "Jetzt" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Jetzt habe ich ein Maschinengewehr.“
  - „Jetzt wird’s spannend!“ sagte er, als ich hereinkam.

### "„" + "Kakteen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Kaktus“ lautet „Kakteen“.
  - Die Mehrzahl von „Kaktus“ lautet „Kakteen“.

### "„" + "Kaktus" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Kaktus“ lautet „Kakteen“.
  - Die Mehrzahl von „Kaktus“ lautet „Kakteen“.

### "„" + "Kalles" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kann man in „Kalles Speisewagen“ einen Tisch reservieren?
  - „Kalles Speisewagen“ hat dichtgemacht.

### "„" + "Kapitän" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kapitän!“
  - „Kapitän?“

### "„" + "Kapitänin" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kapitänin, das Ruder ist schwer“, begann er.
  - „Kapitänin, das sind Leichen“, meldete Larissa.

### "„" + "Keineswegs" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du beliebst zu scherzen.“ – „Keineswegs.“
  - „Kennst du ihn wirklich nicht?“ — „Keineswegs!“

### "„" + "Keith" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Keith hat zwei getötet“, sagte David.
  - „Keith kümmert sich um die Einzelheiten.“ Ich sah ihn vielsagend an.

### "„" + "Kommst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kommst du mich auch besuchen, wenn ich nach Kairo gezogen bin?“ – „Aber hallo!“
  - „Kommst du?“ – „Ne, laß mal! Ich bleibe hier.“ – „Wie du willst.“

### "„" + "Leinen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Leinen einholen!“, rief ich.
  - „Leinen liegen und sind festgesetzt“, meldete er schließlich.

### "„" + "Lena" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lena leitet die Terrasse.“
  - „Lena“, sagte ich, „ich muss mit dir reden.“

### "„" + "Machen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Machen wir damit Gewinn?“, fragte ich.
  - „Willst du schwimmen oder segeln gehen?“ – „Machen wir beides!“

### "„" + "Magst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Magst du mich?“ – „Ja.“
  - „Magst du mit mir spielen?“ — „Nö, mit dir nicht!“

### "„" + "Mal" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mal sehen.“ Er knabberte an einem Scone und nickte zufrieden.
  - „Wann kommst du zurück?“ – „Mal seh’n, wie’s Wetter wird.“

### "„" + "Meinen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Meinen Ring von dem Finger“ antwortete das Mädchen.
  - „Was hast du in den Taschen?“ – „Meinen Haustürschlüssel und einige Münzen.“

### "„" + "Meinst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich mag den Neuen nicht.“ – „Meinst du Tom? Dich mag er, glaube ich.“
  - „Meinst du das ironisch?“ – „Nein, ich meine es ernst.“

### "„" + "Minecraft" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hör auf, „Minecraft“ zu spielen!
  - Höre auf, „Minecraft“ zu spielen!

### "„" + "Mittwoch" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Welcher Tag ist heute?“ – „Mittwoch.“
  - „Welcher Wochentag ist heute?“ – „Mittwoch.“

### "„" + "Moby Dick" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Haben Sie schon einmal „Moby Dick“ gelesen?
  - Habt ihr schon einmal „Moby Dick“ gelesen?

### "„" + "Mr." (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mr. Ackerson!“, brüllte ich dem Zimmermann zu.
  - „Mr. Murchison, holen Sie eine Proviantdose heraus.“

### "„" + "Müssen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Müssen Sie jetzt nicht zurücktreten?“ — „Nein, auf gar keinen Fall.“
  - „Tom macht Frühstück.“ – „Müssen wir die Feuerwehr holen?“

### "„" + "Neugier" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das „Neugier-Marsfahrzeug“ ist auf dem Mars gelandet.
  - Das „Neugier-Marsfahrzeug“ ist im Gale-Krater gelandet.

### "„" + "Nimm" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte „Nimm!“
  - „Nimm meine Sachen.“ – „Behalt sie.“ – „Aber ich will, dass du sie bekommst.“

### "„" + "Nö" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Magst du mit mir spielen?“ — „Nö, mit dir nicht!“
  - „Spricht er hin und wieder von mir?“ – „Nö.“

### "„" + "Och" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Och nö!“ – „Och doch!“
  - „Och nö!“ – „Och doch!“

### "„" + "Oder" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Oder gibt es so etwas nur in Schundromanen?“
  - „Oder weil sie wollen, dass die Universität etwas Bestimmtes erforscht.“

### "„" + "Rapunzel" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rapunzel, Rapunzel, lass dein Haar herunter.“
  - „Rapunzel, Rapunzel, lass mir dein Haar herunter.“

### "„" + "Rotkäppchen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rotkäppchen, das bringt Kuchen und Wein, mach auf.“
  - „Rotkäppchen, wo wohnt deine Großmutter?“

### "„" + "Rudolph" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rudolph, bring uns rein.“
  - „Rudolph, gehen wir.“

### "„" + "Schieß" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dürfen wir dir einige Fragen stellen?“ – „Schieß los!“
  - „Kann ich dich etwas fragen?“ – „Schieß los!“

### "„" + "Schiffsherrin" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schiffsherrin.“
  - „Schiffsherrin“, brachte er hervor.

### "„" + "Schöne" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schöne Kleider“ sagte die eine, „Perlen und Edelsteine“ die zweite.
  - „Schöne runde Zahlen“, erwiderte er.

### "„" + "Selbst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Gefühl von „Selbst“ ist identisch mit dem, was wir Bewusstsein nennen.
  - „Selbst weit draußen in der Außenbucht ist es nicht immer ein halber Zentner.“

### "„" + "Sir" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es wirkt etwas feudalistisch, ihn „Sir“ zu nennen.
  - „Sir!“

### "„" + "Slovensko" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Slowakei heißt auf Slowakisch „Slovensko“.
  - Die Slowakei wird auf Slowakisch „Slovensko“ genannt.

### "„" + "Sollen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sollen wir einen Leibwächter mitnehmen?“, fragte ich.
  - „Sollen wir unser Schiff entsprechend Ihrem Landeraster verlegen?“

### "„" + "Sonst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sonst noch etwas?“
  - „Sonst noch etwas?“ – „Nein, das ist alles.“

### "„" + "Stimmt" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stimmt meine Übersetzung?“ – „Man könnte es nicht besser übersetzen.“
  - „Stimmt mit dieser Lieferung etwas nicht?“

### "„" + "Toms" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Toms Tod war ein Unfall.“ – „Bist du dir sicher?“
  - „Toms Tod war ein Unfall.“ – „Sind Sie sich sicher?“

### "„" + "United" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „City“ hat gegen „United“ verloren.
  - „City“ ist gegen „United“ früh in Führung gegangen.

### "„" + "Verkaufen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verkaufen Sie denn nicht alle zum Buchpreis?“, fragte er.
  - „Verkaufen?“

### "„" + "Verstehst" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verstehst du, was ich meine?“ — „Ja, ich verstehe dich.“
  - „Verstehst du?“ – „Ich verstehe nichts.“

### "„" + "Versuch" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich kann Tom nicht finden.“ – „Versuch mal, ihn anzurufen.“
  - „Versuch, vorsichtiger zu sein!“ – „Gut, ich versuch’s.“

### "„" + "Versuchen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Versuchen wir es drei Tage lang“, sagte ich.
  - „Versuchen wir’s mit einem Stapel Pennys“, sagte ich.

### "„" + "Vogel" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „Vogel“ auf Tibetisch?
  - Wie schreibt man auf tibetisch „Vogel“?

### "„" + "Warte" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Warte mal“, sagte er.
  - „Warte nur, bis meine Freundinnen davon hören“, erwiderte sie.

### "„" + "Welchen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Welchen Apfel hättest du denn gerne?“ – „Ich hätte gerne den da.“
  - „Welchen Namen bevorzugst du?” „Sie sind beide schön.”

### "„" + "Welches" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dieses Lied geht mir nicht mehr aus dem Kopf.“ – „Welches?“
  - „Welches willst du?“ – „Irgendeines. Das ist mir egal.“

### "„" + "Willkommen" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Willkommen auf Goddard“, sagte ich.
  - „Willkommen im Gasthaus am Ende der Welt“, sagte Chiang feierlich.

### "„" + "Wofür" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich sollte Ihnen danken.“ – „Wofür?“
  - „Ich sollte dir danken.“ – „Wofür?“

### "„" + "Wonach" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tom, was machst du denn da?“ – „Wonach sieht es denn aus?“
  - „Was machst du?“ – „Wonach sieht es denn aus?“

### "„" + "Woran" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Woran denken Sie?“ – „An die Mittagspause.“
  - „Woran denkst du?“ – „Ich denke an dich.“

### "„" + "Wozu" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wozu brauchst du das Auto?“ – „Um in die Stadt zu fahren.“
  - „Wozu ist er denn nütze?“ fragte der junge Geselle.

### "„" + "Würde" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Würde bitte jemand den Stadtpolizisten rufen?“, sagte ich.
  - „Würde ich dich denn belügen?“ – „Ja, ich denke schon.“

### "„" + "a" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es erklang ein kaum hörbares Klicken, und hinter dem „a“ erschien ein „m“.
  - Vorsichtig tippte ich auf das „a“.

### "„" + "befriedigend" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe in Englisch ein „befriedigend“ bekommen.
  - Warum habe ich ein „befriedigend“ bekommen?

### "„" + "danke" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du könntest wenigstens „danke“ sagen.
  - Wie sagt man „danke“ auf Japanisch?

### "„" + "erfinden" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Verwende nicht „entdecken“, wenn du eigentlich „erfinden“ meinst.
  - Was sollte es heißen, die Null zu „erfinden“?

### "„" + "guten" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es rief „guten Morgen,“ bekam aber keine Antwort.
  - Keith hatte ich als meinen „guten Freund“ dabei.

### "„" + "ironisch" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wissen Sie überhaupt, was „ironisch“ bedeutet?
  - Wisst ihr überhaupt, was „ironisch“ bedeutet?

### "„" + "ist" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ich tun möchte“, sagte ich, „ist, ein paar neue Dinge hinzuzufügen.
  - „Ernsthafter Sport“, schrieb George Orwell, „ist Krieg ohne das Schießen.“

### "„" + "ja" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „ja“ kommt ihm nie über die Lippen.
  - Die Antwort war „ja“.

### "„" + "lebenslänglich" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Mann bekam „lebenslänglich“.
  - Tom steht „lebenslänglich“ bevor.

### "„" + "sie" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf meine Idee, „sie alle zu durchsuchen“, war er unabhängig gekommen.
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.

### "„" + "unidentifiziertes" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - UFO bedeutet „unidentifiziertes Flugobjekt“.
  - UFO steht für „unidentifiziertes Flugobjekt“.

### "„" + "vielleicht" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das erinnert mich“, sagte ich, „vielleicht sollten wir Tina Tinasdotr befördern.
  - Ich habe „vielleicht“ gesagt.

### "„" + "wenn" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es fehlt das Komma vor „wenn“.
  - „Das mach mir nach,“ sprach der Riese, „wenn du Stärke hast.“

### "„" + "wir" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.
  - „Komm herab zu uns,“ sagten sie, „wir wollen dir nichts zu Leid tun.“

### "„" + "zum" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tatoeba“ bedeutet auf Japanisch „zum Beispiel“.
  - „Tatoeba“ bedeutet „zum Beispiel“ auf Japanisch.

### "„" + "Über" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Liebst du mich?“ – „Über alles!“
  - „Worüber habt ihr beiden euch unterhalten?“ – „Über nichts weiter.“

### "－" + "„" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ich habe aufgehört zu rauchen.“ － „Wann?“ － „Vor zwei Tagen.“
  - „Ich habe aufgehört zu rauchen.“ － „Wann?“ － „Vor zwei Tagen.“

### """ + "Adam" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - White, "Curly" Adam.

### """ + "Besitzer" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Die" Besitzer?

### """ + "Briefe" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum kommen immer "private und vertrauliche" Briefe?

### """ + "Carter" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie wollten mich "Brush" Carter nennen.

### """ + "Danke" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für deine "Akte X." Danke.

### """ + "Das" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Unheimlich" Das ist genau der richtige Ausdruck.

### """ + "Eddie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der einzigartige Clifton "Fou Fou" Eddie am Schlagzeug.

### """ + "Familie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will das Baby keiner "gesunden" Familie geben.

### """ + "Geld" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was ist mit "dem" Geld.

### """ + "Ja" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Wie lang kann man überleben?" Ja.

### """ + "Kerl" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Über "den" Kerl.

### """ + "Komm" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - " Komm, Kleine, zeigen wir ihnen, wie man schwimmt."

### """ + "Kunst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie studierte an der "Illinois State University" Kunst.

### """ + "Menschen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hör auf, mich als einen "normalen" Menschen anzusehen!

### """ + "Nachrichten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Noch mehr "gute" Nachrichten?

### """ + "Oskar" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Orgie" Oskar lesen.

### """ + "Ratford" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist mein Assistant Deputy in Charge Randy "the Snake" Ratford.

### """ + "Sachen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Mädchen können ihre "persönlichen" Sachen mitnehmen.

### """ + "Scheiße" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nach all der "Ja, ja" Scheiße?

### """ + "Schönheitskönigin" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Die" Schönheitskönigin.

### """ + "Suppe" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine "sauer-scharfe" Suppe.

### """ + "Technik" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hast die Meisterschaft mit seiner "schlechten" Technik gewonnen.

### """ + "Theorie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ihre "wissenschaftliche" Theorie.

### """ + "Tom" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und bei "Tom und Jerry" Tom, ja?

### """ + "Und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Abrakadabra!" Und drei Tauben flogen aus seinem Hut.

### """ + "Wilson" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Stanley "Das Werkzeug" Wilson.

### """ + "abonniert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe "Elle" abonniert.

### """ + "abrufen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das "Initialenprogramm" abrufen.

### """ + "all" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kein "Mist", kein "Scheiße" all die Jahre.

### """ + "allein" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und heißt "frei sein" allein sein?

### """ + "also" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein simples "Hin und weg" also.

### """ + "angeht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was den "Antillais" angeht, der ist nur ein gewöhnlicher Zuhälter.

### """ + "angesagt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In der Casa Amarela ist gerade "das haarige Bein" angesagt.

### """ + "angezogen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Solch ein toller Player, und du bist wie ein "Abgesaugter" angezogen.

### """ + "angreife" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eigentlich müsstest du erwarten, dass ich mit "Capo Ferro" angreife.

### """ + "anheben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich musste das Paniklevel auf "extrem nervös" anheben.

### """ + "anhören" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da ist irgendeiner, der will sich immer wieder das Lied "Sad Letter" anhören.

### """ + "ansehen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Heute muss ich mir "FBI" ansehen.

### """ + "aufführen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du kannst "Die Abhängigkeit" aufführen.

### """ + "aufgegessen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Haben die das "Plamm" aufgegessen?

### """ + "ausgeben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Führer soll heute noch "Clausewitz" ausgeben.

### """ + "ausgesprochen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es wird "Hastings" ausgesprochen.

### """ + "ausspricht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Egal wie man "Oppenheimer" ausspricht, jeder weiß, ich bin Jude.

### """ + "auswendig" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du kannst "Whiplash" auswendig?

### """ + "bedeuten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich glaube, ich weiß, was "Wenn du es baust, wird er kommen" bedeuten soll.

### """ + "befolgen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Akustisches Protokoll "Ruhe" befolgen.

### """ + "bei" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du "logierst" bei den Micawbers?

### """ + "benutzen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich werde nicht den Begriff "Intelligenz" benutzen.

### """ + "benutzt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieser Junge hat sie für einen "snuff" benutzt.

### """ + "bezeichnen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Würden Sie sie als "mollig" bezeichnen?

### """ + "bezeichnet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als "Kranke Ostasiens" bezeichnet man uns.

### """ + "bezeichnete" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und sie war, was Mutter als "neureich" bezeichnete.

### """ + "bilden" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hier könnten die Gebäude ein "C" bilden.

### """ + "bin" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du weißt ja, dass ich nicht mehr bei "National" bin, oder?

### """ + "bist" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber ich könnte auch sagen, weil du "auch zu verrückt" bist.

### """ + "brüllt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Weißt du, wie das ist, wenn sie nur noch "Scheidung" brüllt?

### """ + "chinesische" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das "singuläre" chinesische Gericht, das ich kochen kann.

### """ + "dachte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Dreißig Dollar sind viel Geld für das kleine Zimmer," dachte er.

### """ + "darauf" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Suche ein Schild mit einer großen "Eins" darauf.

### """ + "darf" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Gut" darf nicht bloß ein Wort bleiben.

### """ + "davon" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Erfährt die Gewerkschaft "Muschi nur gegen Geld" davon, bin ich erledigt.

### """ + "definiert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein gewisser Philosoph hat einmal das "kollektive Unbewusste" definiert.

### """ + "dein" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er "pimpt" dein Rad ein bisschen.

### """ + "drücken" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Einfach auf "Play" drücken.

### """ + "du" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Vorwärts, Huskies!" du sagst es.

### """ + "durch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich halte nicht mal eine Folge von "Orange Is The New Black" durch.

### """ + "durcheinander" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bringen Sie nicht die beiden Begriffe "Subjekt" und "Objekt" durcheinander!

### """ + "dürfte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Hodenkrebs" dürfte klar sein.

### """ + "einfach" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich sage "sexy" einfach nicht mehr.

### """ + "eingegeben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du bei ChatGPT "fiktionaler Zauberername" eingegeben?

### """ + "einschalten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie lange wirst du noch deine "Eisen" einschalten?

### """ + "einvernehmelich" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - War diese "Entscheidung" einvernehmelich?

### """ + "entspricht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Krass, Alter" entspricht allem, was ich gehört hatte.

### """ + "erfüllt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Welches "E" erfüllt dich?

### """ + "ergeben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Siehst du, niemand würde sich dem Furchtbaren Piraten "Westley" ergeben.

### """ + "ernannt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cao Cao hat sich zum "Premierminister" ernannt.

### """ + "erwähnst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn du noch mal "Die kleine Lok" erwähnst, blase ich dir die Fresse weg.

### """ + "erzählt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was ich dir vorher mit dem "Bitte" erzählt habe, stimmte.

### """ + "fast" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hätten Sie in "Gesprengte Ketten" fast McQueens Rolle gespielt?

### """ + "fertig" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich muss "Räuber" fertig lesen.

### """ + "früher" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Meinst du, du kommst heute mal" früher aus der Schule raus?

### """ + "gab" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mein lieber Freund, "aus heutiger Sicht" gab's damals nicht.

### """ + "ganz" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Teils gewann die "Mutterhälfte" ganz die Oberhand.

### """ + "gebräuchlich" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Seit wann ist das Wort "Biotechnologie" gebräuchlich?

### """ + "gefällt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Wort "gigantisch" gefällt mir sehr.

### """ + "geheißen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und Sie, Herr Baader, Sie haben mich ein "faschistisches Arschloch" geheißen?

### """ + "gehen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will, dass Sie heute an die "Swansea-Drehbank" gehen.

### """ + "gekauft" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Habe ich bei "Grossman's" gekauft.

### """ + "gelegt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat einen Link zu "Mimas Zimmer" gelegt.

### """ + "gelernt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wo haben Sie "Xuan Pin" gelernt?

### """ + "genossen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wir haben "Tanner" genossen.

### """ + "gepingt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Von Miller wurde "Daumen hoch!" gepingt, von Dr. Mann auch.

### """ + "gespielt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du schon "Four Swords" gespielt?

### """ + "getauft" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also habe ich sie "Überdosis" getauft.

### """ + "getrunken" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jetzt wird ein "Miller" getrunken!

### """ + "getötet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Matías hatte den Anführer des "Comando Vermelho" getötet.

### """ + "geworden" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wann genau ist aus der Sonde denn ein "Er" geworden, Professor?

### """ + "gewählt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nein, ich wurde zur "Auffälligsten" gewählt.

### """ + "geändert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie hat ihren Namen in "Anne" geändert.

### """ + "gibst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei "Location" gibst du die URL ein.

### """ + "ginge" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Mein Führer" ginge auch.

### """ + "gäbe" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn es ein Verb namens "nachen" gäbe, wäre "Nacht!" der Imperativ davon.

### """ + "haben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die "Fliegenden Dolche" haben viele Meister.

### """ + "hast" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber das "Beruhigungsmittel" hast du noch bei dir.

### """ + "hatte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Bates Motel in "Psycho" hatte keine separaten Bungalows.

### """ + "hebt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber ich denke, dieser "Thibault" hebt "Capo Ferro" auf, du nicht?

### """ + "herauszulocken" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er sagte wir brauchen echtes Blut um die "Fliegenden Dolche" herauszulocken.

### """ + "herunterriss" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Gute Fee, sag nee" herunterriss.

### """ + "hervor" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und dann käme aus der Mündung ein Zettel mit dem Wort "Peng" hervor.

### """ + "holen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Apropos, wir sollten uns die Rechte an dem Wort "gut" holen.

### """ + "hängen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Freut mich sehr, dass du "Sabor" hängen hast.

### """ + "hätten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nur bei "Scheiße" hätten wir richtig Ärger bekommen.

### """ + "hören" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn ich euren Namen aufrufe, will ich ein deutliches "hier" hören!

### """ + "hört" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nein" hört Darryl sicher nicht oft.

### """ + "ihr" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Steht auf" ihr müsst das Knie nicht beugen "vor dem Thron der Gerechtigkeit."

### """ + "immer" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - An "toujours" immer noch ein "s".

### """ + "ins" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hast mir "aufwachen" ins Ohr gebrüllt.

### """ + "investiert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will keine Emotion verlieren, die du in "Die Abhängigkeit" investiert hast.

### """ + "ja" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ponys," ja.

### """ + "jedenfalls" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine bessere als "Pamela" jedenfalls.

### """ + "kannst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das mit "Enchante" kannst du vergessen.

### """ + "knallt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bacardi" knallt gut.

### """ + "kommst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit "Denken" kommst du nicht ans Ziel.

### """ + "kriegst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für den "Tip" kriegst du von mir auch einen Tipp.

### """ + "lautet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bei Fuß!" lautet eines der Kommandos, auf die sein Hund hört.

### """ + "lesen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In seinen Augen kann man schon jetzt "Amerika" lesen.

### """ + "los" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Man wird die "Last" los.

### """ + "lässt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieses "Sie" lässt mich im fünften Kreis der Hölle versinken.

### """ + "macht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schon das Wort "Heirat" macht mich krank.

### """ + "malen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich gehe "Das Jüngste Gericht" malen.

### """ + "markiert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir folgen keinen Karten zu Schätzen, und "X" markiert nie die Stelle.

### """ + "meinst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es kommt darauf an, was du mit an Gott "glauben" meinst.

### """ + "meinte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit "Absicht" meinte ich sehr koordiniert.

### """ + "morgen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Hunter starten, wenn die App "Aufzeichnen & Melden" morgen live geht.

### """ + "nannte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber es gab einen Grund, warum er Sie "meine Gewitterwolke" nannte.

### """ + "neben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Im Übrigen hat jemand "Kopf abnehmen" neben mein Mikrofon geschrieben.

### """ + "nenne" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich hoffe, es stört Sie nicht, dass ich Sie "Sohn" nenne.

### """ + "nennst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Unglück" nennst du das?

### """ + "noch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du zeigst den "Weißen Hai" noch mal?

### """ + "nochmal" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich lese "Der Graf von Monte Christo" nochmal.

### """ + "rassistisch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ist die Bezeichnung "weißer Mann" rassistisch?

### """ + "reicht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ich kann nicht klagen" reicht mir voll auf.

### """ + "reingeschnitten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vorgestern hatte er Geschlechtsteile in "Cinderella" reingeschnitten.

### """ + "reist" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn du "in die Provinz" reist, dann weißt du, wo du übernachtest.

### """ + "repariert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ob sie wohl das "A" repariert haben?

### """ + "restauriert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Cinemathek hat "Sabor" restauriert.

### """ + "riefe" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und plötzlich höre ich ihre Stimme, als ob sie "Danny" riefe.

### """ + "sage" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und was ist, wenn ich "nein" sage?

### """ + "sang" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Betrunken, als Chavela Vargas "La Noche de mi Amor" sang.

### """ + "scheint" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Lazlo" scheint hierüber unglücklich.

### """ + "schicken" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie zieht gerade um, also "postlagernd" schicken?

### """ + "schläft" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Madame "völlig fertig" schläft bei dir.

### """ + "sehen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als ich klein war, wollte ich unbedingt "Der Weiße Hai" sehen.

### """ + "sendet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die "Voice of America" sendet aus Washington.

### """ + "singe" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn ich dann "Eye of the Tiger" singe, verliert ihr eure Hemmungen.

### """ + "sonstwohin" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Steck dir dein "Hi" sonstwohin.

### """ + "sprechen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will mit dem "Antillais" sprechen, verstanden?

### """ + "stellen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Phaser auf "Betäuben" stellen.

### """ + "stinkt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nattō" stinkt furchtbar, aber schmeckt ausgezeichnet.

### """ + "studiert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sofern der Gegner nicht seinen "Agrippa" studiert hat.

### """ + "stürze" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf "Die Abhängigkeit" stürze ich mich.

### """ + "tun" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Wir" tun gar nichts.

### """ + "umschließt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Suizid begehen" umschließt beides, den Versuch und die Vollendung.

### """ + "unbedingt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fernando will den "Weißen Hai" unbedingt sehen.

### """ + "unter" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Seit Anbeginn der Menschheit gibt es die "Anderen" unter uns.

### """ + "verdient" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ja, er hat "lebenslänglich" verdient.

### """ + "versorgt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der "Antillais" versorgt hauptsächlich den Großraum Paris.

### """ + "viel" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie hält von "Apparat Mensch" viel.

### """ + "vorkommt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt keine englischen Sätze bei Tatoeba, in denen das Wort "Lego" vorkommt.

### """ + "vorne" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit einem "Kuji" vorne dran?

### """ + "vorsingen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Soll ich erst den Titelsong aus "Starsky & Hutch" vorsingen?

### """ + "warf" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "The Baked Potato" warf ihn weg.

### """ + "weglassen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das "wertschätzen" weglassen?

### """ + "weitermachen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich glaube, wenn alles gut läuft, können wir bald mit "Enchante" weitermachen.

### """ + "wollte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und mit "Hauptact" wollte ich euch beeindrucken.

### """ + "zugrunde" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Uns so ging unsere "Zwangsbildung" zugrunde.

### """ + "zuvor" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich war dort, wo noch kein "Totenkopf" zuvor gewesen war.

### """ + "zweimal" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe "Star Wars" zweimal gesehen.

### """ + "überhaupt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was bedeutet denn "Woo-Sung" überhaupt?

### """ + "–" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Was geschah hier?" – "Nichts."

### """ + "„" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Diese Schlampe hat mir in die Eier getreten." „Du hattest es verdient."

### ""Bounty Law"" + "zusammen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir arbeiten seit den letzten zwei Staffeln "Bounty Law" zusammen.

### "'s" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Etwas von Kaki-Ma's "Shondesh"?

### "'s" + "‚" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Chef, ich bin's‚ Lölik!

### "-" + "Jet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alles, von der kleinsten Cessna bis zum größten "weiß der Geier"- Jet.

### "-" + "Millionen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fresst Scheiße - Millionen von Fliegen können nicht irren.

### "-" + "als" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt weit mehr Yuan- als Euromillionäre auf der Welt.

### "-" + "angekommen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich bin kurz nach dem Massaker an den Flüchtlingen hier in Deressa - angekommen.

### "-" + "ausgerechnet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe sie zufällig getroffen - ausgerechnet in der Mongolei.

### "-" + "bringst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nächstes Mal - bringst du ihn rauf und wir erledigen ihn.

### "-" + "da" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Augenblick - da ist einer an der Tür.

### "-" + "dann" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie flackern wieder auf, verbrauchen irre viel Energie - dann gehen sie aus.

### "-" + "dass" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir können froh sein, - dass wir uns lebend wiedersehen.

### "-" + "deine" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du kannst gehen oder bleiben - deine Wahl.

### "-" + "den" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich werde ihn bitten, - den Auftrag vorzuziehen.

### "-" + "ehrlich" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe nur Augen für dich - ehrlich.

### "-" + "einen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir haben - wortwörtlich - einen Haufen zu Essen.

### "-" + "erforscht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn ihr euren Bruder finden wollt, - erforscht die Vergangenheit eurer Mutter.

### "-" + "erzähl" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dein Bruder meldet sich wieder - erzähl ihm von deinen Geschichten.

### "-" + "folgte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn er hier war, - folgte ich ihm auf Schritt und Tritt.

### "-" + "für" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ihre Stärke ist, dass sie sich - für Frau Mak hält.

### "-" + "je" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Japaner benutzen unterschiedliche Wörter, - je nachdem mit wem sie sprechen.

### "-" + "jetzt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber wie dem auch sei, - jetzt ist alles klar.

### "-" + "kann" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nur wenn ich glaubwürdig bleibe, - kann ich in seine Seele eindringen.

### "-" + "lasse" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber wenn er etwas hat, was wir brauchen, - lasse ich ihn länger leben.

### "-" + "mehr" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Denn ich weiß - mehr als ihr.

### "-" + "mein" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Verdammt - mein Auto ist geklaut!

### "-" + "meint" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn er wieder anruft, - meint er es ernst.

### "-" + "mitte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Haltet das Floss in der Fluss- mitte.

### "-" + "nach" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und dann schrieb Vater, er könne mir die Reise - nach England nicht bezahlen.

### "-" + "noch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich kann Treppen weder hoch- noch hinuntersteigen.

### "-" + "ok" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hey, Gui- ok, schalt doch bitte das Radio aus.

### "-" + "seine" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das heißt für mich, dass ich - seine Geliebte werde.

### "-" + "sieht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schaut man sie an, - sieht man, dass sie alle etwas gemeinsam haben.

### "-" + "steht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn die drei Planeten sich verfinstern, - steht das schwarze Loch offen.

### "-" + "um" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es geht um etwas wichtigeres als die Solaristik - um unser Erkenntnisvermögen.

### "-" + "vom" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir kamen - vom Festland, als der Krieg anfing.

### "-" + "vorne" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle gehen - vorne durch.

### "-" + "was" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wa-- was?

### "-" + "weggeschickt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat Tsao und den Leibwächter - weggeschickt.

### "-" + "wird" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn der Druck zu groß ist - wird das Boot zerquetscht.

### "-" + "wortwörtlich" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir haben - wortwörtlich - einen Haufen zu Essen.

### "-" + "zur" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Im Herzen eines Agenten ist nur Platz für die Treue - zur Partei, seinem Land.

### "-" + "zweimal" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht Lungenentzündung, ein- zweimal.

### "-" + "überfliegt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Staffel - überfliegt Deutschland in einer Nachtmission.

### ";" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - GROẞVATER; "Buttercups Leere verzehrte sie."

### "<<" + "entsprechen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wieso <<entsprechen meine Angaben der Wirklichkeit fast gar nicht>>?

### "A" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „A“ ist der erste Buchstabe des Alphabets.

### "Aber" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber "fettes Schwein" sagt man nicht.

### "Abteilung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Abteilung "Phänomene".

### "Ach" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ach … Sie … Sie hat's nicht überstanden.

### "Adasdotr" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Adasdotr“, antwortete er.

### "Afrikaans" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Südafrika wird auf Afrikaans "Suid-Afrika" genannt.

### "Agentur" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Spreche ich mit der Agentur "The Care"?

### "Ah" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ah“ ist eine Interjektion.

### "Aldina" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aldina !

### "Allerdings" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Allerdings«, antwortete Chiang.

### "Allerdings" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Allerdings“, erwiderte Heilerin Agatha.

### "Als" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als "Kranke Ostasiens" bezeichnet man uns.

### "Also" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also … Also, du weißt schon.

### "Alter" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein Mann in seinem Alter !

### "Amanda" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Amanda«, begrüßte mich Antonio.

### "Amanda Kirasdotr." + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hier ist unser Kapitän, Amanda Kirasdotr.“

### "Amerikas" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Stimme Amerikas“ sendet aus Washington.

### "An" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - An "toujours" immer noch ein "s".

### "Anarchokapitalisten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom bezeichnet sich als „Anarchokapitalisten“.

### "Anfang" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hättest gleich am Anfang "Gratis-Europa-Reise" sagen können.

### "Angelegenheiten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben noch andere Angelegenheiten“, fuhr der Grafschaftsbibliothekar fort.

### "Angst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er weiß nicht, was das Wort „Angst“ bedeutet.

### "Animatron" + "ics" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Animatronics kann hier weg, nicht einmal du.

### "Anime" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er mochte Zeichentrickfilme japanischer Art, auch „Anime“ genannt.

### "Anklagepunkte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Anklagepunkte“, las ich.

### "Ankunfts-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf dem Plan dort sind Ankunfts- und Abfahrtszeiten aufgeführt.

### "Anleger" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dafür gibt es doch Anleger“, antwortete er.

### "Anlegestelle" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich nehme an, Ausbaggern zählt als Reparatur einer Anlegestelle“, sagte ich.

### "Anne" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist ein Weihnachtsgeschenk für dich, Anne“, sagte Matthew schüchtern.

### "Anredeformen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich fürchte, wir verwenden unterschiedliche Anredeformen“, sagte Saunders.

### "Antonio" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Fast hätte ich „Antonio“ gesagt, aber das ging ihn nichts an.

### "Antwort" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei der ersten Frage wählte sie Antwort "B".

### "Antworten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lesen Sie unsere „Fragen und Antworten“.

### "Apfel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir bezeichnen New York als den „dicken Apfel“.

### "App" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Hunter starten, wenn die App "Aufzeichnen & Melden" morgen live geht.

### "Applaus" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Applaus "Und das war der Oxer!"

### "Aquarium" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das große Aquarium "Ocean Park" am Tigerstrand in Dalian ist bereits eröffnet.

### "Arbeit" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom trägt immer einen Schlips bei der Arbeit .

### "Arbeit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arbeit“ und „Familie“ waren für sie Synonyme.

### "Arbeits-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir beten für die Armen, die Hungerleidenden, die Arbeits- und Obdachlosen.

### "Arktos" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arktos“ heißt auf Griechisch Bär.

### "Armenhilfe" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Stadt hat Armenhilfe … aber wie vielen Menschen kann sie helfen?

### "Art" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich mache Ihnen was in der Art "Schmutziger Schnee".

### "Arzt-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine, die Arzt- und Krankenhauskosten abdeckt, mit zusätzlichem Krankengeld.

### "Aspekt" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieser Tisch ist gut bis auf einen Aspekt - er passt nicht in mein Zimmer.

### "Astronaut" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „Astronaut“ kommt aus dem Lateinischen und bedeutet „Sternenschiffer“.

### "Atlantik" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hotel «Atlantik».

### "Auch" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auch "Schwein" genannt.

### "Auf" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf "Die Abhängigkeit" stürze ich mich.

### "Aufseher" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aufseher“, erwiderte ich.

### "Aufstehen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dies frühzeitige Aufstehen«, dachte er, »macht einen ganz blödsinnig.

### "Augenblick" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Augenblick - da ist einer an der Tür.

### "Aus" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aus "Bounty Law".

### "Auskunft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie hat Anspruch auf diese Auskunft“, schaltete Pavelson sich ein.

### "Ausland" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Buch „Englischunterricht im Ausland“ war für mich nicht von Nutzen.

### "Auto" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Können wir das Auto "Hae-Jin Express" nennen?

### "Automobil" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Automobil“ ist eine Hybridbildung.

### "B" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Relation „A liebt B“ ist weder reflexiv noch transitiv noch symmetrisch.

### "Badstraße" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nehmen Sie den Bus Richtung „Badstraße“ bis zur Seestraße.

### "Band" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kennen Sie die Band "The The"?

### "Barlow" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Den gibt’s nur bei Barlow“, sagte er.

### "Barlow’s" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Barlow’s“, antwortete Piotr.

### "Barrys" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das da ist der Teich der Barrys“, sagte Matthew.

### "Barzahlung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Barzahlung“, sagte er.

### "Bau" + "‹" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Crescent Dock war ›im Bau‹, aber jahrelang war dort nichts geschehen.

### "Bauernhof" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich arbeite auf einem Bauernhof“, sagte Tom.

### "Baum" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Nacktärsche nennen diesen Baum "caoutchouc", "der Baum, der weint".

### "Baustelle" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die ewige Stadt - das ist Rom; und die immerwährende Baustelle - das ist Berlin.

### "Be-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die größte Herausforderung würde das Be- und Entladen sein.

### "Beförderungen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe eine Liste mit Beförderungen“, sagte Sparhawk.

### "Begriffe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bringen Sie nicht die beiden Begriffe "Subjekt" und "Objekt" durcheinander!

### "Begründung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und schreib als Begründung "mangelnde Verdachtsmomente".

### "Belauschen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie haben kein Gesetz gegen das Belauschen“, sagte Higginbotham.

### "Benehmen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was für ein Benehmen !

### "Berg-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom fuhr mit der Berg- und Talbahn.

### "Berufs-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein ausgewogenes Verhältnis zwischen Berufs- und Privatleben ist wichtig.

### "Beseda" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Beseda“ bedeutet auf Slowenisch „Wort“.

### "Besitzer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich hole den Besitzer“, sagte sie.

### "Besuch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das hier ist unser zweiter Besuch“, sagte ich.

### "Besucher" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir hatten schon andere Besucher“, sagte ich, „also verstehe ich eure Fragen.

### "Bevölkerungs-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hinsichtlich Bevölkerungs- und Nahrungsproblemen ist der Nutzen eindeutig.

### "Beweise" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hier gibt es kaum Beweise“, verkündete er.

### "Bezeichnung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ist die Bezeichnung "weißer Mann" rassistisch?

### "Bienen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gold nennen sie "Schweiß der Sonne", die Bienen "Väter des Honigs".

### "Bier" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In Berlin heißt Bier "Molle".

### "Bitte" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Und jetzt habe ich eine Bitte«, verkündete Michael.

### "Bitte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „Bitte“ auf Wolof?

### "Blautöne" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Blautöne“, sagte ich, „und die Violetttöne, obwohl die etwas teuer sind.“

### "Blues-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Besitzt eine Kette von Blues- und Rockclubs in Chicago.

### "Blut" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Blut … Blut.

### "Boden-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Beide Holzlager der Stadt hatten einen Überschuss an Boden- und Wandbalken.

### "Bolashak" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das „Bolashak“-Programm ist sehr nützlich.

### "Bouquet" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mein Bouquet !

### "Brand-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - So werden keine Brand- oder Schnittwunden oder schmutzigen Ärmel mehr vorkommen.

### "Brasiliens" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Stadt Recife ist als „Venedig Brasiliens“ bekannt.

### "Braut" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wo ist die Braut ?

### "Bravo" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bravo !

### "Brut" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie mag es nicht, wenn ich ihre Kinder als ihre „Brut“ tituliere.

### "Bräutigams" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und jetzt singe ich auf Wunsch des Bräutigams "Ich brauche Liebe".

### "Buchhandlungen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „In Eastport gibt es nicht genug Geschäft für zwei Buchhandlungen“, sagte er.

### "Bucht" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Wellen waren »offenes Meer«, nicht »Äußere Bucht«, aber noch zu bewältigen.

### "Bug-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich warf Bug- und Heckanker aus, beide mit doppelter Leine.

### "Bushidō" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Samurai lebten nach einem Ehrenkodex, der „Bushidō“ hieß.

### "Bäume" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich liebe Bäume“, sprach das Eichhörnchen.

### "Bücherlieferung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Bücherlieferung“, erwiderte ich.

### "Büro" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das „ovale Büro“ befindet sich im Weißen Haus.

### "Camden Town" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ist die nächste Haltestelle „Camden Town“?

### "ChatGPT" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du bei ChatGPT "fiktionaler Zauberername" eingegeben?

### "Chavela Vargas" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Betrunken, als Chavela Vargas "La Noche de mi Amor" sang.

### "Cheese" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Guckt in die Kamera und sagt „Cheese“!

### "Chickanook" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Apotheke «Chickanook».

### "Chirurg" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Im Gefängnis wurde Tom der „Chirurg“ genannt.

### "Chirurgen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben einen Chirurgen“, sagte ich.

### "Chris" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Wir tun das für dich, Chris«, hielt ich dagegen.

### "Christa" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Christa—Maria Sieland.

### "Clair de lune" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Clair de lune“ (Mondschein) wurde von Claude Debussy komponiert.

### "Clifton" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der einzigartige Clifton "Fou Fou" Eddie am Schlagzeug.

### "Club" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sein Hauptquartier ist im Club "Metro".

### "Cocktail" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich glaube, ich nenne den Cocktail "Vesper".

### "Corpolo" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Scureza di Corpolo !

### "Cosa Nostra" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Mafia wird auch als die „Cosa Nostra“ bezeichnet.

### "Cowboys" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du nennst sie „Mavericks“; ich nenne sie „Cowboys“.

### "Craig" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Weißt du, in Großbritannien spricht man Craig "Creig" aus.

### "Da-songs" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist Da-songs "Trauma-Überwindungs-Torte".

### "Dackel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dorgan wusste nicht, wie man „Dackel“ schreibt.

### "Dad" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Schwägerin von Clorindas Dad“, ergänzte Michael.

### "Dank" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Na, vielen Dank«, sagte ich.

### "Dank" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gott sei Dank“, sagte Clorinda.

### "Dehn-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zuerst ein paar Dehn- und Streckübungen.

### "Deressa" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich bin kurz nach dem Massaker an den Flüchtlingen hier in Deressa - angekommen.

### "Deutschland" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Deutschland wird auf Deutsch »Deutschland« genannt.

### "Diabetes-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fettleibigkeit erhöht das Diabetes- und Herzkrankheitsrisiko.

### "Diana" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist meine kleine Diana“, sagte Frau Barry.

### "Dihydrogenmonoxid" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dihydrogenmonoxid“ ist die allzu technische Bezeichnung für Wasser.

### "Dilemma" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.

### "Dilemmas" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.

### "Dilemmata" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.

### "Direktor" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich biete an, sie zu unterrichten, Direktor“, sagte ich.

### "Docks" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dann sind da noch die Reparaturen an den Docks“, sagte ich.

### "Docks" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Über die Docks … was?

### "Domingo" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Domingo“ ist das spanische Wort für „Sonntag“.

### "Dostojewskis" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie Polina aus Dostojewskis "Der Spieler"!

### "Drache" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Rote Drache“ ist eine allseits bekannte Glücksspiel- und Opiumhöhle.

### "Dreieck" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist schwierig „Dreieck“ zu definieren.

### "Dreifaltigkeitsplatz" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Dreifaltigkeitsplatz“ war das ursprüngliche Zentrum von Sankt Petersburg.

### "Dreimal" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dreimal "Entschuldigung" und es ist aus.

### "Drogen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Sache mit dem internationalen Drogen- und Mädchenhandel.

### "Du" + ""logierst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du "logierst" bei den Micawbers?

### "Du" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du“, sagte ich und gab ihm einen kleinen Stoß mit dem Knie, „schlaf nicht ein!“

### "Du" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du … Vielleicht bist du ja doch ein Lynxley.

### "Duce" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Heil dem Duce !

### "Dundonian" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Rob ist ein stolzer „Dundonian“.

### "Durchgedreht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Toms Lieblingslied ist „Durchgedreht“ von der Musikgruppe „Die Eichhörnchen“.

### "Edelsteine" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schöne Kleider“ sagte die eine, „Perlen und Edelsteine“ die zweite.

### "Eichkatze" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Marias Lieblingssuperheldin ist die „Eichkatze“.

### "Einen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Einen "Unfall"?

### "Einkaufen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einkaufen“, sagte Kat.

### "Einlagenkonten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diesmal haben wir nur eine Reihe von Einlagenkonten“, sagte Kat.

### "Einsamkeit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ist das Gegenteil von „Einsamkeit“?

### "Einverstanden" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Einverstanden«, verkündete er.

### "Einverstanden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einverstanden“, sagte ich.

### "Elefantenhode" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ist es eine Elefantenhode ?

### "Elektrizität" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ihr habt keine Elektrizität“, sagte Arthur, „und wir haben keine Bücher.

### "Eltern" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ehrst du deine Eltern ?

### "Energie" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie flackern wieder auf, verbrauchen irre viel Energie - dann gehen sie aus.

### "Englisch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich brauche ein Wörterbuch „Japanisch-Englisch“.

### "Englischen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das nennt man im Englischen "day for night".

### "Entschuldigung" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Entschuldigung«, stand auf dem Zettel in meinem Briefkasten.

### "Er" + ""pimpt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er "pimpt" dein Rad ein bisschen.

### "Erderwärmung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sollte ich „Klimawandel“ oder „Erderwärmung“ sagen?

### "Ernie" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ernie“, rief Piotr, „Kunden aus Spencerport.

### "Erwachsene" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich hasse es, wenn Erwachsene "sexuell aktiv" sagen.

### "Erwachsenen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist ein Erwachsenen- und kein Kinderfilm.

### "Erwischt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erwischt“, antwortete Kat.

### "Es" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es … Judy, es lohnt sich nicht, dafür zu sterben.

### "Esel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Apuleius schrieb „Der goldene Esel“.

### "Ess-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich schicke dir bei gleicher Gelegenheit Ess- und Rauchwaren.

### "Essen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber sie servieren Essen“, sagte ich.

### "Ethik" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ethik“ bedeutet die Richtlinien des Handelns.

### "Eule" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie lautet das griechische Wort für „Eule“?

### "Falken" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Löwen“ haben gegen die „Falken“ einen leichten Sieg errungen.

### "Familie" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arbeit“ und „Familie“ waren für sie Synonyme.

### "Farbe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Man nennt diese Farbe "Mitternachtsblau".

### "Farbstoffe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Farbstoffe“, sagte sie.

### "Farbton" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du den Farbton "Kitten Kisses"?

### "Farm" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich arbeite auf einer Farm“, sagte Tom.

### "Feilschen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es geht nicht ums Feilschen“, erwiderte ich.

### "Feuergottheit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vom „Kagura-Tanz der Feuergottheit“ höre ich auch zum ersten Mal.

### "Finger" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Meinen Ring von dem Finger“ antwortete das Mädchen.

### "Firma" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir eröffnen die Firma "Gebrüder Bagrow".

### "Fisch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es gibt zu wenig Fisch“, verkündete er.

### "Flammen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Flammen- und Wasser-Schwertkämpfer.

### "Flammenatmung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Flammenatmung“ hat eine lange Geschichte.

### "Flasche" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gib mir noch eine Flasche“, sagte er.

### "Fock-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Klar zum Setzen der Fock- und Großmarssegel“, rief ich.

### "Forbes" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe den Artikel in „Forbes“ gelesen.

### "Fortgeschrittenen-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Haben Sie ein Fortgeschrittenen- oder ein Anfängerniveau?

### "Fox News" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf sämtlichen Fernsehergeräten laufen „Fox News“.

### "Frage" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wichtiger als die Frage "Was?"

### "Frage" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es gibt noch eine andere Frage“, verkündete William Williamson.

### "Fragen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was sind denn das für Fragen ?

### "Fragen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe keine weiteren Fragen“, sagte Brunel.

### "France" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Frankreich heißt auf Französisch „France“.

### "Französisch-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom und Maria geben beide Französisch- und Englischunterricht.

### "Frau" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will eine Frau !

### "Frauen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie nennen alle Frauen "Miezen".

### "Frauen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber, sind die Frauen- und Männerbereiche getrennt?

### "Fremden" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Leigh, komm, sag unserem geheimnisvollen Fremden "hallo".

### "Freunden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Beim Übersetzen muss man vor „falschen Freunden“ auf der Hut sein!

### "Freundinnen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Für Freundinnen«, sagte ich.

### "Frosch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf der Seite stand lediglich das Wort „Frosch“.

### "Frühling" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es komme der Frühling !

### "Füchse" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wasserbewohner wie auch Füchse … sind niedere Säugetiere.

### "Für" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für "Sasaki".

### "Für" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Für'n Arsch.

### "Für" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für … sie alle?

### "Fürst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wer hat „Der Fürst“ geschrieben?

### "Ganzjahres" + "job" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist ein Vollzeit- und Ganzjahresjob.

### "Gatsby" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe auf der Oberschule „Der große Gatsby“ gelesen.

### "Geburten-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Geburten- und Sterberate waren fast gleich.

### "Gejaul" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gejaul … beim Empfang.

### "Geld" + "verschwendung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist Zeit- und Geldverschwendung.

### "Geldbörse" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer fand meine Geldbörse ?

### "Geldsack" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom wurde von seinen Freunden „Herr Geldsack“ genannt.

### "Gemurre" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich höre Gemurre“, sagte Clyde Rogerson.

### "Genossenschaft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es ist eine Genossenschaft“, sagte Missy Marky.

### "Genug" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Genug !

### "Georgier" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vitya, wie sagt ein Georgier "Warte"?

### "Geschichte" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Lange Geschichte«, verkündete ich.

### "Geschäftspartner" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bei dem Schiff waren sie Geschäftspartner“, sagte ich.

### "Gesicht" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er fand mein Gesicht "konventionell".

### "Getreide" + "mühlen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Und wo sind all unsere Holz- und Getreidemühlen?«, fragte ich.

### "Gewerkschaft" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Erfährt die Gewerkschaft "Muschi nur gegen Geld" davon, bin ich erledigt.

### "Gibt's" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gibt's "Flügel" von Nautilus, auf CD?

### "Gina" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gina !

### "Glatzkopf" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und schicke den Tschubs ein Foto von dem Glatzkopf … dem Tataren.

### "Gluten-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gluten- oder Laktoseintoleranz?

### "Glücksspiel-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der „Rote Drache“ ist eine allseits bekannte Glücksspiel- und Opiumhöhle.

### "Goddard" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Willkommen auf Goddard“, sagte ich.

### "Gold" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „Geld“ oder „Gold“ gesagt?

### "Goldenen Drachen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom hat im „Goldenen Drachen“ gegessen.

### "Gott" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es kommt darauf an, was du mit an Gott "glauben" meinst.

### "Gramophon" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und du weisst nichts über das Gramophon ?

### "Gravitations-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Juno wird die Gravitations- und Magnetfelder des Jupiter vermessen.

### "Großmarssegel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Klar zum Setzen der Fock- und Großmarssegel“, rief ich.

### "Gut" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gut“, sagte sie.

### "HUNDE" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - KEINE JUDEN KEINE HUNDE "Eintritt verboten für Juden und Hunde."

### "Haar-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie kaufte eine Haar- und eine Zahnbürste.

### "Hackmesser" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Meine Frau besitzt mehrere Hackmesser“, ergänzte Antonio.

### "Hafenmeister" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich grüße Euch, Hafenmeister«, sagte er.

### "Hai" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hai“ heißt auf Lettisch „haizivs“.

### "Haifisch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haizivs“ ist das lettische Wort für „Haifisch“.

### "Haizivs" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haizivs“ ist das lettische Wort für „Haifisch“.

### "Hallo" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Hallo«, brachte ich hervor.

### "Hallo" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hallo“, rief ich.

### "Hallo" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hallo … Nicholas.

### "Hals-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hals- und Steinbruch.

### "Halsband" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Halsband“ sagte das Mädchen.

### "Hand-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein Verkehrspolizist regelt mit Hand- und Armzeichen den Verkehr.

### "Handy" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hey, das Handy … ist das Erste, was die tracken.

### "Hansen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich kann die Akte „Tom Hansen“ nicht finden.

### "Harborview House" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Da ist Harborview House“, sagte ich.

### "Hass" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Liebe“ und „Hass“ sind Gegensätze.

### "Hau" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hau'n wir ab.

### "Haus-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wirst für immer diese Haus- und Telefonnummer vergessen.

### "Hause" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Herr ist nicht zu Hause“, sagte ihr das Dienstmädchen.

### "Haushalts-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Keine Haushalts- und schon gar keine Eisenwaren.

### "Heck-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Heck- und Bugmannschaften!

### "Heil Hitler" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Oder seine Briefe mit „Heil Hitler“ zu unterschreiben.

### "Heimat" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heimat“ ist ein deutsches Wort ohne englische Entsprechung.

### "Heißt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Heißt "erstes Oben“, dass er im letzten Jahr verstorben ist?

### "Herr" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich danke Euch für mein Leben, Herr“, fuhr er fort.

### "Herz-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das könnte eine Herz- oder eine Gehirnkrankheit sein.

### "Hexe" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Brenne nieder, du alte Hexe !

### "Hilfe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „Ich brauche Hilfe“?

### "Hindenburg" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Hindenburg“ ging plötzlich in Flammen auf.

### "Hit" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann lehnen Sie sich zurück und genießen Sie Chicagos Hit "If You Leave Me Now".

### "Hitler" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und dann dürft ihr eurem Onkel Hitler "Guten Tag" sagen.

### "Hochspannung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Schilder, auf denen „Hochspannung“ steht, sind häufig gelb.

### "Hochzeit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darum ging es also bei den Scherzen auf der Hochzeit“, sagte ich.

### "Hohlgramm" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es ist ein Hohlgramm“, erklärte Vincent.

### "Holmes" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ihr Name kommt mir sehr bekannt vor, Herr Holmes“, sagte er mit einem Lächeln.

### "Hotel" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie war oft in Shanghai und wohnte immer im Hotel "Ostasien".

### "Hotel Tatoeba" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Lieblingslied ist „Hotel Tatoeba“.

### "Humor" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Man nennt es „kranken Humor“.

### "Hund" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir nannten den Hund "Indiana"!

### "Hundeleben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was bedeutet „Hundeleben“?

### "Hut" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Siehst du die Dicke mit dem Hut ?

### "Hyberbel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hyberbel“ ist ein hochtrabender Ausdruck für „Übertreibung“.

### "Höhenzug" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hinter dem Höhenzug“, sagte Naismith.

### "Hühner-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hühner- und Schweinefleisch riecht sehr stark, also muss es vorgegart werden.

### "IHR" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - WIE GEHT ES IHR …durch die einfache Luftbewegung.

### "Idiot" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du musst saugen, Idiot !

### "Idiot" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja, wenn wir in Osaka jemanden beschimpfen, sagen wir auch „Idiot“.

### "Immer" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Immer "Ja, Sir" und "Nein, Sir".

### "Irgend" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Irgend'n atmosphärischer Firlefanz oder was Elektrisches.

### "Ist" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ist "Rank" die Abkürzung für "Franklin"?

### "Italien" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es lebe Italien !

### "JUMP" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieser Buchladen hat den Verkauf von „JUMP“ eingestellt.

### "Ja" + "‚" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja‚ Michail Iwanowitsch.

### "Jaguar" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie nannte das Kätzchen „Jaguar“.

### "Jahren" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das letzte Raumschiff kam vor zwanzig Jahren“, antwortete Chiang.

### "James" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - James“, sagte ich, „du gehst direkt zur Akademie.

### "Jeans" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich trage fast jeden Tag eine Jeans .

### "Jedenfalls" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jedenfalls … bis jetzt.

### "Jerry" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dir auch einen guten Abend, Jerry“, sagte ich.

### "Jeruschalajim" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Jerusalem heißt auf Hebräisch „Jeruschalajim“.

### "Jetboot" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Willkommen auf dem Yangtze-Jetboot "Forest".

### "Jill" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Jill“, sagte ich, „du bist mit Marika befreundet, nicht wahr?

### "Jim" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Jim“, sagte ich, „wie läuft es mit der Räumung des Überlaufs?“

### "Jones" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin Nicholas Jones“, sagte der Älteste der drei.

### "Junge" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du böser Junge !

### "Junge" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber wo soll ich denn jetzt eine Frau herkriegen, mein Junge ?

### "Junimonat" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es ist Junimonat“, sagte Borea.

### "Juno" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Raumsonde „Juno“ der NASA ist in eine Bahn um den Jupiter eingetreten.

### "Kabinen" + "druck" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kabinendruck fällt.

### "Kaffee-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bist du eher ein Kaffee- oder ein Teetrinker?

### "Kalles Speisewagen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Habt ihr schon mal in „Kalles Speisewagen“ diniert?

### "Kampagne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein Vorbild für meine Kampagne "Saubere Leinwand!"

### "Kampf" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Kampf“ ist ein Buch von Adolf Hitler.

### "Kapitel" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ganze Kapitel "Induktionsmotor".

### "Kat" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das war wirklich ein großartiges Feilschen, Kat“, sagte ich.

### "Katze" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „chat“ heißt auf französisch „Katze“.

### "Keith" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Na gut, Keith“, fragte ich mit breitem Grinsen.

### "Kerze" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Attilio, eine Kerze !

### "Kiew" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom und Maria aßen „Hühnchen Kiew“.

### "Kind" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da wir zusammen hier sind, einen weiteren Toast auf das Kind !

### "Kind" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er war so nett, ich habe als Kind "Onkel" zu ihm gesagt.

### "Kirasdotr" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin Amanda Kirasdotr“, verkündete ich.

### "Klageankündigung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Drei Tage später erhielt Suellen die erste „vorläufige Klageankündigung“.

### "Klar" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Klar“, sagte ich.

### "Kleider" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schöne Kleider“ sagte die eine, „Perlen und Edelsteine“ die zweite.

### "Klein-" + "als" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich würde lieber in einer Klein- als in einer Großstadt leben.

### "Klimawandel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sollte ich „Klimawandel“ oder „Erderwärmung“ sagen?

### "Knuffige" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieses Eichhörnchen soll fortan den Namen „der Knuffige“ tragen.

### "Kolumnist" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin Zeitungsreporter und Kolumnist“, sagte Maurice.

### "Kommerziell" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Kommerziell«, antwortete ich.

### "Kommt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kommt "Powerfrau" in deinen Romanen vor?

### "Kopf-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe Kopf- und Halsschmerzen.

### "Korruption" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Staatsstreiche, Mafia, Korruption - sind das nicht Ihre Kunden?

### "Kracher" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Holt die Kracher !

### "Krediten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es steckt alles in Krediten“, antwortete ich.

### "Kreisel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Am „Silizium-Kreisel“ sind zahlreiche junge IT-Unternehmen ansässig.

### "Kriegskunst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom hat „Die Kriegskunst“ von Sūnzǐ gelesen.

### "Kröte" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da ist eine Kröte !

### "Kunststoff-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kunststoff- und Glasflaschen müssen in die richtigen Behälter geworfen werden.

### "Kyōto" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir nennen diese Stadt „Klein-Kyōto“.

### "Kälte" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Brenne nieder den Winter und die Kälte !

### "Käufer" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Kein Käufer«, sagte Brunel.

### "Käufer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben noch keinen Käufer“, sagte Christopher.

### "Köder" + "e" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ködere den Wolf mit der Henne.

### "König" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Spöttisch „König“ genannt zu werden macht einen noch lange nicht zum König.

### "Kümmer" + "e" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kümmere Dich nicht darum.

### "Künste" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wo ist der „Palast der schönen Künste“?

### "La casa de papel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „La casa de papel“ ist die beste spanische Serie.

### "Laden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Fischerei bezahlt dafür, nicht der Laden“, sagte ich.

### "Lager" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie kamen wir mit „nichts mehr auf Lager“ zurecht?

### "Lallo" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lallo !

### "Lallo" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Möchtst du etwas mehr, Lallo ?

### "Lamberto" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lamberto“ ist der Name einer großen Hotelkette.

### "Land" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er starb für sein Land .

### "Land-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle Land- und Stadtgerichte haben dem Dail Treue geschworen.

### "Landefeld" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben das Landefeld“, sagte ich.

### "Lang-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sieht aus, als hätten sie eine Lang- und zwei Kurzwaffen.

### "Laus" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Mehrzahl von „Laus“ ist „Läuse“.

### "Lebens" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Francis Collins bezeichnet das Genom als das „Buch des Lebens“.

### "Lebens-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum ändern sich Lebens- und Denkweisen?

### "Leere" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wenn ich auf meine Jugend zurückblicke, sehe ich spirituelle Leere“, sagte Tom.

### "Lego" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es gibt keine englischen Sätze auf Tatoeba, die das Wort „Lego“ enthalten.

### "Leibwächter" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat Tsao und den Leibwächter - weggeschickt.

### "Leicester" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Leicester“ wird „Lester“ ausgesprochen.

### "Leichen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kapitänin, das sind Leichen“, meldete Larissa.

### "Leiter" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schnell, hol eine Leiter !

### "Lena" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lena“, sagte ich, „ich muss mit dir reden.“

### "Lester" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Leicester“ wird „Lester“ ausgesprochen.

### "Leute" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt hier Leute , die sagen, das sei die neue Normalität.

### "Leuten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich vertraue meinen Leuten“, sagte ich.

### "Lieber" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lieber "Bobbi", Herr Euclides.

### "Liebes" + "lied" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ist dein Lieblings-Liebeslied?

### "Liebes" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Amanda, Liebes“, begann sie, „du hast Bindfaden Nummer vier gekauft.“

### "Lieblings" + "attraktion" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das ist die Flussfahrt, eine von Charlottes Lieblingsattraktion.

### "Lieblings" + "aufleger" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wer ist dein Lieblingsaufleger?

### "Lieblings" + "bananennachtisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ist dein Lieblingsbananennachtisch?

### "Lieblings" + "erdbeernachtisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ist dein Lieblingserdbeernachtisch?

### "Lieblings" + "fernsehkoch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wer ist dein Lieblingsfernsehkoch?

### "Lieblings" + "liedermacher" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wer ist dein Lieblingsliedermacher?

### "Lieblings" + "regisseur" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ah, mein Lieblingsregisseur!

### "Liege-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zuerst wären da die Liege- und Hafengebühren.

### "Lietuva" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Litauen heißt auf Litauisch „Lietuva“.

### "Life" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „In My Life“ist das Lieblings-Beatles-Lied meiner Mutter.

### "Linke" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich hoffe, dass die SPD keine Koalition mit der Partei „Die Linke“ bilden wird.

### "Liste" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe eine Liste“, sagte Kat.

### "Lord" + "prokurator" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mr. Saunders hier ist ein leitender stellvertretender Lordprokurator.

### "Loricum" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Er war mit ziemlicher Sicherheit in Loricum“, sagte ich.

### "Luft" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich hab keine Luft !

### "Luft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hol erst einmal Luft“, sagte ich.

### "Lächeln" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lächeln !

### "Läden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es gibt Läden“, erwiderte ich.

### "Läuse" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Mehrzahl von „Laus“ ist „Läuse“.

### "Läusen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Glaubst du, ich fürchte mich vor diesen Schwarzhemden-Läusen ?

### "Lösung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich glaube, ich habe noch eine andere Lösung“, sagte ich.

### "Löwen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Löwen“ haben gegen die „Falken“ einen leichten Sieg errungen.

### "Madame" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Madame "völlig fertig" schläft bei dir.

### "Madeira" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Madeira“ ist ein Weinname.

### "Magazins" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie waren das Titelmodel des Magazins "Enchante".

### "Mal" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nächstes Mal - bringst du ihn rauf und wir erledigen ihn.

### "Mal" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich habe die Reparaturliste vom letzten Mal«, sagte ich.

### "Mama" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir singen einen Monat lang in der Galaxis die „Mama“, und er ist unser.

### "Margrethe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Margrethe“, sagte ich, „lass mir ein großes Stück Kuchen und ein Glas Wein da.

### "Maria" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie heißt du?“ – „Maria“, sagte das kleine Mädchen schüchtern.

### "Markofferson" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Chris Markofferson“, antwortete ich.

### "Martina Parker" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich bin Martina Parker«, sagte sie.

### "Mary" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Satz enthält zwei Subjekte, nämlich „Tom“ und „Mary“.

### "Maschinen" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann flogen ihre … Maschinen … ihr Segelboot hinunter zur Bucht.

### "Maschinenfabrik" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Genosse Wan arbeitet in der Maschinenfabrik "Ostwind".

### "Matrix" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie oft hast du „Matrix“ gesehen?

### "Mau" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mau“, sagte die Katze und sah mich an.

### "Mavericks" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du nennst sie „Mavericks“; ich nenne sie „Cowboys“.

### "Maße" + "n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom trinkt nie in Maßen, sondern immer zu viel.

### "Medizin" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die letzte Medizin - war schnell verkauft.

### "Meer" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Wellen waren »offenes Meer«, nicht »Äußere Bucht«, aber noch zu bewältigen.

### "Meerjungfrau" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die kleine Meerjungfrau“ ist der Titel eines dänischen Märchens.

### "Mein" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mein "Cigarral"!

### "Meister" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Kein Gott, kein Land, kein Meister«?

### "Meisterwerk" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - So wie die Lady in dem Meisterwerk "Schoßhündchen der Witwe"?

### "Menge" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eine Menge“, antwortete Kat.

### "Menschenfresser" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Anwohner nennen diesen Fluss „Menschenfresser“ und fürchten ihn.

### "Merkur" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die ersten Raumanzüge der NASA wurden für das Programm „Merkur“ entwickelt.

### "Michael" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Michael“ ist ein männlicher, aber „Michaela“ ein weiblicher Vorname.

### "Michaela" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Michael“ ist ein männlicher, aber „Michaela“ ein weiblicher Vorname.

### "Michail Svetlov" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ist er wirklich von der «Michail Svetlov»?

### "Milch" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Meine Brüste sind voll Milch !

### "Millionen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zwei Millionen“, erwiderte ich.

### "Minuten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir brauchen ein paar Minuten“, sagte ich.

### "Mir" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Verpasst meiner Seite auf Facebook doch bitte ein „Gefällt-Mir“.

### "Miranda" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Miranda !

### "Mistral" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Wind, der häufig in der Provence weht, heißt „Mistral“.

### "Mit" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mit'm Bajonett.

### "Mitte" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jetzt alle in die Sonne, die Braut in die Mitte !

### "Mittel-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Spanisch wird in Mittel- und Südamerika gesprochen.

### "Moment" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Moment“, sagte David.

### "Moms" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Moms "Freund aus der Kirchengemeinde".

### "Mord-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der einzige Mord- und Selbstmordfall in der Geschichte Fairvales.

### "Mordanschlag" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der andere Mordanschlag“, sagte Keith.

### "Morgen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Morgen«, antwortete ich.

### "Moses" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich glaube, der zweite Moses“, erwiderte Maurice.

### "Moskwitsch" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ein «Moskwitsch».

### "Mountain" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Toms Lieblingsfilm ist „Brokeback Mountain“.

### "Muiriel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Passwort lautet „Muiriel“.

### "Museums-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir haben den Museums- und Ausstellungskomplex besucht.

### "Musik" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Liebst Du Musik ?

### "Mutter" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Von Falschheit kann nie Gutes kommen, Mutter“, sagte sie.

### "Mädchen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei uns heißen alle Mädchen "jüngste".

### "Männer-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich konnte nicht ausmachen, ob es ein Männer- oder ein Frauengesicht war.

### "Méjico" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Den Mexikanern gefällt es nicht, wenn man „Méjico“ anstatt „México“ schreibt.

### "México" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Den Mexikanern gefällt es nicht, wenn man „Méjico“ anstatt „México“ schreibt.

### "Mörderabsätzen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Manchmal spricht man bei arg hochhackigen Frauenschuhen von „Mörderabsätzen“.

### "Müller-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich komme aus einer Müller- und Bäckerfamilie.

### "Nachfrage" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dafür gibt es nicht viel Nachfrage“, antwortete er.

### "Nachgeben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kein Nachgeben“, sagte er.

### "Nacht" + ""negativiert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat den Virus praktisch über Nacht "negativiert".

### "Nachwuchskünstler" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er gewann den Preis in der Rubrik „Bester Nachwuchskünstler“.

### "Naso" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gib mir deinen Arm, Naso !

### "Nerven-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Durch eine Lähmung des Nerven- und Atmungssystems tritt der Tod ein.

### "Nichts" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nichts "und".

### "Niemals" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Niemals "Ihr Dildo".

### "Nihon" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Japan wird auf Japanisch auch „Nihon“ genannt.

### "Nimm" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nimm "Bingo Martin" zum Beispiel.

### "Nimm" + "‘" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nimm‘s mir doch weg.

### "Nova Capricornis vier" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Auf Nova Capricornis vier«, verkündete er, »bin ich beinahe ertrunken.«

### "Nur" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nur'n paar Junkies.

### "Nymphe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Garten der Nymphe“ ist ein berühmtes Gemälde von Maria Palette.

### "Oben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Heißt "erstes Oben“, dass er im letzten Jahr verstorben ist?

### "Ohne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ohne "Kanone" geht's nicht!

### "Oliva" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Oliva !

### "Open Table" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom hat sich über „Open Table“ einen Tisch in dem Restaurant reservieren lassen.

### "Operation" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Operation "Kaffee-Sturm".

### "Ordnung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist schon in Ordnung“, sagte Jill.

### "Orient" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Gérard de Nerval schrieb „Voyage en Orient“ (Reise im Orient).

### "Paar" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir fanden ein Paar "Viva Sade!"

### "Papa" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie war das mit dir und Papa ?

### "Park-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir sind an der Ecke Park- und Hauptstraße.

### "Partie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jetzt beginnt gleich die Partie "White Bears" gegen "Kenzan Fighters".

### "Pate" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Pate“ ist mein Lieblingsfilm.

### "Patent" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ihr Original-Patent … wurde gerettet!

### "Patienten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich behandle einen Patienten“, sagte der Arzt.

### "Pechs" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - «Insel des Pechs»!

### "Pennys" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Versuchen wir’s mit einem Stapel Pennys“, sagte ich.

### "Personal" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich frage Yu Arnesons Personal“, fügte Keith hinzu.

### "Pfau" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist ein Pfau !

### "Pferde-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pferde- und Rinderherden streiften frei über die Ebene.

### "Pfirsichbaum" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pfirsichbaum“ heißt auf Altenglisch „persoctrēow“.

### "Pfoten" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Weg mit deinen Pfoten !

### "Pfund" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fast vier Pfund“, sagt Bob.

### "Philosophie" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Philosophie“ ist ein griechisches Wort, das „Liebe zur Weisheit“ bedeutet.

### "Photographie" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine Photographie !

### "Photoshop" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom kann recht gut mit „Photoshop“ umgehen.

### "Phrase" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ok, aber nur, wenn Sie die Phrase "völlig klar" nicht von mir stehlen.

### "Pinterest" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maria hat bei „Pinterest“ ein Bild von ihren Keksen hochgeladen.

### "Piraten" + ""Westley" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Siehst du, niemand würde sich dem Furchtbaren Piraten "Westley" ergeben.

### "Pochi" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mike nennt seinen Hund „Pochi“?

### "Politik" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er liest nur die Rubriken „Wirtschaft“ und „Politik“.

### "Potjomkin" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Panzerkreuzer Potjomkin“ ist Sergei Eisensteins großartigster Film.

### "Preise" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das waren ganz schön hohe Preise“, sagte Jill.

### "Prigogine Works" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie ist von Prigogine Works“, sagte ich.

### "Problemstellungen" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zu eindeutigen Lösungen für eindeutige Problemstellungen - zu kommen.

### "Programm" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Stephan nannte das Programm "Schwangeres Pferd".

### "Projekt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Darum gibt's das Projekt "Hoffnung".

### "Protokoll" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - IP steht für „Internet-Protokoll“.

### "Puls-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Puls- und Atemresonanz normal.

### "Punkt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe noch einen letzten Punkt“, sagte ich.

### "Qualitätsleute" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Qualitätsfisch für Qualitätsleute« lautete ihr Wahlspruch.

### "Quantenphysik" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für Hitler ist Quantenphysik "jüdische Wissenschaft", sagte er zu Einstein.

### "Queen Elizabeth" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich taufe dieses Schiff auf den Namen „Queen Elizabeth“.

### "Quelle" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Menghino, wie weit ist's bis zur Quelle ?

### "Querdenker" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Toms Rufzeichen lautet „Querdenker“.

### "Randy" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist mein Assistant Deputy in Charge Randy "the Snake" Ratford.

### "Rap" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Frauen werden in vielen Liedern des Genres „Rap“ herabgewürdigt.

### "Recnet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Recnet“, sagte Vincent.

### "Reihe" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jetzt alle in einer Reihe !

### "Reis" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist Reis“, antwortete Cooper Naismith.

### "Reise" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und dann schrieb Vater, er könne mir die Reise - nach England nicht bezahlen.

### "Reise" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Meine Glückwünsche zu Ihrer erfolgreichen Reise“, sagte ich.

### "Relnovo" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ach, dieses Relnovo“, sagte William.

### "Renaissance" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was bedeutet „Renaissance“?

### "Restaurant" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Waren Sie gestern im Restaurant - "Shu Yu"?

### "Richtigen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich kenne genau den Richtigen“, antwortete er.

### "Richtlinien" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lies bitte unsere „Regeln und Richtlinien“.

### "Ring" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich dachte, der Ring - ist vielleicht fertig.

### "Roboter" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wildtiere sind keine Roboter“, sagt sie.

### "Roggen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir haben zufälligerweise zwei Exemplare des „Fängers im Roggen“.

### "Rory" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rory“, sagte ich, „das sollte für dein Abendessen reichen.“

### "Rosetta" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rosetta“ wurde von der Europäischen Weltraumorganisation gebaut.

### "Rossija" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Russland heißt auf Russisch „Rossija“.

### "Rovers" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bist du für „City“ oder die „Rovers“?

### "Runde" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum trainiert ihr beide nicht einfach mal eine Runde "Fresse halten"?

### "Runden" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich glaube, wir kommen gerade so über die Runden«, sagte ich.

### "Sache" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Noch eine letzte Sache“, sagte Sparhawk.

### "Satellit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Welche verschiedenen Bedeutungen können Sie für das Wort „Satellit“ finden?

### "Satz" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie hasst den Satz "Gebenedeit ist die Frucht deines Leibes".

### "Satzes" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Einen schönen „Tag des Satzes“!

### "Sauge" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sauge !

### "Saugen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Saugen !

### "Schafe" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Ärsche der Schafe ?

### "Schallplatten" + "unterhalter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wer ist dein Lieblings-Schallplattenunterhalter?

### "Schande" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine Schande !

### "Schaufenster" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir gingen weiter und hielten beim nächsten Schaufenster … einer Schneiderei.

### "Schaut" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schaut !

### "Scheiß" + "aktion" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was für eine Scheißaktion.

### "Scheiß" + "tag" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was für ein Scheißtag!

### "Scheiße" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fresst Scheiße - Millionen von Fliegen können nicht irren.

### "Schießpulver" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der häufigste davon ist „Schießpulver“.

### "Schiffe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich suche Bücher über Schiffe“, sagte ich.

### "Schiffskapitänin" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin keine Schiffskapitänin“, sagte ich.

### "Schiffsmeisterin" + "‹" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - ›Schiffsmeisterin‹?

### "Schild" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hängte ein Schild "Geschlossen" an die Eingangstür des Geschäftes.

### "Schlaf-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Haus verfügt über ein Ess-, ein Schlaf- und ein Arbeitszimmer.

### "Schlangenbiss" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber bei einem Schlangenbiss … geht's schnell.

### "Schließlich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schließlich "wählt eine Seele ihre eigene Gesellschaft."

### "Schloss" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Schlüssel steckt im Schloss“, fügte er hinzu.

### "Schlägereien" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bei dir gab es keine Schlägereien“, fuhr Kevin fort.

### "Schlüssel" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gib mir den Schlüssel !

### "Schneide-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Eckzähne sind zwischen den Schneide- und den Backenzähnen.

### "Schneller" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Schneller«, sagte ich zwischen zwei Bissen Obst.

### "Schnäppchenfest" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darum ist es ja ein Schnäppchenfest“, antwortete er.

### "Schusswaffenvereinigung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom ist ein Mitglied der „Nationalen Schusswaffenvereinigung“.

### "Schwein" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe Tom als „fettes Schwein“ tituliert.

### "Schwuchtel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sagt weiter „Schwuchtel“, ja?

### "Schwul" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schwul“?

### "Science" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Studie wurde im Journal „Science“ veröffentlicht.

### "Segel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dieses Boot hat Segel“, sagte Nicholas.

### "Selbst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Gefühl von „Selbst“ ist identisch mit dem, was wir Bewusstsein nennen.

### "Selbsterhaltung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kannst du mit dem Begriff „Selbsterhaltung“ etwas anfangen?

### "Senat" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Begriffe „Senat“ und „Zoll“ waren mir beide neu.

### "Sendung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Sendung "Große Denker der Welt, Albert Einstein" ist zu Ende.

### "Serie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du schaffst für mich eine neue Serie "Chefkoch Skinners Tiefkühlkost".

### "Shakespeare" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er wusste nicht, dass Shakespeare "Romeo und Julia" geschrieben hat.

### "Sheila" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fräulein Sheila“, sagte ich zur Signalgeberin.

### "Sicherheits" + "unterweisung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nichts ist beruhigender als eine gute, alte Sicherheitsunterweisung.

### "Sie" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nur, wenn Sie'n Heldentod sterben wollen.

### "Sing" + "e" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Singe bitte ein Lied in deiner Sprache!

### "Sinn" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Du hast schon Artikel im Sinn«, sagte Terry.

### "Sir" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es wirkt etwas feudalistisch, ihn „Sir“ zu nennen.

### "Situation" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Boldi befand sich in einer „hopfenlosen Situation“.

### "Skorpions" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - In der arabischen Sprache wird Antares „Herz des Skorpions“ genannt.

### "Skype" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom und Maria sprechen häufig über „Skype“ miteinander.

### "Snobs" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das sind doch alles Snobs "aus gutem Hause".

### "Soccer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Soccer“ nennen die Europäer „football“.

### "Sofort" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sofort“, stimmte ich zu.

### "Sohn" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dein Halunke von einem Sohn !

### "Sohn" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich nenne meinen Sohn "Ungedeckt".

### "Solaristik" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es geht um etwas wichtigeres als die Solaristik - um unser Erkenntnisvermögen.

### "Sondern" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sondern … von einer Schlange.

### "Sonnen-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir benutzen sie oft als Sonnen- oder Regenschirme.

### "Sonnen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sonnen- und Regenmädchen entstehen dank Gaias Homöostase.

### "Sonnenauf-" + "bis" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es dauerte von Sonnenauf- bis Sonnenuntergang.

### "Sonntag" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Domingo“ ist das spanische Wort für „Sonntag“.

### "Sowjetrepubliken" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „UdSSR“ steht für „Union der Sozialistischen Sowjetrepubliken“.

### "Spam" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das einzige, was ich einen Hund je verschmähen sah, ist „Spam“.

### "Spiegel" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Einen Spiegel !

### "Sprache" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Internationale Sprache“ ist sehr nützlich.

### "Spukhaus" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sami nannte das Video „Das Spukhaus“.

### "Stadt" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die ewige Stadt - das ist Rom; und die immerwährende Baustelle - das ist Berlin.

### "Stadt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es geht um die Stadt“, sagte der Hafenmeister.

### "Stadtzentrum" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir wohnen nahe dem Stadtzentrum .

### "Staffel" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Staffel - überfliegt Deutschland in einer Nachtmission.

### "Staffeln" + ""Bounty Law"" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir arbeiten seit den letzten zwei Staffeln "Bounty Law" zusammen.

### "Stanley" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Stanley "Das Werkzeug" Wilson.

### "Statt" + ""Dora" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Statt "Dora" und all diesen Quatsch sehen wir uns dieses Video an.

### "Steht" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Steht "gewandt" im Wörterbuch?

### "Sterne" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vielleicht trägt sie den Makel der Sterne“, sagte Lanceson düster.

### "Sternenschiffer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „Astronaut“ kommt aus dem Lateinischen und bedeutet „Sternenschiffer“.

### "Sternschnuppen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Leute sprechen bei Meteoren oft von „Sternschnuppen“.

### "Stiftung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In die Stiftung "Padre Jesuíno Vidal" in Pelotas, in Rio Grande do Sul.

### "Stoff" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieser Stoff - war Ihrer Frau zu auffällig.

### "Stoff-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Italiener sind Stoff- und Lebensmittelhändler.

### "Stoss" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schwieriger Stoss !

### "Stromschnellen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Indianer nennen die Stromschnellen "Chirimagua", die zornigen Geister.

### "Studiengang" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum gibt es keinen Studiengang "Dokumentenfälschung"?

### "Stunde" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist schon wieder „fröhliche Stunde“.

### "Städte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Marsch in die Städte“ dauerte viel länger, als ich gedacht hätte.

### "Stärke" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stärke“ bezieht sich nicht nur auf unseren Körper.

### "Suomi" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Finnland heißt auf Finnisch „Suomi“.

### "Suppe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „So klar wie diese herrliche Kartoffel-Lauch-Suppe“, sagte er.

### "Svetlov" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - «Michail Svetlov»?

### "Südafrika" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf Afrikaans heißt Südafrika "Suid-Afrika".

### "Tag" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vielleicht kommt ja immer noch irgendwann der Tag“, erwiderte er.

### "Takte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fünf Takte "Donna Lee".

### "Tannenbaum" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „O Tannenbaum“ ist ein traditionelles deutsches Volkslied.

### "Tanz-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Heute besitze ich diese Tanz- und Fechtschule, die rosa Rumba Ranch.

### "Teil" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Jetzt kommt der geheime Teil«, sagte ich.

### "Teo" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Komm runter, Teo !

### "Teufel" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schert euch zum Teufel !

### "The Slits" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ari Up (Ariane Forster) war die Frontfrau von „The Slits“.

### "Times" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich lese die „Times“.

### "Tinder" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Viele sind bei „Tinder“ nur auf eine kurze Beziehung aus.

### "Tippen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich wusste, dass man das „Tippen“ nannte, hatte ihn aber noch nie dabei gesehen.

### "Titta" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schau, Titta !

### "Tja" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tja … hat er nicht.

### "Tod" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stelldichein mit dem Tod“ ist ein Krimi von Agatha Christie.

### "Toilette" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - An der Tür war ein großes T für „Toilette“.

### "Tom" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum hat Tom "Ja" gesagt?

### "Torfleet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin der Hafenmeister von Torfleet“, antwortete er.

### "Trag" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Trag's mit Fassung.

### "Trapez-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber am berühmtesten war sie als Trapez- und Seilkünstlerin.

### "Trauerweide" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Schicken Sie ein Taxi zum Restaurant «Trauerweide».

### "Trauma-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er bekommt Trauma- und Kunsttherapie.

### "Treue" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Im Herzen eines Agenten ist nur Platz für die Treue - zur Partei, seinem Land.

### "Trikolore" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Trikolore“ ist nicht die einzige Trikolorenflagge auf der Welt.

### "Trilogie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Star der Rache-Trilogie "Die Retourkutsche".

### "Trumpete" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Clinton kann Saxophon spielen und Trump die „Trumpete“.

### "Tschto" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tschto "Standby"?

### "Tschüß" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich bin nur gekommen, um »Tschüß« zu sagen.

### "Tsubasa" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Tsubasa“ ist ein sehr schneller Zug.

### "Tu" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu's nicht, mein Sohn.

### "Twitter" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Man kann dem Papst jetzt auf „Twitter“ folgen.

### "Typische" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Typische "Vier alte Übel".

### "Tütchen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir brauchen Tütchen“, sagte Kat.

### "Tüte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie eine Tüte "Skittles".

### "U-Bahn" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In Großbritannien nennt man die U-Bahn "underground" und nicht "subway".

### "USA" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum sagt man in den USA "fall" für den Herbst?

### "UdSSR" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „UdSSR“ steht für „Union der Sozialistischen Sowjetrepubliken“.

### "Um" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Um "ja" oder "nein" zu sagen?

### "Umarm" + "e" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Umarme mich fest.

### "UniComp" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe keinen UniComp“, antwortete mein ältester Gast.

### "Universalwarenhaus" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie wollte ihn „Ehrliche Amandas Universalwarenhaus“ nennen.

### "Unternehmensengel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maria betätigt sich als „Unternehmensengel“.

### "Unterricht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich gebe auch gerne Unterricht“, sagte Gilbert.

### "Ursprung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und zwar, weil dein Ursprung "Nutzlosigkeit” ist.

### "Vaterland" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf die Familie, auf die Kirche, und aufs Vaterland !

### "Veranstaltungs" + "zentrum" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das ist ein Kultur- und Veranstaltungszentrum in Altona.

### "Verb" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Verb "sein" wird in allen Sprachen unregelmäßig konjugiert.

### "Verbrechen" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie hat wundervolle Ideen für Verbrechen … und für Mode.

### "Verbrecher" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es sind sehr gefährliche Verbrecher“, betonte Saunders.

### "Verdammt" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Verdammt - mein Auto ist geklaut!

### "Vergnügen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erst das Geschäft, dann das Vergnügen“, erwiderte ich.

### "Vermissten-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Geben Sie im Büro eine Vermissten- und eine Diebstahlsanzeige auf.

### "Verstanden" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Verstanden«, sagte Terry.

### "Verstanden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verstanden“, antwortete ich.

### "Verstecken" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir haben nur „Verstecken“ gespielt.

### "Verstehe" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Verstehe«, sagte ich, völlig verblüfft.

### "Versuch" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Versuch "letzte Anrufe".

### "Verwandte" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer ist der Verwandte ?

### "Vielfältigkeiten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das sind zu viele „Vielfältigkeiten“, Chris.

### "Vielleicht" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht "Daijin"?

### "Volleyball" + "trainer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich bin der neue Volleyballtrainer, Chou Ming.

### "Vom" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vom "Grafen von Monte Christo"?

### "Vor" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vor "wie viel"?

### "Vor" + "an" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Voran.

### "Vor" + "rücken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vorrücken.

### "Vor-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich weiß nicht, ob Vor- oder Nachmittag ist.

### "Voraussicht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das nenne ich „weise Voraussicht“.

### "Vorder-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alles hat eine Vorder- und eine Rückseite.

### "Vorher" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vorher“?!

### "Vögel-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie ist ein Vögel- und Blumenfreund.

### "WILDPILZE" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - FAMILIENTRAGÖDIE DURCH WILDPILZE "Ermordet durch Wildpilze"?

### "Wache" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kat hält Wache“, sagte er.

### "Waffen-" + "lieferungen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es geht um amerikanische Waffen- lieferungen an Chiang Kai-shek.

### "Waffen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Erhältlich in jedem gut sortierten Waffen- und Souvenirgeschäft.

### "Wagen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wieso im Wagen ?

### "Wagen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Wagen- und die Sitznummer stehen auf der Fahrkarte.

### "Wahlkampfspenden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was in aller Welt – nein, nicht auf unserer Welt – waren „Wahlkampfspenden“?

### "Waisenkinder" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wo sind die Waisenkinder ?

### "Wart" + "‘s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wart‘s ab.

### "Wartet" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wartet !

### "Wartungs-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei Wartungs- und Reinigungsmaßnahmen sind Pausen vorgeschrieben.

### "Warum" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum "mein"?

### "Was" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was“, fragte er, „sollte man mit jemandem machen, der alle hintergangen hat?“

### "Was" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was … Nein!

### "Watson" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arbeit hilft am besten gegen Vergrämtheit, mein lieber Watson“, sagte er.

### "Weile" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir brauchen vielleicht eine Weile“, sagte ich.

### "Weisheit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Philosophie“ ist ein griechisches Wort, das „Liebe zur Weisheit“ bedeutet.

### "Weit-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Brillen für Weit- und Kurzsichtige".

### "Weiterfeuern" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weiterfeuern“, befahl ich den Schiffsarmbrustmannschaften.

### "Welches" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Welches "E" erfüllt dich?

### "Welt" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nichts hält ewig in dieser verhexten Welt - nicht mal unsere Sorgen.

### "Wenigen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Oligarchie bedeutet „Herrschaft der Wenigen“.

### "Wenn" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn "G" für Glück steht, wieso schaust du dann so gebrochen?

### "Wer" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maschinen sind kein »Wer«, und denken können nur Menschen.

### "Wettbewerb" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle weinten wie beim Wettbewerb "Wer weint am lautesten".

### "Wetterwache" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verdoppeln Sie die Wetterwache“, befahl ich.

### "Whisky-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In einem Whisky- oder Longdrink-Glas?

### "Wiedersehen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auf Wiedersehen !

### "Wieso" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wieso ?

### "Wikipedia" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Geben Sie mal bei Wikipedia "Vollpfosten" ein.

### "Wir" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir“, sagte der Zaunkönig.

### "Wirtschaft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er liest nur die Rubriken „Wirtschaft“ und „Politik“.

### "Wort" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Beseda“ bedeutet auf Slowenisch „Wort“.

### "Wörtchen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nur das Wörtchen "Ja", und ich kann Euch die Fesseln abnehmen, Euch befreien.

### "Zahlen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schöne runde Zahlen“, erwiderte er.

### "Zar" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es hieß der «Zar».

### "Zarewitsch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bring mich nicht um, lieber Zarewitsch“, sprach der Bär.

### "Zeit" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Um diese Zeit !

### "Zeit-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist Zeit- und Geldverschwendung.

### "Zentner" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mindestens einen halben Zentner“, antwortete er.

### "Zieh" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zieh's trotzdem mal an.

### "Zigarren" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Setzen Sie sich und probieren Sie eine dieser Zigarren“, sagte er.

### "Zimmer" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Da ist noch ein Zimmer«, sagte der Kerl.

### "Zimmer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die gnädige Frau weilt auf ihrem Zimmer“, sagte der Diener.

### "Zitronen-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zitronen- und Orangenbäume wuchsen im Garten.

### "Zoll" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Begriffe „Senat“ und „Zoll“ waren mir beide neu.

### "Zorin" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zorin“, sagte ich, „die Rechnung und ein Trinkgeld für Margrethe.

### "Zug" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nimm einen Zug "Alter Tobi".

### "Zum" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zum "Terceiro Comando".

### "Zwei" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zwei "Golden Menus"!

### "Zwergnonne" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine Zwergnonne !

### "Zwilling" + "s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zwillingspaar.

### "Zwillinge" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zwillinge“, sagte Clorinda.

### "Zwölfer-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die bekommt man in Sechser-, Zwölfer- oder Achtzehnerpackungen.

### "ab" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nimm den Hut ab !

### "ab" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht wartet er nur ab - bis wir wieder auftauchen.

### "ab" + "lenken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lass dich nicht von einem Jungen vom Lernen ablenken!

### "ab" + "zuschließen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich hatte nicht viel Zeit, um meine Hausaufgaben abzuschließen.

### "abbringen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir wollen dich nicht von deinem Weg abbringen“, sagte ich.

### "aber" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das sind echt gute Lines, aber "Ladies First".

### "abkürzen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich will euch die Sache abkürzen“, verkündete Keith den vieren.

### "abschicken" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe vergessen, auf „abschicken“ zu klicken.

### "abwimmeln" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich will Sie nicht abwimmeln“, sagte er.

### "abzu" + "holen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es wird Zeit, Thomas von der Schule abzuholen.

### "abzu" + "lösen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Indien ist im Begriff, China als bevölkerungsreichstes Land der Erde abzulösen.

### "acht" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Es waren sogar mehr als acht«, sagte ich.

### "achten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du hast deine Älteren zu achten“, erwiderte sie.

### "alle" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich liebe euch alle !

### "alle" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Glocke ruft uns alle“, erwiderte er.

### "allen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Da drinnen wird sie doch von allen "herzensgute Frau Geum-Ja" genannt.

### "alles" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Für Feuerfische nehme ich alles“, sagte er.

### "alte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du verlässt also das gute alte "Wir belehren dich"?

### "an" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Klaviatur fängt an - und endet.

### "an" + "zubieten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Marktüblich wäre, im neuen Gebäude eine neue Wohnung anzubieten.

### "an-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom stand an der Tür, den Lichtschalter an- und ausschaltend.

### "andauernd" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sag nicht andauernd "Leiche".

### "angeblich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bedeutet angeblich "Bullen".

### "angebrannt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mama ist heute das Essen angebrannt“, erzählte Fritzchen seiner Oma.

### "angehen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich glaube, wir sollten die Sache ein wenig behutsamer angehen«, sagte er.

### "annehmen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir würden Reis im Tausch annehmen“, sagte Kat.

### "anschaut" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sehen Sie, wie er mich anschaut ?

### "ansehen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Und wir sollten uns ein paar Karten ansehen“, sagte Kat.

### "arbeiten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wer essen will, muss arbeiten“, verkündete ich.

### "auf" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Im Esperanto endet ein abgeleitetes Adverb auf -e.

### "auf" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und so passt du also auf ihn auf ?

### "auf" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pass auf dich auf“, sagte er.

### "auf-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich hörte eine Autotür auf- und zugehen.

### "aufbrechen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber wir müssen jetzt aufbrechen“, sagte er.

### "aufgehört" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es hat aufgehört !

### "aufwachen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Gegenteil von „aufwachen“ ist „einschlafen“.

### "aufzunehmen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sind wir bereit, die Migranten aufzunehmen ?

### "aufzunehmen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bereitmachen, Fahrt voraus aufzunehmen“, rief ich.

### "aufzurichten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie dürfen damit anfangen, es wieder aufzurichten“, erwiderte ich.

### "aus" + ""Peyton Place" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die aus "Peyton Place".

### "ausdrücklich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In der Anleitung schreiben sie ausdrücklich "Schneiden".

### "ausgemacht" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer hat das Licht ausgemacht !

### "ausreichend" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er bekam ein „ausreichend“.

### "auszahlt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich verstehe, wenn man problematische Kleinanleger auszahlt“, sagte ich.

### "bald" + ""abkratzen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn er nicht bald isst, wird er bald "abkratzen".

### "baut" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Neue Erkenntnisse wie „wie man ein Raumschiff baut“?

### "bedeutete" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Früher bedeutete "Asche auf mein Haupt", man war im Urlaub in Auschwitz.

### "beeindruckend" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist sehr beeindruckend“, sagte ich.

### "befragt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe die Wache befragt“, sagte Henrison.

### "begegnet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ja, wir sind uns schon einmal begegnet“, sagte Mr. Jordan.

### "begeisterte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Diesen Typen begeisterte "snuff".

### "begleichen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fang damit an, alle deine größeren Barsalden zu begleichen“, schlug ich vor.

### "beiden" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir nennen die beiden "Kummer" und "Mitleid".

### "bekloppt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist bekloppt“, sagte Tom.

### "bekommen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du wirst meine Tasche an deine Löffel bekommen !

### "bekommen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mit etwas Glück werdet ihr bald Fisch bekommen“, antwortete ich.

### "belustigend" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist äußerst belustigend“, befand der Teufel.

### "beobachten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Man muss sie beobachten“, sagte der Grafschaftsbibliothekar.

### "beraten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich muss mich mit meinen Töchtern beraten“, sagte Bombastico.

### "bereit" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heckspille betakelt, bemannt und bereit“, kam die Meldung von unten.

### "berührst" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Weisst du, dass der Heilige Ludwig weinen muss, wenn du dich berührst ?

### "beschäftigt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin gerade wahnsinnig beschäftigt“, antwortete sie.

### "besprechen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Und danach können wir Ihre Anliegen besprechen“, fügte ich hinzu.

### "besser" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du sagst also besser "hier", Carson.

### "besser" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Noch besser“, sagte er.

### "besseres" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du kein besseres "Heil mir" als das?

### "beugen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Steht auf" ihr müsst das Knie nicht beugen "vor dem Thron der Gerechtigkeit."

### "beurteilen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Man kann Leute nicht nach ihrem Aussehen beurteilen“ ist gut gesagt, oder?

### "bevorzugen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir bevorzugen "Aquarius".

### "bezahlen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich muss unsere Rechnung bezahlen“, sagte ich und stand auf.

### "bezahlt" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Sie wurden im Voraus bezahlt«, unterbrach ich ihn.

### "billig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ja, aber diese Leute verkaufen sie billig“, sagte Terry.

### "billiger" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aus reinem Kupfer gewebt wäre sie billiger“, stöhnte Yu.

### "bin" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich bin "es".

### "bin" + "‘s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tom, bist du‘s?“ – „Nein, ich bin‘s nicht.“

### "bist" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Komm und sag mir, wer du bist“, sprach jene, „und wie du hergekommen bist.“

### "bitte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zum Abschluss sagen Sie bitte "Weiter".

### "bitte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sagen Sie „bitte“ .

### "bittest" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich verstehe, worum du bittest“, sagte Arthur.

### "blasen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nicht blasen !

### "bleiben" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und ich muss zu Hause bleiben !

### "bleiben" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du kannst gehen oder bleiben - deine Wahl.

### "blicken" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ich sehe Sie oft aus Ihrem Fenster blicken "und bei der Messe am Sonntag.

### "bloß" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Doktor Willis nennt ihn bloß "er".

### "brauchst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich wusste, dass du eine Eskorte zu David brauchst“, erklärte er.

### "braun" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Motto für den Antiextremismus-Tag war „bunt statt braun“ .

### "chat" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „chat“ heißt auf französisch „Katze“.

### "da" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Steht da "Monsieur"?

### "da" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom war immer da .

### "damit" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hör auf damit !

### "damit" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du nicht heiss damit ?

### "danach" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sprechen Sie direkt zum Anrufer und sagen Sie danach "Weiter".

### "dankbar" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dafür sind wir zutiefst dankbar“, sagte Maurice.

### "darum" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich kümmere mich darum“, sagte er.

### "das" + ""Kleine Paris im Osten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Habt ihr die Fernsehwerbung für das "Kleine Paris im Osten" gesehen?

### "dass" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er wusste nicht, dass "Romeo und Julia" von Shakespeare geschrieben wurde.

### "davon" + "gemacht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kurzum hat er sich davongemacht, ohne seine Schulden zu begleichen.

### "davon" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Südlich des Äquators gibt es eine Menge davon«, sagte er.

### "davon" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diesmal kommst du nicht davon“, verkündete er.

### "daß" + "Tom" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich bemerkte, daßTom Marias Schlappen trug.

### "deine" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das Bezahlen ist Sache der Familie, nicht deine“, antwortete sie.

### "deiner" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wart auf mich da unten in deiner "Metro".

### "deliziös" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Seine Zehen riechen gar deliziös“, dachte der Hund.

### "den" + ""Antillais" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was den "Antillais" angeht, der ist nur ein gewöhnlicher Zuhälter.

### "denen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir sagen denen "Guten Tag".

### "denkt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn KI „denkt“ oder sich dem Denken annähert, wer sind wir dann?

### "denn" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wo denn !

### "der" + ""Koganeimaru" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er ist Kapitän auf der "Koganeimaru".

### "der" + ""Pepuplik" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das ist der Präsident der "Pepuplik".

### "der" + "Arbeit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er erinnert mich einen Buchhalter bei derArbeit.

### "deshalb" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und deshalb … bist du jetzt Schiffsherrin Amanda Kirasdotr.

### "deutliches" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn ich euren Namen aufrufe, will ich ein deutliches "hier" hören!

### "dich" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Setz dich !

### "dichten" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn du betrunken bist, kannst du sogar dichten !

### "die" + ""Adolpho" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das stimmt, mein letztes Schiff war die "Adolpho".

### "die" + ""Koganeimaru" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nein, das ist die "Koganeimaru"!

### "die" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Besetzen Sie die …“ Sein Trupp rannte bereits zur Heck-Saugpumpe.

### "diesem" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hör auf mit diesem "Sie", sag einfach "Du".

### "dirigiert" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er dirigiert "Künstlerleben", einen Walzer von Johann Strauß.

### "doch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber die armen Kinder dauern mich doch“ sagte der Mann.

### "doch" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber es gibt doch … Gesetze.

### "dort" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Brennholz kommt von dort“, sagte sie.

### "drauf" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Als Absender steht drauf "Daniel Garrido".

### "drei" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Nummer drei«, sagte ich.

### "drei" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nun sage ich «drei» und deine Hände erstarren.

### "drei" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Eins, zwei, drei …« Sie rammten meine Schlafzimmertür.

### "drei-" + "bis" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fleisch esse ich nur drei- bis viermal im Monat.

### "dreifachen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dafür bedanken wir uns mit einem dreifachen "Prosit!"

### "dreizehn" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „dreizehn“ oder „dreißig“ gesagt?

### "dreißig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „dreizehn“ oder „dreißig“ gesagt?

### "drin" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hier ist alles drin“, sagte Tom, auf seinen Kopf deutend.

### "drücke" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich drücke "Play".

### "du" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich stelle hier die Fragen, nicht du !

### "du" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Pronomen «du» bekam nun andere Bedeutung.

### "du" + "‘s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tom, bist du‘s?“ – „Nein, ich bin‘s nicht.“

### "duh" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie „Buh-duh, buh-duh, buh-duh“.

### "durch" + "gedrungen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe die Zeit gemessen und die wie ein Gebirge durchgedrungen.

### "durchsuchen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf meine Idee, „sie alle zu durchsuchen“, war er unabhängig gekommen.

### "eBay" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Denken Sie ernsthaft darüber nach, das auf „eBay“ zu verkaufen?

### "echten" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir sind keine echten "Biker", wir machen nur ne Spritztour.

### "ehrgeizig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist sehr ehrgeizig“, sagte Jim.

### "ehrlich" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom lebt nach der Devise „Nur der Dumme ist ehrlich“.

### "ein" + "atmen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Als würde es einatmen.

### "ein" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich nehme an, es war ein … langsamer Tod.

### "eine" + ":" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alle wollen nur das eine: Frieden.

### "eine" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wie macht so eine ?

### "einem" + ""I" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber im lateinischen Alphabet beginnt Jehova mit einem "I".

### "einfach" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Leben unter der Erde ist nicht einfach .

### "einfach" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich war einfach … hier.

### "eingekauft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe falsch herum eingekauft“, sagte Kat.

### "eingeschlossen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ihr werdet in vier Räumen eingeschlossen“, sagte ich.

### "einige" + ""befragt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Von denen hatte man schon einige "befragt".

### "einschlafen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Gegenteil von „aufwachen“ ist „einschlafen“.

### "endlich" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gib endlich … auf.

### "entdecken" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Verwende nicht „entdecken“, wenn du eigentlich „erfinden“ meinst.

### "entgangen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »David Robertson ist gerade knapp einem Mordanschlag entgangen«, sagte er.

### "entschuldigen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich muss mich nochmals entschuldigen“, sagte ich.

### "entwickelt" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es wurde ein neues Kommunikationsmittel entwickelt - die Eisenbahn.

### "entwirren" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich sehe schon, da gibt es eine Menge Knoten zu entwirren“, brachte ich heraus.

### "er" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dort ist er !

### "erklingt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es erklingt "Chi Mi Frena In Tal" von Enrico Caruso & Marcella.

### "erledigt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schon erledigt“, verkündete sie.

### "erreichen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben versucht, euren Commnet aus dem Orbit zu erreichen“, sagte Arthur.

### "erst" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was kam zu erst - das Ei oder das Huhn?

### "erträglich" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Folgende anzuhören ist kaum erträglich - es zu sagen ist noch schwerer.

### "erzielen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich werde für ihren Fang den bestmöglichen Preis erzielen“, sagte ich.

### "es" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Onkel, schau, ich hypnotisiere es !

### "es" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Herr Anwalt, wie schwer wiegt es ?

### "es" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Tim ist es«, drang die Stimme durch die Tür.

### "etwas" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Suchst du etwas ?

### "etwas" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Da ist noch etwas“, sagte sie.

### "etwas" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - So etwas …?

### "euch" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ab ins Bett mit euch !

### "fangen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sollen wir ihn fangen ?

### "fantastisch" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber du machst das fantastisch“, sagte ich.

### "fehlen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Deine Zimtschnecken werden mir fehlen«, sagte ich.

### "festgesetzt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Leinen liegen und sind festgesetzt“, meldete er schließlich.

### "finden" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Leben, einen Weg durch die Verunreinigung finden .

### "findet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Karte war mit „Von einem Mann, der dich attraktiv findet“ unterschrieben.

### "fische" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich fische«, verkündete ich.

### "football" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Soccer“ nennen die Europäer „football“.

### "fort" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gradisca hat geheiratet und geht nun weit, weit fort !

### "fragen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Erst musst du fragen "Wer?"

### "fragen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Muss ich Opa fragen«, sagte sie.

### "fragte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Also ich fragte "Wie heisst du"?

### "freundlich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich wünsche euch freundlich "Guten Morgen!", und ihr mir auch.

### "freundlich" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie sind sehr freundlich“, antwortete er.

### "freundliches" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Noch ein freundliches "Fickt euch"?

### "früh" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich komme wegen meiner Schwester, bin ich zu früh ?

### "früh" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Morgen früh“, sagte ich.

### "fucking" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kein fucking "aber"!

### "funktioniert" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es hat funktioniert“, sagte Vincent.

### "fängt" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Dann versuchen Sie mal, jemanden zu finden, der Weißfische fängt«, sagte sie.

### "für" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was »Nau Kaperkorn ist für« bedeuten soll, erschließt sich mir überhaupt nicht.

### "ganz" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nicht ganz“, antwortete ich.

### "geben" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Da muss es noch einen Haken geben«, erwiderte Khan.

### "geben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es spricht nichts dagegen, ihr weiter Schmerzmittel zu geben“, verkündete er.

### "gebrüllt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Haben wir jemand die Knarre an die Schläfe gehalten und gebrüllt "Kohle her"?

### "gefallen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er ist in Ohnmacht gefallen !

### "gefallen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lass dir meine Seite auf Facebook doch bitte „gefallen“!

### "gefangen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe nicht viele Feuerfische gefangen“, fügte ich ein.

### "gefunden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben den Fehler gefunden“, sagte er.

### "gefällt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du sagtest, dir gefällt "Fever Dog".

### "gegeben" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wann hatte er dir den ersten Kuss gegeben ?

### "gegeben" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Aber wir haben ihnen doch das ganze Geld gegeben …« Meine Stimme verebbte.

### "gegen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jetzt beginnt gleich die Partie "White Bears" gegen "Kenzan Fighters".

### "gehe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich gehe "Das Jüngste Gericht" malen.

### "geheiratet" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie habt ihr euch getroffen, euch verliebt und dann geheiratet ?

### "geheiratet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben am selben Tag geheiratet“, sagte sie.

### "gehen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Okay, wir gehen "all in".

### "gehen" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alle gehen - vorne durch.

### "gehn" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wollen wir im Hafen fischen gehn ?

### "geht" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gradisca geht !

### "geht" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was geht'n jetzt?

### "gehörst" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Erstens dazu, dass du jetzt Rower’s Folly gehörst«, sagte er.

### "gehört" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Begehrst du, was andern gehört ?

### "geladen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Meistens stand in den Worten darin »nicht geladen«.

### "geliebt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Er hat mich mal geliebt“, antwortete sie traurig.

### "gelöst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die haben Sie gerade gelöst“, antwortete er.

### "genannten" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir beginnen mit dem so genannten "Collatz-Problem".

### "genau" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ganz genau«, donnerte Chris.

### "gerechnet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Damit hatten wir gerechnet“, sagte ich.

### "gerettet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe die beiden aus den Fängen der Horrorhexe gerettet“, verkündete Piotr.

### "gern" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er wird nicht gern "Herr" genannt, das regt ihn auf.

### "gesagt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie haben alle nette Dinge über die Rosinen gesagt“, erwiderte sie.

### "gesagt" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Haben Sie gesagt … Tage?

### "gesattelt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wird bereits gesattelt“, sagte Victorson.

### "geschossen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Auf sie wurde nicht geschossen“, antwortete Oberkommandant Parsons.

### "gesegelt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir sind gesegelt“, sagte Kat.

### "gespannt" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Die Armbrüste sind nicht gespannt«, antwortete ich.

### "gesprochen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wohl gesprochen“, brachte er hervor.

### "gesprungen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Fische sind geradezu ins Netz gesprungen“, berichtete sie.

### "gestattet" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Das ist nicht gestattet …“, begann er.

### "gesucht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Danach habe ich gesucht“, erklärte er.

### "getroffen" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe sie zufällig getroffen - ausgerechnet in der Mongolei.

### "getötet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Keith hat zwei getötet“, sagte David.

### "geweint" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du hast geweint“, bemerkte Josie.

### "gezogen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich habe ihn aus dem Wasser gezogen«, sagte ich.

### "gibt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt "beaucoup" oder "meilleur".

### "gibt" + "‘" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Im zweiten Stock gibt‘s noch Arbeit.

### "gibt" + "‘s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was gibt‘s zum Frühstück?

### "gibt's" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fräulein, gibt's "Flügel" von Nautilus?

### "glatt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Brüche in deinen Beinen sind glatt“, verkündete Agatha.

### "gleich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich gebe dir gleich "echt"!

### "gleich" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Jedenfalls alle gleich«, antwortete ich.

### "grad" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er rief grad "auf Turm und Zinnen", das war der Ruf der französischen Ritter.

### "große" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das große "K".

### "großen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Suche ein Schild mit einer großen "Eins" darauf.

### "größten" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alles, von der kleinsten Cessna bis zum größten "weiß der Geier"- Jet.

### "guckst" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was guckst'n so?

### "gut" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Lamm und Kohl klingt gut«, sagte ich.

### "haben" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wir haben "Tanner" genossen.

### "haben" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir haben - wortwörtlich - einen Haufen zu Essen.

### "haben" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Gut, eine vorsichtige Schiffsführerin zu haben«, sagte sie.

### "haber" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie lautet die Befehlsform von „haber“?

### "habt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich freue mich, dass Ihr das getan habt“, sagte die gute Königin.

### "haizivs" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hai“ heißt auf Lettisch „haizivs“.

### "halb" + "volle" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Onkel hat mir eine halbvolle Flasche geschenkt.

### "halt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann halt "Guten Abend".

### "hast" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was hast'n du für'n Traum?

### "hat" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Krieg bestimmt nicht, wer Recht hat - sondern nur, wer am Leben bleibt.

### "hat" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat … Geburtstag?

### "hatte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du warst die beste Partnerin, die ich je hatte“, warf William Williamson ein.

### "hebt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Aber ich denke, dieser "Thibault" hebt "Capo Ferro" auf, du nicht?

### "heim" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Naso, ich gehe heim !

### "heiss" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist heiss !

### "heiße" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er stellte mich als "John" vor, aber ich heiße "Jack".

### "helfen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nach „lieben“ ist „helfen“ das schönste Zeitwort der Welt.

### "hellwach" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Im Ernst, wenn du bei mir Dienst hast, bist du immer hellwach“, sagte ich.

### "her" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bringt ihn her !

### "herein" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Amanda, komm herein“, sagte er.

### "herein-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie würden uns einen Tip geben, wenn etwas herein- oder herausging.

### "herum" + "zu" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist gefährlich, mit einer Klinge so herumzuwedeln.

### "herum" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stolperte herum“ trifft es genau; ich fand die Frage nicht.

### "hervorragend" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es war hervorragend“, antwortete ich.

### "hier" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie sind hier ..

### "hier" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich brauche ihn gegen zwei hier … ja, ich habe schon von Gegenwind gehört.

### "hierhin" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum kommt er denn nicht hierhin ?

### "hin" + "fahren" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lassen Sie uns doch heute abend mal irgendwo hinfahren!

### "hin" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Das kommt hin«, antwortete er.

### "hin" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das kriege ich hin“, sagte er.

### "hinuntersteigen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Martina hatte »hinuntersteigen« gesagt, und hier war ich.

### "holen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lass mich Antonio holen“, flüsterte sie.

### "hurt" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Und ich kann mit dieser hurt'gen Hand zum Beutelschneiden gut mich entwickeln.

### "hören" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Warte nur, bis meine Freundinnen davon hören“, erwiderte sie.

### "ich" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vorher verrecke ich !

### "ich" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das heißt für mich, dass ich - seine Geliebte werde.

### "ich" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Das regel ich«, antwortete er.

### "ihm" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - So wie ihr angezogen seid, sag nicht nur ich zu ihm "sie".

### "ihnen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bist eine von ihnen“, sagte Bombastico.

### "ihr" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was habt ihr "nicht gesehen"?

### "ihr" + "‘s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wollt ihr‘s sehen?

### "ihre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Mädchen können ihre "persönlichen" Sachen mitnehmen.

### "ihre" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann flogen ihre … Maschinen … ihr Segelboot hinunter zur Bucht.

### "ihrem" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit ihrem … Baby.

### "im" + ""Tang" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der Koch war früher im "Tang" in Shanghai.

### "immer" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich tue das nicht immer .

### "in" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist unter den jungen Leuten jetzt „in“, gebrauchte Kleidung zu tragen.

### "in-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will, dass Sie diese Wilden in- und auswendig kennenlernen.

### "ins" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Keiner geht ins "Chicken On A Stick".

### "ins" + ""Piramida" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich gehe entweder ins "Piramida" oder ins, Planetarium.

### "investierte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fast fünfzehn Teile auf zehn investierte“, antwortete er.

### "irgendwelche" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er will wissen, ob sie dir irgendwelche … Ideen mitgegeben haben.

### "ist" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jedes Mal, wenn eine Manifestation im Gange ist !

### "ist" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn der Druck zu groß ist - wird das Boot zerquetscht.

### "jeder" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Genauen Angaben nach ist jeder "Unschuldige" ein Feind der Nation.

### "jedes" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fast jedes“, antwortete er.

### "jene" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es geht darum, jene "immense Kraft" zu entfesseln, bevor die Nazis es tun.

### "jetzt" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich kann darüber jetzt - nicht reden.

### "jetzt" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie fühlst du dich jetzt ?

### "kaisha" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „kaisha“ auf Englisch?

### "kamen" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir kamen - vom Festland, als der Krieg anfing.

### "kauft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Derjenige, der ihn kauft“, sagte ich.

### "kebap" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maria hat ihren Neffen „Şiş kebap“ gemacht.

### "keine" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bücher habe ich hier keine“, sagte ich.

### "keiner" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will das Baby keiner "gesunden" Familie geben.

### "kennenlernen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich würde sie gern kennenlernen“, verkündete ich.

### "klar" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eine beplankte Dory, klar“, antwortete ich.

### "kleinen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir beginnen wieder mit dem großen und dem kleinen "Ha".

### "komme" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich komme«, sagte ich.

### "kommen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er wird mit mir zur Arbeit kommen !

### "kommen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wann darf Ihr kleiner Prinz zu Ihnen ins Bett kommen ?

### "kommen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber wir bauen ihn, und dann wird er schon kommen“, warf Johanna ein.

### "kommt" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er kommt !

### "konntest" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du hast das so gut erledigt, wie du konntest“, sagte er, als er herankam.

### "kriegst" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du kriegst'n Vorsprung.

### "kurz" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nur ganz kurz "Danke" sagen.

### "können" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Catull hätte schreiben können "Ich liebe und hasse", aber er tat es nicht.

### "könnte" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer sonst könnte …?

### "kümmern" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Sie und Ihr Rechtsberater müssen sich darum kümmern«, stimmte Laneson zu.

### "kündigen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tut mir leid, habe ich gesagt, dass sie kündigen "muss"?

### "küssen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wussten Sie, dass die Leute mit der Zunge küssen ?

### "lang" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Versuchen wir es drei Tage lang“, sagte ich.

### "lassen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom folgte immer der Devise „leben und leben lassen“.

### "laufen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Langsam, ich will nicht auf Grund laufen“, antwortete er.

### "lehrt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Professor Quirrell lehrt "Verteidigung gegen die dunklen Künste".

### "leiderlicherweise" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Leider gibt es das Wort „leiderlicherweise“ nicht.

### "leiht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ja, ich habe gesagt, dass er sich auch in Loricum Geld leiht“, antwortete ich.

### "lese" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich lese "Der Graf von Monte Christo" nochmal.

### "liebe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich liebe "Cops", die TV-Serie.

### "lieben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nach „lieben“ ist „helfen“ das schönste Zeitwort der Welt.

### "lieber" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tatsächlich wollte ich immer lieber "Tereza Victoria" sein.

### "liegen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bugleinen liegen“, meldete Paulson, „und sind eingeholt.

### "losmusst" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Die erste Ladung kommt aus dem Ofen, bevor du losmusst«, verkündete sie.

### "lächerlich" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist doch lächerlich“, erwiderte sie.

### "läuft" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Heute läuft "Marathon im Herbst".

### "lüften" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Na gut, es ist Zeit, mein Geheimnis zu lüften“, sagte ich.

### "mach" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja, mach'n Abflug!

### "mache" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich mache "Witch Watch".

### "machen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jetzt werden wir eine Photographie machen !

### "machen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was muss ich machen ?

### "machst" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was machst'n als erstes, wenn du hier raus bist?

### "macht" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Schnabeltier macht "fft, fft, fft"!

### "mag" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich mag "Indiana".

### "mal" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du wirst mal'n ganz Harter, wenn du groß bist.

### "mal" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Warte mal“, sagte er.

### "manchmal" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Berührst du dich manchmal ?

### "manchmal" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was meinst du mit „manchmal“?

### "manövriert" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du hast das Schiff hervorragend manövriert“, sagte Sparhawk.

### "mein" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich mein … Nein.

### "meine" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Deine Welt ist nicht wie meine“, sagte Arthur.

### "meint" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie meint "Weight Watchers".

### "meinten" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ja, wir meinten "uns".

### "mich" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lass mich !

### "mich" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Aber meine Fische warten auf mich«, sagte ich.

### "mich" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das erinnert mich“, sagte ich, „vielleicht sollten wir Tina Tinasdotr befördern.

### "mir" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das gefällt mir“, sagte ich.

### "mit" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kerl mit'm großen Schwanz scharf ist.

### "mit" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht solltest du dir überlegen, was du mit … machen willst.

### "mixt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er mixt "Non, je ne regrette rien" von Edith Piaf unter das Lied.

### "morgen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das »morgen« sagte er mit einem gespielten Knurren.

### "männliche" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es gibt keine „männliche“ Art zu lernen.

### "möchte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich möchte "Duftende Blume".

### "möchte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ich tun möchte“, sagte ich, „ist, ein paar neue Dinge hinzuzufügen.

### "mögen" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und wir mögen … eine von euch.

### "müssen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie müssen "Schützengräben" machen.

### "müssen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Es tut mir leid, Ihnen Ihre Zeit stehlen zu müssen«, sagte er.

### "n" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Aus irgendeinem Grund habe ich „m“ gelesen und nicht „n“.

### "nachdenken" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darüber muss ich nachdenken“, sagte er.

### "nachgedacht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darüber habe ich nachgedacht“, sagte ich.

### "nahr-" + "und" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Avocado ist eine nahr- und schmackhafte Frucht.

### "natürlich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich dachte auch an "Das Auge" und natürlich "Taucherglocke".

### "nen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Geh doch in nen "Sex-Shop".

### "nettes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das war ein nettes "Huhu".

### "neugierig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie machen mich neugierig“, sagte ich.

### "nicht" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Schau nicht !

### "nicht" + ""beteilige" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Weil sie es hasst, wenn ich nur herumsitze und mich nicht "beteilige".

### "nicht" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tsao gehört also nicht - zu den Vertrauten.

### "nicht" + ">>" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wieso <<entsprechen meine Angaben der Wirklichkeit fast gar nicht>>?

### "nicht" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht will ich gar nicht … Moment!

### "nie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du nie "Moidodyr" gelesen?!

### "niemals" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Man soll niemals "nie" sagen.

### "niemand" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Will niemand "Guten Morgen" sagen?

### "nokedli" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kilonis isst gern „nokedli“, ein ungarisches Nudelgericht.

### "nur" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wieso nur ?

### "nur" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist nur … Nun, ich mag sie wirklich.

### "oben" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dort oben !

### "oder" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gab es bei euch irgendwelche Schultraditionen oder -rituale?

### "oft" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Haben Sie gesehen wie oft ?

### "oft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie wiederholen sich oft“, sagte Maurice.

### "oh" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Gebrauch von „oh“ im Englischen ist ziemlich kompliziert.

### "otoshidama" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist das, was wir ein „otoshidama“ nennen.

### "packen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie sollten aber nichts packen“, sagte Victor.

### "passiert" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was ist passiert ?

### "passiert" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Du kannst versuchen, dir auszurechnen, was passiert«, sagte er.

### "persoctrēow" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pfirsichbaum“ heißt auf Altenglisch „persoctrēow“.

### "pflocken" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich werde dich in den Boden pflocken !

### "r" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom spricht den Buchstaben „r“ falsch aus.

### "ratlos" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin ratlos“, sagte ich.

### "rauf" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie kam er da rauf ?

### "raufgekommen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie ist er da raufgekommen ?

### "raus" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Komm schon, raus !

### "raus" + "geboxt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie oft habe ich dich irgendwo rausgeboxt?

### "raus" + "zureden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nenn mich nicht "Schatz", um dich da rauszureden, verdammt!

### "rausschmeißt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn der Chef einen „rausschmeißt“, heißt das, dass man entlassen ist.

### "rechtfertigen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Genug, um den Preis zu rechtfertigen“, sagte Kat.

### "reden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir müssen reden“, sagte sie.

### "rein-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es soll niemand rein- oder rausgehen.

### "reinkopiert" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du einfach reinkopiert "Dschungel"?

### "reinrassiger" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lucky ist ein reinrassiger "Panthera tigris amoyensis".

### "richtig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ganz richtig“, sagte ich.

### "rosa" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Genitiv Singular von „rosa“ lautet „rosae“.

### "rosae" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Genitiv Singular von „rosa“ lautet „rosae“.

### "rufen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie rufen "Lebwohl" oder "Willkommen daheim".

### "ruft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diejenige, der gehorcht werden muss, ruft“, verkündete er.

### "s" + "paar" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zwillingspaar.

### "sag" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mund auf und sag "A".

### "sagst" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hallo, Schwager, wenn du sagst "alles gut", tritt er.

### "sah" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich sah , wie er die Straße überquerte.

### "sauber" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich mach sauber«, verkündete er.

### "schaffen" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und mit harter Arbeit kann man das schaffen … in nur einem Jahr!

### "schau" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Onkel, schau !

### "schauen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kommt schauen !

### "schert" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wen schert "der Rumäne"?

### "schieben" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Diese immer rein und raus schieben ?

### "schliessen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich muss schliessen !

### "schockiert" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin schockiert, wirklich schockiert“, sagte Ehlers.

### "schreib" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Äh, ich schreib's euch auf.

### "schreiben" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie schreiben "jung"!

### "schreiben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bringst Leute dazu, für dich zu schreiben“, sagte ich.

### "schreibt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nicholas hatte sein Buch „Wie man Belletristik schreibt“ fertiggestellt.

### "schreien" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir schreien "Hurra" und brechen die Tür auf.

### "schuldig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Klient will auf „schuldig“ plädieren.

### "schwer" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sind die nicht schwer ?

### "schwer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kapitänin, das Ruder ist schwer“, begann er.

### "schwierig" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist mitunter schwierig "richtig" und "falsch" auseinanderzuhalten.

### "schwul" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zwischen „schwül“ und „schwul“ besteht nur ein kleiner Unterschied.

### "schwül" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zwischen „schwül“ und „schwul“ besteht nur ein kleiner Unterschied.

### "schön" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hör mal, Akira, was soll denn hier bitte schön "schon wieder" heißen?

### "schön" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bitte schön“, sagt die Bordbedienung.

### "sehe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich sehe“, sagte der Blinde.

### "seien" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie seien "irre".

### "sein" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Möge das Paar auf ewig glücklich sein !

### "seinen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ja, er nennt sie seinen "kleinen Koch".

### "seinen" + ""Agrippa" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sofern der Gegner nicht seinen "Agrippa" studiert hat.

### "selbst" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Na ja, das "Buzz" ist in Ordnung, aber wer nennt sich selbst "Cocks"?

### "selbst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wahrhaftig, Watson, Sie übertreffen sich selbst“, sagte Holmes.

### "serviert" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das Essen wird umgehend serviert“, verkündete ich.

### "setzen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Clark, du kannst dich gern zu uns setzen«, sagte ich.

### "sich" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ihre Stärke ist, dass sie sich - für Frau Mak hält.

### "sieben" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „sieben“ ließen natürlich James und Janet außen vor.

### "siehst" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du siehst "Wahrheit" als etwas Absolutes.

### "sieht" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kann man verhindern, sich zu berühren, wenn man sie im Tabakladen sieht ?

### "simples" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ein simples "Hin und weg" also.

### "singen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kellner singen "Happy Birthday" auf Chinesisch.

### "singt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer singt "La Fandunga"?

### "so" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bleibt so !

### "so" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das, was man so "Liebe" nennt.

### "so" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ist das richtig so ?

### "soap" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „soup“ oder „soap“ gesagt?

### "sogenannten" + ""Zyklus" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie hat es jedoch nie geschafft, den sogenannten "Zyklus" zu vollenden.

### "soll" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe mich gefragt, wie das funktionieren soll“, sagte Maurice.

### "sollen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich hätte von einem Kondom Gebrauch machen sollen“, jammerte Tom.

### "solltet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die ihr wahrscheinlich erst einmal abschließen solltet“, sagte ich.

### "sondern" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nur heißt es nicht mehr "Liebe", sondern "Weihnachten".

### "soup" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „soup“ oder „soap“ gesagt?

### "spiele" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Setz dich wieder und spiele !

### "spielen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du gehst ins Haus, und hörst auf, Theater zu spielen !

### "spielen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie singen und spielen "A Te O Cara, Amor Talora" von Popol Vuh.

### "spindel" + "dürr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du liebe Zeit, die Kleine ist spindeldürr.

### "spät" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ein bisschen zu früh ist besser als zu spät“, flüsterte Dietrich.

### "stecken" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich werde dir den Stuhl in den Hals stecken !

### "stellen" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hermann Göring vor Gericht stellen ..

### "still" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sei still“, flüsterte er.

### "strawberries" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Warum heißen Erdbeeren im Englischen „strawberries“?

### "ständig" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vor der Kamera denke ich ständig "Was macht Liliane?"

### "stören" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich will ein glückliches Paar nicht stören«, erwiderte Clark.

### "süßen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dabei findest du einen Weg zur Rettung deiner süßen "wie heißt sie noch mal".

### "tief" + "em" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Man hätte meinen können, wir befänden uns in hundert Fuß tiefem Wasser.

### "tief" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Es ist überall drei Meter tief“, sagte Louise.

### "tot" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Vater ist seit fast zwei Jahren tot“, antwortete ich.

### "trotzdem" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will aber trotzdem "Ich sehe was, was du nicht siehst" spielen.

### "tun" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Dagegen kann ich etwas tun«, sagte ich.

### "um" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und für dich ist es zu kalt, um … zu helfen.

### "umbringen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Er wollte mich umbringen“, sagte ich leise.

### "umgebracht" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Die Bellers haben eine ganze Reihe meiner Freunde umgebracht«, sagte Rudolph.

### "umstrittener" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alles kann und sollte übersetzt werden, einschließlich „umstrittener“ Sätze.

### "unberechenbar" + "er" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ein weltfremder, unberechenbarer Künstler!

### "unglaublich" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist ja unglaublich“, sagte ich.

### "uns" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie hat uns "Arschloch" genannt.

### "uns" + "'res" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie wohl von uns'res werten Bruders Tod noch das Gedächtnis frisch.

### "uns" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das wäre was für uns“ sprach der Hahn.

### "unsere" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Uns so ging unsere "Zwangsbildung" zugrunde.

### "unseres" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Anlässlich der Feier des ganz besonderen Tages unseres … Geburtstagsjungen.

### "unter" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte »unter«, nicht »vor«.

### "unüberlegt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und vor allem, dass ich so unüberlegt "ja" gesagt habe.

### "ursprünglich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Projekt hieß ursprünglich "Atlantic Plaza Residence".

### "variabel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - In der heutigen Zeit ist „variabel“ ein Schimpfwort.

### "verabschieden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich muss mich nun verabschieden“, sagte er plötzlich.

### "verbracht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie haben noch nie Zeit auf dem Wasser verbracht“, sagte Michael.

### "verdammt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ach, verdammt“, sagte ich.

### "verdient" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie habe ich das verdient ?

### "verflochten" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vieles ist irgendwie mit ihm verflochten“, erwiderte David.

### "verfluchen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Gott möge euch alle verfluchen !

### "vergiften" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich werde euch vergiften !

### "verheiratet" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich bin bereits verheiratet“, antwortete Maria.

### "verkaufen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir haben allerlei Dinge zu verkaufen“, sagte Kat.

### "verkauft" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber er wird für doppelt so viel verkauft“, sagte Christopher.

### "verkünden" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kannst du nicht wenigstens verkünden "Ein Kind ist uns geboren"?

### "verletzen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du wirst dich noch verletzen !

### "verletzt" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du dich verletzt ?

### "vermissen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du wirst uns bestimmt vermissen !

### "versammelt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir sind durch die Glocke versammelt“, intonierten die Anwesenden.

### "verschlossen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wer hat das Tor verschlossen ?

### "verstehe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie sagt "Danke", und ich verstehe "Hau ab".

### "verstehe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich verstehe“, antwortete ich.

### "verstehen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich werde die Frauen nie verstehen“, sagte Tom.

### "versucht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das haben wir versucht“, antwortete Jacob Williamson.

### "verteilt" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe die NASA-Aufkleber überall in meinem Zimmer verteilt“, sagte sie.

### "verweht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du schon einmal „Vom Winde verweht“ gelesen?

### "verwendest" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du verwendest "Bonettis Verteidigung" gegen mich, huh?

### "viel" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ganz schön viel "Wir".

### "vielleicht" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dann vielleicht "abnormal"?

### "vielleicht" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Subversive Propaganda vielleicht ?

### "vielleicht" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe „vielleicht“ gesagt.

### "vier-" + "oder" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vielleicht vier- oder fünfmal so groß.

### "voll" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zu meiner Zeit waren die voll "in".

### "voller" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eine ganze große Welt voller "Nein".

### "vor" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte »unter«, nicht »vor«.

### "vor" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kommt vor“, sagte ich.

### "voran" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich komme mit dem Buch über Zeitungsmanagement voran“, sagte Maurice.

### "voraus" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Der Teich liegt voraus“, sagte Vicki.

### "vorhast" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe überall erzählt, was du mit den Gefangenen vorhast“, sagte Sparhawk.

### "vorvorvorletztes" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieses Satzes vorvorvorletztes Wort ist „vorvorvorletztes“.

### "war" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hey, ich weiß, mit uns zu kommen, war … hart für dich.

### "warst" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du warst "Bounty Law".

### "warum" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich verstehe nicht, warum“, sagte Sparhawk.

### "was" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich tauchte meine Feder ein und schrieb „wer“, „was“ und so weiter auf.

### "weg" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wirf diesen Stein weg !

### "weiter" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mach weiter !

### "weiterfahren" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Können wir weiterfahren !

### "weiß" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Denn ich weiß - mehr als ihr.

### "wenig" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist sehr wenig“, sagte Kat.

### "wenigstens" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du solltest wenigstens "danke" sagen.

### "wenn" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es fehlt das Komma vor „wenn“.

### "wer" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich tauchte meine Feder ein und schrieb „wer“, „was“ und so weiter auf.

### "werden" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aber ich will, dass sie oben am Landefeld gebaut werden“, fügte ich hinzu.

### "wertlos" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diese Schuldscheine sind so gut wie wertlos“, antwortete ich.

### "wieder" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dein Bruder meldet sich wieder - erzähl ihm von deinen Geschichten.

### "will" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich will "Der schöne Gleichgültige" von Cocteau inszenieren.

### "will" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was ich will … ist, von hier verschwinden.

### "wir" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wohin gehen wir ?

### "wirklich" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich kann es wirklich !

### "wirklich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sagte er wirklich "Industriekur"?

### "woanders" + "hin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir werden woandershin gehen.

### "wohl" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das nennt man wohl "bluffen".

### "wollen" + "?" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Warum würdest du Lehrer werden wollen ?

### "wollte" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „In deinem letzten Brief war alles enthalten, was ich wollte“, sagte ich.

### "wortwörtlich" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir haben - wortwörtlich - einen Haufen zu Essen.

### "wurde" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Von Miller wurde "Daumen hoch!" gepingt, von Dr. Mann auch.

### "wurden" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie wurden "nicht umsonst geboren".

### "wär" + "‘" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie wär‘s, wenn ich dich auf einen Kaffee einlade?

### "wären" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich war nicht sicher, ob Sie noch hier wären“, sagte ich.

### "wärst" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Stellst du Tina als deine Freundin vor, könnte man denken, du wärst "es".

### "wärst" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du wärst - in Schwierigkeiten ohne mich.

### "wünschen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie wollten gerade erzählen, was Sie wünschen“, sagte ich.

### "you" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was bedeutet „How are you“?

### "zerbrechlich" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf der Schachtel ist ein Aufkleber mit der Aufschrift „zerbrechlich“.

### "zieht" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Und er zieht "Monsieur" als Anrede vor.

### "ziemlich" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er sagte, Sie verhielten sich ziemlich "kastrierend".

### "zitierst" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du hasst den Krieg, aber du zitierst "Die Kunst der Kriegsführung"?

### "zu" + "'n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er macht ab und zu'n Job für mich.

### "zu" + "geben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom weiß auch nicht, was los ist, aber er will es nicht zugeben.

### "zu" + "getragen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ebendas hat sich zugetragen.

### "zu" + "wedeln" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist gefährlich, mit einer Klinge so herumzuwedeln.

### "zuerst" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du zuerst“, sagte Nicholas und deutete auf Vincent.

### "zugenommen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Er hat aus Nordwesten allmählich zugenommen“, antwortete Jennifer.

### "zurück" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir holen es zurück“, verkündete ich.

### "zurückgekommen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bin gerade zurückgekommen“, antwortete ich.

### "zurückkommen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sie könnten zurückkommen“, erwiderte Nicholas.

### "zurücklassen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wir sollten ein paar persönliche Dinge zurücklassen“, sagte Maurice.

### "zurücksetzen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du auf „zurücksetzen“ gedrückt?

### "zusammen" + "!" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Näher zusammen !

### "zusammen" + "gesetzt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Und du bist wieder ganz zusammengesetzt.

### "zwei" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Na, ihr zwei "Mütter"?

### "zwei" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Nummer zwei«, fuhr ich fort.

### "zwei" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „In einem Jahr, vielleicht auch in zwei“, antwortete ich.

### "zwischen" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist ein großer Unterschied zwischen "überwiegend tot" und "völlig tot".

### "zählen" + "«" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ich muss das erst zählen«, erklärte er.

### "«" + "Ihre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Oh nein.« Ihre Stimme hellte sich auf.

### "«" + "Keith" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Also bin ich unserem trockenen Flusslauf stromaufwärts gefolgt.« Keith kam mit.

### "«" + "Laneson" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Ja.« Laneson sah nicht aus, als erwarte er Geld.

### "«" + "Meine" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Aber wir haben ihnen doch das ganze Geld gegeben …« Meine Stimme verebbte.

### "«" + "Sie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Eins, zwei, drei …« Sie rammten meine Schlafzimmertür.

### "«" + "Stille" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Wir wissen, dass sie heute ausziehen muss.« Stille.

### "«" + "bedeuten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Was »Nau Kaperkorn ist für« bedeuten soll, erschließt sich mir überhaupt nicht.

### "«" + "dachte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Was ist mit mir geschehen?« dachte er.

### "«" + "genannt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Deutschland wird auf Deutsch »Deutschland« genannt.

### "«" + "gesagt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Martina hatte »hinuntersteigen« gesagt, und hier war ich.

### "«" + "kam" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Immer zusammen!« kam es laut aus dem Raum zurück.

### "«" + "lautete" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Qualitätsfisch für Qualitätsleute« lautete ihr Wahlspruch.

### "«" + "schrie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sieh nur, Vater,« schrie sie plötzlich auf, »er fängt schon wieder an!

### "«" + "zu" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich bin nur gekommen, um »Tschüß« zu sagen.

### "«" + "»" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - »Tom!« »Mary!«

### "»" + "Abgemacht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Abgemacht!«, rief ich zurück.

### "»" + "Ahoi" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ahoi, Dory!«, rief er.

### "»" + "Alkohol" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Alkohol?«, fragte ich.

### "»" + "Allerdings" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Allerdings«, antwortete Chiang.

### "»" + "Bis" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Bis jetzt habe ich heute einen Scone gegessen.«

### "»" + "Clark" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Clark, du kannst dich gern zu uns setzen«, sagte ich.

### "»" + "Computer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Computer.«

### "»" + "Dagegen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Dagegen kann ich etwas tun«, sagte ich.

### "»" + "David" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »David Robertson ist gerade knapp einem Mordanschlag entgangen«, sagte er.

### "»" + "Deine" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Deine Zimtschnecken werden mir fehlen«, sagte ich.

### "»" + "Deutschland" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Deutschland wird auf Deutsch »Deutschland« genannt.

### "»" + "Eins" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Eins, zwei, drei …« Sie rammten meine Schlafzimmertür.

### "»" + "Einverstanden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Einverstanden«, verkündete er.

### "»" + "Entschuldigung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Entschuldigung«, stand auf dem Zettel in meinem Briefkasten.

### "»" + "Erstens" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Erstens dazu, dass du jetzt Rower’s Folly gehörst«, sagte er.

### "»" + "Eure" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Eure Kleidung liegt für euch bereit.«

### "»" + "Freut" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Freut mich, dass es euch schmeckt!«

### "»" + "Gebrauche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Gebrauche die Kreide, um eine Tür zu zeichnen, irgendwo in deinem Zimmer.«

### "»" + "Gregor" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Gregor, Gregor,« rief er, »was ist denn?«

### "»" + "Großartig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Großartig!«, sagte ich.

### "»" + "Gut" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Gut, eine vorsichtige Schiffsführerin zu haben«, sagte sie.

### "»" + "Gute" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Gute Fahrt!«, rief er.

### "»" + "Guten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Guten Fang!«, antwortete ich.

### "»" + "Haben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Haben all diese Läden denselben Grundriss?«, fragte Jim.

### "»" + "Herein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Herein!«, rief ich.

### "»" + "Hier" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Hier ist niemand.«

### "»" + "Hilfe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Hilfe!«, schrie sie.

### "»" + "Ihr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ihr seid Meister Chiang?«, fragte eine von ihnen.

### "»" + "Ist" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Ist das Spencerport auf Starlanding?«

### "»" + "Jedenfalls" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Jedenfalls alle gleich«, antwortete ich.

### "»" + "Kann" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Kann das überhaupt funktionieren?«, fragte er.

### "»" + "Klar" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Klar.«

### "»" + "Kommerziell" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Kommerziell«, antwortete ich.

### "»" + "Lamm" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Lamm und Kohl klingt gut«, sagte ich.

### "»" + "Lange" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Lange Geschichte«, verkündete ich.

### "»" + "Laß" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Laß dich von den Feen leiten.«

### "»" + "Leute" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Leute?«, schaltete sich schließlich Jill Teranikesdotr ein.

### "»" + "Mary" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Tom!« »Mary!«

### "»" + "Miau" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Kuh macht »Muh!«, die Katze macht »Miau!«.

### "»" + "Mit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Mit Geld«, antwortete ich.

### "»" + "Morgen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Morgen«, antwortete ich.

### "»" + "Muh" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Kuh macht »Muh!«, die Katze macht »Miau!«.

### "»" + "Muss" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Muss ich Opa fragen«, sagte sie.

### "»" + "Na" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Na, vielen Dank«, sagte ich.

### "»" + "Nau" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was »Nau Kaperkorn ist für« bedeuten soll, erschließt sich mir überhaupt nicht.

### "»" + "Nein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Nein,« sagte Gregor.

### "»" + "Nun" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Nun?« fragte Herr Samsa.

### "»" + "Nur" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Nur sich nicht im Bett unnütz aufhalten,« sagte sich Gregor.

### "»" + "Qualitätsfisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Qualitätsfisch für Qualitätsleute« lautete ihr Wahlspruch.

### "»" + "Reiten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Reiten wir?«, fragte er.

### "»" + "Schneller" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Schneller«, sagte ich zwischen zwei Bissen Obst.

### "»" + "Schon" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Schon satt, oder hast du Hunger?«

### "»" + "Sind" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Sind wir hier bereit?«, fragte ich.

### "»" + "Soll" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Soll ich hinausgehen?«, fragte Chiang.

### "»" + "Sortieren" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Sortieren und nachzählen?«, fragte ich, als ich fertig war.

### "»" + "Spencerport" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Spencerport?«, kam die Antwort.

### "»" + "Südlich" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Südlich des Äquators gibt es eine Menge davon«, sagte er.

### "»" + "Tim" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Tim ist es«, drang die Stimme durch die Tür.

### "»" + "Tom" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Tom!« »Mary!«

### "»" + "Tschüß" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich bin nur gekommen, um »Tschüß« zu sagen.

### "»" + "Verstanden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Verstanden«, sagte Terry.

### "»" + "Verstehe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Verstehe«, sagte ich, völlig verblüfft.

### "»" + "Vielleicht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Vielleicht Unterhaltung?«, fragte ich.

### "»" + "Wann" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Wann hörst du auf?«, fragte Rudolph.

### "»" + "Waren" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Waren, die wir transportieren.« Ich hatte inzwischen Konkurrenz.

### "»" + "Warum" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Warum überrascht mich das nicht?«, fragte er.

### "»" + "Welche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Welche Segelerfahrung hast du?«

### "»" + "Wer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maschinen sind kein »Wer«, und denken können nur Menschen.

### "»" + "Wie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Wie viel?«, fragte ich.

### "»" + "Zweimal" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Zweimal?«, fragte ich.

### "»" + "Zählt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Zählt es, wenn ich am Verhungern bin?«, fragte ich.

### "»" + "aber" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe ja Appetit,« sagte sich Gregor sorgenvoll, »aber nicht auf diese Dinge.

### "»" + "bin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - »Auf Nova Capricornis vier«, verkündete er, »bin ich beinahe ertrunken.«

### "»" + "das" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Weg muss es,« rief die Schwester, »das ist das einzige Mittel, Vater.

### "»" + "er" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sieh nur, Vater,« schrie sie plötzlich auf, »er fängt schon wieder an!

### "»" + "es" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Gregor,« rief es – es war die Mutter –, »es ist dreiviertel sieben.

### "»" + "hinuntersteigen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Martina hatte »hinuntersteigen« gesagt, und hier war ich.

### "»" + "macht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dies frühzeitige Aufstehen«, dachte er, »macht einen ganz blödsinnig.

### "»" + "morgen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das »morgen« sagte er mit einem gespielten Knurren.

### "»" + "nicht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Meistens stand in den Worten darin »nicht geladen«.

### "»" + "offenes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Wellen waren »offenes Meer«, nicht »Äußere Bucht«, aber noch zu bewältigen.

### "»" + "unter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte »unter«, nicht »vor«.

### "»" + "vor" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich sagte »unter«, nicht »vor«.

### "»" + "Äußere" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Wellen waren »offenes Meer«, nicht »Äußere Bucht«, aber noch zu bewältigen.

### "Äh" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Äh … äh … was?

### "Öllampe" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eine Öllampe“, sagte Arthur.

### "Übel" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Für „das kleinere Übel“ entschieden.

### "Übertreibung" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hyberbel“ ist ein hochtrabender Ausdruck für „Übertreibung“.

### "Übrigens" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Übrigens“, sagte ich, „ich habe versucht, in Torfleet und Castlefleet anzulegen.

### "übersetzt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tokyo heißt übersetzt "östliche Hauptstadt".

### "überspringen" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie ich sehe, können wir das Krafttraining überspringen“, verkündete er.

### "überweisen" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir müssen es überweisen …alles wird gut, Fräulein.

### "злой" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „злой“ ist das einzige einsilbige russische Eigenschaftswort.

### "–" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Was geschah hier?" – "Nichts."

### "—" + "Maria" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Christa—Maria Sieland.

### "’s" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Die Orangerie ist neben Polly’s“, antwortete er.

### "“" + "Agatha" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ich muss das Zeug zwei Wochen lang nehmen, stimmt’s?“ Agatha nickte.

### "“" + "Art" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt keine „männliche“ Art zu lernen.

### "“" + "Auch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Chris Markofferson hat sein Geld nicht hier aufbewahrt.“ Auch das war Yus Werk.

### "“" + "Aurora" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Haben sie Vincent gefangen genommen?“ Aurora klang wirklich beunruhigt.

### "“" + "Einige" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Die wissen Bescheid.“ Einige Nachbarn lachten.

### "“" + "Ernie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Er hatte fünfzig Stück, alles, was er für seine Arbeit brauchte.“ Ernie nickte.

### "“" + "Heilerin" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Bist du das, Amanda?“ Heilerin Agathas Stimme drang den Korridor entlang.

### "“" + "Kontakt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich halte zu meinen Freunden und Verwandten über „Facebook“ Kontakt.

### "“" + "Louise" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ihr scheint das ja in großem Maßstab zu betreiben.“ Louise nannte einen Betrag.

### "“" + "Mein" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Kleines Mädchen?“ Mein Tonfall war vermutlich etwas eisig.

### "“" + "Na" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Und wie waren die Preise beim letzten Einkauf?“ Na gut.

### "“" + "Naismith" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „In den üblichen Fässern zu fünfzig Pfund?“ Naismith nickte.

### "“" + "Piotr" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Vielen Dank.“ Piotr verbeugte sich und trat rückwärts zur Tür hinaus.

### "“" + "Sein" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Besetzen Sie die …“ Sein Trupp rannte bereits zur Heck-Saugpumpe.

### "“" + "Sie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Für dich.“ Sie wandte den Blick ab.

### "“" + "Sparhawk" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Herein.“ Sparhawk saß am Kapitänstisch, vor sich eine reichliche Mahlzeit.

### "“" + "Sätze" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alles kann und sollte übersetzt werden, einschließlich „umstrittener“ Sätze.

### "“" + "an" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom stimmte „Zum Geburtstag viel Glück!“ an.

### "“" + "anstatt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Den Mexikanern gefällt es nicht, wenn man „Méjico“ anstatt „México“ schreibt.

### "“" + "antworteten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ach nein,“ antworteten sie, „die Bedingungen sind zu schwer.

### "“" + "auch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe „Krieg und Frieden“ auch noch nicht gelesen.

### "“" + "ausgesprochen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Leicester“ wird „Lester“ ausgesprochen.

### "“" + "bedeutete" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich erklärte, was „gut“ bedeutete.

### "“" + "befahl" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Die Landungsplattform raus!“ befahl Captain Sparhawk.

### "“" + "befindet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das „ovale Büro“ befindet sich im Weißen Haus.

### "“" + "bei" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir würden uns über ein „Gefällt mir!“ bei Facebook freuen!

### "“" + "bekam" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es rief „guten Morgen,“ bekam aber keine Antwort.

### "“" + "bekannt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Stadt Recife ist als „Venedig Brasiliens“ bekannt.

### "“" + "bemerkte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Das ist aber ein hübsches Mädchen!“ bemerkte ich.

### "“" + "bestanden" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Er hat den Kurs mit einem „gut“ bestanden.

### "“" + "besteht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Zwischen „schwül“ und „schwul“ besteht nur ein kleiner Unterschied.

### "“" + "bevor" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom steht „lebenslänglich“ bevor.

### "“" + "bezieht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Stärke“ bezieht sich nicht nur auf unseren Körper.

### "“" + "bilden" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich hoffe, dass die SPD keine Koalition mit der Partei „Die Linke“ bilden wird.

### "“" + "bis" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nehmen Sie den Bus Richtung „Badstraße“ bis zur Seestraße.

### "“" + "dabei" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Keith hatte ich als meinen „guten Freund“ dabei.

### "“" + "dachte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ach, warum hört sie mich denn nicht?“ dachte Echo bei sich.

### "“" + "das" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nach „lieben“ ist „helfen“ das schönste Zeitwort der Welt.

### "“" + "dauerte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Der „Marsch in die Städte“ dauerte viel länger, als ich gedacht hätte.

### "“" + "der" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Raumsonde „Juno“ der NASA ist in eine Bahn um den Jupiter eingetreten.

### "“" + "die" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Schöne Kleider“ sagte die eine, „Perlen und Edelsteine“ die zweite.

### "“" + "diniert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Habt ihr schon mal in „Kalles Speisewagen“ diniert?

### "“" + "drohte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Geld her, oder es passiert was!“ drohte der Räuber.

### "“" + "eingestellt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieser Buchladen hat den Verkauf von „JUMP“ eingestellt.

### "“" + "enthalten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es gibt keine englischen Sätze auf Tatoeba, die das Wort „Lego“ enthalten.

### "“" + "entschieden" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Für „das kleinere Übel“ entschieden.

### "“" + "entwickelt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die ersten Raumanzüge der NASA wurden für das Programm „Merkur“ entwickelt.

### "“" + "erschien" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es erklang ein kaum hörbares Klicken, und hinter dem „a“ erschien ein „m“.

### "“" + "etwas" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Kannst du mit dem Begriff „Selbsterhaltung“ etwas anfangen?

### "“" + "falsch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom spricht den Buchstaben „r“ falsch aus.

### "“" + "fertiggestellt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nicholas hatte sein Buch „Wie man Belletristik schreibt“ fertiggestellt.

### "“" + "finden" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Welche verschiedenen Bedeutungen können Sie für das Wort „Satellit“ finden?

### "“" + "flehte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Tu’s nicht, Anne!“ flehte Diana. „Du fällst herunter und stürzt dich zu Tode!“

### "“" + "folgen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Man kann dem Papst jetzt auf „Twitter“ folgen.

### "“" + "forderte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Steigt denn vom Pferd und zieht!“ forderte Bucklaw.

### "“" + "fragen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Während die einen „Warum?“ fragen, fragen die anderen „Warum nicht?“

### "“" + "fragten" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Seid ihr denn nicht verwundet?“ fragten die Reiter.

### "“" + "früh" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „City“ ist gegen „United“ früh in Führung gegangen.

### "“" + "fuhr" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Hier also,“ fuhr sie fort, „werden die Belege aufbewahrt.“

### "“" + "gebaut" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom und Maria haben ein „Erdschiff“ gebaut.

### "“" + "gedrückt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du auf „zurücksetzen“ gedrückt?

### "“" + "gegessen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom hat im „Goldenen Drachen“ gegessen.

### "“" + "gehört" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hast du schon einmal etwas von „Tatoeba“ gehört?

### "“" + "gemacht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Maria hat ihren Neffen „Şiş kebap“ gemacht.

### "“" + "gespielt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wir haben nur „Verstecken“ gespielt.

### "“" + "gewesen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Toms Antwort wäre sicher „Nein!“ gewesen.

### "“" + "gewählt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieser Wagen wurde zum „Auto des Jahres“ gewählt.

### "“" + "ging" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die „Hindenburg“ ging plötzlich in Flammen auf.

### "“" + "haben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die „Löwen“ haben gegen die „Falken“ einen leichten Sieg errungen.

### "“" + "handelt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bei „ich“, „sie“ und „wir“ handelt es sich um persönliche Fürwörter.

### "“" + "herabgewürdigt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Frauen werden in vielen Liedern des Genres „Rap“ herabgewürdigt.

### "“" + "hieß" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Samurai lebten nach einem Ehrenkodex, der „Bushidō“ hieß.

### "“" + "höre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vom „Kagura-Tanz der Feuergottheit“ höre ich auch zum ersten Mal.

### "“" + "in" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist leicht, „ich bin“ in jede Sprache zu übersetzen.

### "“" + "lachte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ha, ha!“ lachte der Neffe von Scrooge. „Ha, ha, ha!“

### "“" + "lesen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich wollte schon immer mal „Krieg und Frieden“ lesen.

### "“" + "ließen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die „sieben“ ließen natürlich James und Janet außen vor.

### "“" + "meinst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Verwende nicht „entdecken“, wenn du eigentlich „erfinden“ meinst.

### "“" + "meinte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mit „hier“ meinte ich Australien.

### "“" + "miteinander" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom und Maria sprechen häufig über „Skype“ miteinander.

### "“" + "nannte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich wusste, dass man das „Tippen“ nannte, hatte ihn aber noch nie dabei gesehen.

### "“" + "nur" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Viele sind bei „Tinder“ nur auf eine kurze Beziehung aus.

### "“" + "plädieren" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mein Klient will auf „schuldig“ plädieren.

### "“" + "sage" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es ist das erste Mal, dass ich Taninna „Ich liebe dich!“ sage.

### "“" + "sagst" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wenn du „Ich liebe dich!“ sagst, dann werde ich das auch zu dir sagen.

### "“" + "sendet" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die „Stimme Amerikas“ sendet aus Washington.

### "“" + "seufzte" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Ach!“ seufzte sie.

### "“" + "stammt" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Das Wort „Golem“ stammt aus dem Hebräischen.

### "“" + "tituliere" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sie mag es nicht, wenn ich ihre Kinder als ihre „Brut“ tituliere.

### "“" + "tituliert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ich habe Tom als „fettes Schwein“ tituliert.

### "“" + "tragen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieses Eichhörnchen soll fortan den Namen „der Knuffige“ tragen.

### "“" + "trifft" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Stolperte herum“ trifft es genau; ich fand die Frage nicht.

### "“" + "umgehen" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom kann recht gut mit „Photoshop“ umgehen.

### "“" + "unterschrieben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Karte war mit „Von einem Mann, der dich attraktiv findet“ unterschrieben.

### "“" + "verloren" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „City“ hat gegen „United“ verloren.

### "“" + "veröffentlicht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Die Studie wurde im Journal „Science“ veröffentlicht.

### "“" + "wird" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Leicester“ wird „Lester“ ausgesprochen.

### "“" + "wurde" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Rosetta“ wurde von der Europäischen Weltraumorganisation gebaut.

### "“" + "zum" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sagt „Nein!“ zum Rassismus!

### "“" + "zurecht" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Wie kamen wir mit „nichts mehr auf Lager“ zurecht?

### "“" + "üben" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Diese „Freunde“ üben einen schlechten Einfluss auf dich aus.

### "”" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Warum studiert ihr Interlingua?” - „Weil es sehr einfach ist.”

### "”" + "Julie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - ”Julie Clapet."

### "”" + "Niemandskinder" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - ”Niemandskinder".

### "”" + "Robert" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - ”Robert ist ein Perverser, ein Scheißer, ein Unterhosengesicht."

### "„" + "Ab" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ab einem bestimmten Alter will niemand seine Eltern um Hilfe bitten“, sagt Li.

### "„" + "Abschicken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe eine Antwort geschrieben, aber vergessen, auf „Abschicken!“ zu drücken.

### "„" + "Aha" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aha!“ sagten jene.

### "„" + "Alice" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wem gehören diese Bücher?“ „Alice.“

### "„" + "Allerdings" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Allerdings“, erwiderte Heilerin Agatha.

### "„" + "Anarchokapitalisten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom bezeichnet sich als „Anarchokapitalisten“.

### "„" + "Anderthalb" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Anderthalb Standards, und keinen Pfennig mehr!“, erwiderte er.

### "„" + "Angst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er weiß nicht, was das Wort „Angst“ bedeutet.

### "„" + "Anime" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er mochte Zeichentrickfilme japanischer Art, auch „Anime“ genannt.

### "„" + "Anklagepunkte" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Anklagepunkte“, las ich.

### "„" + "Anscheinend" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das kann ich nicht.“ – „Anscheinend kann es keiner.“

### "„" + "Antonio" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Fast hätte ich „Antonio“ gesagt, aber das ging ihn nichts an.

### "„" + "Apotheker" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Apotheker hat die Frauen von Spencerport nach Strich und Faden ausgenommen.“

### "„" + "Arktos" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arktos“ heißt auf Griechisch Bär.

### "„" + "Astronaut" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „Astronaut“ kommt aus dem Lateinischen und bedeutet „Sternenschiffer“.

### "„" + "Auch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Auch wenn es eine Herausforderung wird, den hiesigen Akzent zu lernen.“

### "„" + "Aufschlüsseln" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aufschlüsseln”?

### "„" + "Aufseher" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aufseher“, erwiderte ich.

### "„" + "Aufwachen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Aufwachen!“ – „Ich bin wach!“

### "„" + "Augenblick" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bitte öffnen Sie die Tür.“ – „Augenblick bitte.“

### "„" + "Auto" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieser Wagen wurde zum „Auto des Jahres“ gewählt.

### "„" + "Automobil" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Automobil“ ist eine Hybridbildung.

### "„" + "Backbordleinen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Backbordleinen straffen und das Schiff damit zum Stillstand bringen.“

### "„" + "Badstraße" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nehmen Sie den Bus Richtung „Badstraße“ bis zur Seestraße.

### "„" + "Bald" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bald werde ich erheblich mehr Fisch fangen“, sagte ich.

### "„" + "Barlow’s" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Barlow’s“, antwortete Piotr.

### "„" + "Barzahlung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Barzahlung“, sagte er.

### "„" + "Beethoven" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was ist denn das?“ – „Beethoven.“ – „Das ist ja knorke! Mann, ist das knorke!“

### "„" + "Befand" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Befand sich in dem Zimmer eine Leiche?“ — „Nein, da war keine Leiche.“

### "„" + "Benutzt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Benutzt ihr nur Kerzen?“, fragte ich.

### "„" + "Bereitmachen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bereitmachen, Fahrt voraus aufzunehmen“, rief ich.

### "„" + "Berühr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Berühr den Bildschirm dort, wo dein Name steht.“

### "„" + "Beseda" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Beseda“ bedeutet auf Slowenisch „Wort“.

### "„" + "Besetzen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Besetzen Sie die …“ Sein Trupp rannte bereits zur Heck-Saugpumpe.

### "„" + "Besonders" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Jeder macht Fehler.“ – „Besonders Vollidioten!“

### "„" + "Bester" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er gewann den Preis in der Rubrik „Bester Nachwuchskünstler“.

### "„" + "Betrachten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Betrachten ja, aber nicht berühren!“ steht am Obststand.

### "„" + "Bienen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bienen geben uns Honig.“ – „Ach, das ist aber sehr nett von denen.“

### "„" + "Billiges" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Billiges Zeug?“

### "„" + "Blaue" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Lied „Blaue Augen" ist von ihr.

### "„" + "Bleistift" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bleistift!“

### "„" + "Blinde" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Blinde Passagiere?“, fragte ich.

### "„" + "Bolashak" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das „Bolashak“-Programm ist sehr nützlich.

### "„" + "Bombastico" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bombastico!“, rief ich und erhob mich, um ihn zu umarmen.

### "„" + "Bring" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bring mich nicht um, lieber Zarewitsch“, sprach der Bär.

### "„" + "Brokeback" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Toms Lieblingsfilm ist „Brokeback Mountain“.

### "„" + "Brut" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie mag es nicht, wenn ich ihre Kinder als ihre „Brut“ tituliere.

### "„" + "Brücke" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Brücke“ ist übertrieben.

### "„" + "Buch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Francis Collins bezeichnet das Genom als das „Buch des Lebens“.

### "„" + "Bugspille" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bugspille bemannt und bereit!“, rief Vickisdotr vom Vorschiff.

### "„" + "Buh" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie „Buh-duh, buh-duh, buh-duh“.

### "„" + "Bushidō" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Samurai lebten nach einem Ehrenkodex, der „Bushidō“ hieß.

### "„" + "Bücher" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Bücher habe ich hier keine“, sagte ich.

### "„" + "Camden Town" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ist die nächste Haltestelle „Camden Town“?

### "„" + "Captain" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Captain“, sagte Saunders, „seht euch die Tischlampe genau an.“

### "„" + "Cheese" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Guckt in die Kamera und sagt „Cheese“!

### "„" + "Chiang" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Chiang, schick diese Nachrichten bitte sofort per Heliograph.“

### "„" + "Chirurg" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Im Gefängnis wurde Tom der „Chirurg“ genannt.

### "„" + "Clair de lune" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Clair de lune“ (Mondschein) wurde von Claude Debussy komponiert.

### "„" + "Clorindas" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Clorindas beide Sägewerke kommen mit den Aufträgen überhaupt nicht hinterher.“

### "„" + "Cosa Nostra" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Mafia wird auch als die „Cosa Nostra“ bezeichnet.

### "„" + "Cowboys" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du nennst sie „Mavericks“; ich nenne sie „Cowboys“.

### "„" + "Dackel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dorgan wusste nicht, wie man „Dackel“ schreibt.

### "„" + "Damit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Damit hatten wir gerechnet“, sagte ich.

### "„" + "Daran" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Daran gibt es nur ein kleines Problem“, sagte Plaitonos Euripideson.

### "„" + "Davon" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Davon habe ich Gerüchte gehört“, sagte Piotr.

### "„" + "Debussy" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Welcher klassische Komponist gefällt dir am besten?“ – „Debussy.“

### "„" + "Deiner" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Deiner war wohl nicht ganz so gut“, antwortete ich.

### "„" + "Denke" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Denke bitte gut darüber nach!“ — „Worüber denn?“

### "„" + "Denkst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Denkst du, was ich denke?“ – „Was denkst du denn?“

### "„" + "Derjenige" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Derjenige, der ihn kauft“, sagte ich.

### "„" + "Diejenige" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Diejenige, der gehorcht werden muss, ruft“, verkündete er.

### "„" + "Dihydrogenmonoxid" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dihydrogenmonoxid“ ist die allzu technische Bezeichnung für Wasser.

### "„" + "Dilemma" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.

### "„" + "Dilemmas" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.

### "„" + "Dilemmata" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Plural von „Dilemma“ ist „Dilemmas“ oder „Dilemmata“.

### "„" + "Domingo" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Domingo“ ist das spanische Wort für „Sonntag“.

### "„" + "Donnerwetter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Da kann ich nur „Donnerwetter!“ sagen.

### "„" + "Dories" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dories halten einiges aus“, sagte Christopher.

### "„" + "Dort" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo ist Tom?“ – „Dort geht er!“

### "„" + "Dreieck" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist schwierig „Dreieck“ zu definieren.

### "„" + "Dreifaltigkeitsplatz" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Dreifaltigkeitsplatz“ war das ursprüngliche Zentrum von Sankt Petersburg.

### "„" + "Dreihunderttausend" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dreihunderttausend Standards sind tatsächlich eine Menge Geld“, sagte ich.

### "„" + "Dumme" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dumme Menschen!“ sprach das Eichhörnchen.

### "„" + "Dundonian" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Rob ist ein stolzer „Dundonian“.

### "„" + "Durchgedreht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Toms Lieblingslied ist „Durchgedreht“ von der Musikgruppe „Die Eichhörnchen“.

### "„" + "Durchs" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie ist sie denn hereingekommen?“ – „Durchs Fenster.“

### "„" + "Echt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich lerne Japanisch.“ – „Echt? Na Donnerwetter!“

### "„" + "Egal" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Deine Mannschaft hat schon wieder verloren!“ – „Egal. Ich trag’s mit Fassung.“

### "„" + "Ehrliche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie wollte ihn „Ehrliche Amandas Universalwarenhaus“ nennen.

### "„" + "Eichkatze" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Marias Lieblingssuperheldin ist die „Eichkatze“.

### "„" + "Eifersüchtig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bist eifersüchtig.“ – „Eifersüchtig? Auf wen denn?“

### "„" + "Eignerin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eignerin?“ fragte Vicki.

### "„" + "Einfach" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einfach aussterben”?

### "„" + "Einkaufen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einkaufen“, sagte Kat.

### "„" + "Einsamkeit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was ist das Gegenteil von „Einsamkeit“?

### "„" + "Einverstanden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Einverstanden“, sagte ich.

### "„" + "Elektrik" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Elektrik?“, fragte ich.

### "„" + "Englisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Welche Fremdsprache hast du genommen?“ – „Englisch.“

### "„" + "Englischunterricht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Buch „Englischunterricht im Ausland“ war für mich nicht von Nutzen.

### "„" + "Entschuldigen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Entschuldigen Sie. Wo sind bitte die Fahrstühle?“ – „Gleich da drüben.“

### "„" + "Erderwärmung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sollte ich „Klimawandel“ oder „Erderwärmung“ sagen?

### "„" + "Erholen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erholen sich unsere Bankiers?“ fragte TinaTinasdotr mich.

### "„" + "Erinnere" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erinnere ich mich daran, Hafengebühren für eine Silver Moon gesehen zu haben?“

### "„" + "Erkennen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erkennen Sie diesen Mann?“ – „Nein.“

### "„" + "Ermordet" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ermordet?“, fragte sie.

### "„" + "Ernsthaft" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Guten Morgen! Es liegt Schnee!“ – „Ernsthaft?“

### "„" + "Ernsthafter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ernsthafter Sport“, schrieb George Orwell, „ist Krieg ohne das Schießen.“

### "„" + "Erwischt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Erwischt“, antwortete Kat.

### "„" + "Erzähl" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tom hat Maria gern.“ — „Erzähl mir etwas, was ich noch nicht weiß!“

### "„" + "Esse" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Esse heute mit Freunden, melde mich dann morgen früh.“

### "„" + "Ethik" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ethik“ bedeutet die Richtlinien des Handelns.

### "„" + "Etwa" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie lange wirst du bleiben?“ – „Etwa zwei Wochen.“

### "„" + "Eule" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie lautet das griechische Wort für „Eule“?

### "„" + "Eure" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Eure Worte sind eine Beleidigung für alle Lieuwhon!“

### "„" + "Fahren" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fahren Sie allein in die Ferien?“ – „Nein, meine Probleme kommen mit.“

### "„" + "Falken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Löwen“ haben gegen die „Falken“ einen leichten Sieg errungen.

### "„" + "Familie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Arbeit“ und „Familie“ waren für sie Synonyme.

### "„" + "Fandst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fandst du seine Musik gut?“ - „Nein, du?“

### "„" + "Fangen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fangen wir mit den Farbstoffen an?“, fragte Kat.

### "„" + "Faulenzen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gibt es eigentlich etwas, was Tom besser kann als Maria?“ – „Faulenzen.“

### "„" + "Feuer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist kein Spaß, in einem vollen Theater „Feuer!“ zu rufen.

### "„" + "Finde" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich finde, Tom sollte das machen.“ – „Finde ich auch.“

### "„" + "Flammenatmung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Flammenatmung“ hat eine lange Geschichte.

### "„" + "Flinke" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Adressieren Sie den Auftrag an „Flinke Abrissbirne in Tuscon, Arizona.“

### "„" + "Forbes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe den Artikel in „Forbes“ gelesen.

### "„" + "Fox News" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf sämtlichen Fernsehergeräten laufen „Fox News“.

### "„" + "France" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Frankreich heißt auf Französisch „France“.

### "„" + "Frank-Walter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie heißt der deutsche Bundespräsident?“ – „Frank-Walter Steinmeier.“

### "„" + "Französisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was lernst du da?“ – „Französisch.“

### "„" + "Freude" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das war's dann mit „Freude, Freude, Freude“.

### "„" + "Freund" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das ist ein hebräisches Wort, und es bedeutet „Freund“.

### "„" + "Frosch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf der Seite stand lediglich das Wort „Frosch“.

### "„" + "Fräulein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fräulein Sheila“, sagte ich zur Signalgeberin.

### "„" + "Furchtbar" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Hund hat keine Nase.“ – „Wie riecht er denn?“ – „Furchtbar.“

### "„" + "Fährst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fährst du mit diesem Bus?“ – „Nein, der ist zu voll.“

### "„" + "Fällt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fällt dir irgendeine Veränderung an mir auf?“ fragte Maria.

### "„" + "Fängers" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir haben zufälligerweise zwei Exemplare des „Fängers im Roggen“.

### "„" + "Fünf" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wieviel schulde ich dir?“ – „Fünf Taler.“

### "„" + "Fürchten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Fürchten Sie sich vor etwas?“ fragte ich.

### "„" + "Gab" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gab es irgendwelche neuen Entwicklungen?“ fragte Holmes.

### "„" + "Gebt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gebt mir fünf Minuten?“

### "„" + "Gefahr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gefahr gehört zu meinem Geschäft“, bemerkte ich.

### "„" + "Gehst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gehst du auf die Party?“ – „Nein, am allerliebsten bliebe ich ganz zu Haus.“

### "„" + "Gehören" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gehören diese Autos Ihnen?“ – „Ja, das sind meine.“

### "„" + "Gelt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gelt,“ sprach er, „das war ein wenig besser?“

### "„" + "Genau" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Genau wie bei den anderen vier.“

### "„" + "Genauso" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Genauso viel wie du“, erwiderte ich.

### "„" + "Genug" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Genug, um den Preis zu rechtfertigen“, sagte Kat.

### "„" + "Gerade" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hat Tom die Prüfung bestanden?“ – „Gerade so.“

### "„" + "Geschwindigkeit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gordon, ein schnelles Wort.“ – „Geschwindigkeit.“

### "„" + "Gestern" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gestern habe ich Tom geküsst.“ – „Warst du da betrunken?“

### "„" + "Gesundheit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gesundheit!“ - „Danke, dir auch.“

### "„" + "Gesunken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gesunken?“

### "„" + "Gleich" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Entschuldigen Sie. Wo sind bitte die Fahrstühle?“ – „Gleich da drüben.“

### "„" + "Glänzende" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lass uns tanzen gehen!“ – „Glänzende Idee!“

### "„" + "Gold" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „Geld“ oder „Gold“ gesagt?

### "„" + "Goldenen Drachen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom hat im „Goldenen Drachen“ gegessen.

### "„" + "Gordon" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gordon, ein schnelles Wort.“ – „Geschwindigkeit.“

### "„" + "Granny" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Granny“, rief ich.

### "„" + "Greift" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Greift zu!“ sagte Tom und eröffnete damit das kalte Büffet.

### "„" + "Griechisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was ist denn das für eine Sprache? Ich verstehe kein Wort.“ – „Griechisch.“

### "„" + "Grießbrei" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was gibt es heute zum Nachtisch?“ – „Grießbrei. Den hat sich Opa gewünscht.“

### "„" + "Großartige" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Großartige Idee!“

### "„" + "Guck" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Guck mal, was ich gerade gekauft habe!“ – „Was ist das denn?“

### "„" + "Gütiger" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gütiger Himmel!“, antwortete Ernie.

### "„" + "Hai" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hai“ heißt auf Lettisch „haizivs“.

### "„" + "Haifisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haizivs“ ist das lettische Wort für „Haifisch“.

### "„" + "Haizivs" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haizivs“ ist das lettische Wort für „Haifisch“.

### "„" + "Halt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Halt!“, rief ich aus voller Kehle.

### "„" + "Hass" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Liebe“ und „Hass“ sind Gegensätze.

### "„" + "Hattest" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hattest du eine gute Überfahrt?“, fragte ich.

### "„" + "Haus" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie schreibt sich „Haus“ auf Arabisch?

### "„" + "Haßt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Haßt du Tom?“ – „Nein, ich finde ihn nur ein bißchen sonderbar.“

### "„" + "He" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „He, guck mal, was ich kann!“ – „Ich versuche zu arbeiten, Tom!“

### "„" + "Hebräisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ist das Aramäisch oder Arabisch?“ – „Hebräisch.“

### "„" + "Heckspille" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heckspille betakelt, bemannt und bereit“, kam die Meldung von unten.

### "„" + "Heed" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heed Orcasyb, wie ich sehe.“

### "„" + "Heil Hitler" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Oder seine Briefe mit „Heil Hitler“ zu unterschreiben.

### "„" + "Heimat" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heimat“ ist ein deutsches Wort ohne englische Entsprechung.

### "„" + "Heinrich" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heinrich, der Wagen bricht.“

### "„" + "Heißest" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heißest du Heinz?“

### "„" + "Helfen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was können wir für Sie tun?“ – „Helfen Sie mir, diesen Koffer zu tragen.“

### "„" + "Her" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Her mit dem Geld!“ – „Nur über meine Leiche!“

### "„" + "Herein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Herein.“ Sparhawk saß am Kapitänstisch, vor sich eine reichliche Mahlzeit.

### "„" + "Herrin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Herrin“, keuchte James Victorson.

### "„" + "Herrschaft" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Oligarchie bedeutet „Herrschaft der Wenigen“.

### "„" + "Herz" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - In der arabischen Sprache wird Antares „Herz des Skorpions“ genannt.

### "„" + "Herzlichen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Herzlichen Dank für alles.“ – „Gern geschehen.“

### "„" + "Hiergeblieben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich komme wohl besser ein andermal wieder.“ – „Hiergeblieben! Setz dich!“

### "„" + "Hilft" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hilft Cola gegen Durchfall?“ – „Wenn die Flasche steckenbleibt, ja.“

### "„" + "Hindenburg" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Hindenburg“ ging plötzlich in Flammen auf.

### "„" + "Hinter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hinter dem Höhenzug“, sagte Naismith.

### "„" + "Hochspannung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Schilder, auf denen „Hochspannung“ steht, sind häufig gelb.

### "„" + "Hoffnung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Gibt es noch Hoffnung?“ – „Hoffnung gibt es immer.“

### "„" + "Holt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Holt mir den Großen Atlas.“

### "„" + "Hotel Tatoeba" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Lieblingslied ist „Hotel Tatoeba“.

### "„" + "How" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was bedeutet „How are you“?

### "„" + "Hundeleben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was bedeutet „Hundeleben“?

### "„" + "Hundert" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hundert, Kapitänin“, beendete Henrison die Zählung.

### "„" + "Hyberbel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hyberbel“ ist ein hochtrabender Ausdruck für „Übertreibung“.

### "„" + "Hättest" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hättest du Lust auf Kino?“ – „Ach, nein, Tom, nicht heute. Ein anderes Mal!“

### "„" + "Hör" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe gesagt „Hör auf!“

### "„" + "Hübsch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ist das Ronnis Rad?“ – „Ja.“ – „Hübsch.“

### "„" + "Hühnchen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom und Maria aßen „Hühnchen Kiew“.

### "„" + "Idiot" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja, wenn wir in Osaka jemanden beschimpfen, sagen wir auch „Idiot“.

### "„" + "Ihn" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Also, hast du es getan?“ – „Was getan?“– „Ihn küssen natürlich!“

### "„" + "Immerhin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Machst du gute Fortschritte?“ – „Es geht nur schleppend voran.“ – „Immerhin!“

### "„" + "Instant" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kaffee?“ – „Instant?“ – „Ja.“ – „Dann nicht.“

### "„" + "Internationale" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Internationale Sprache“ ist sehr nützlich.

### "„" + "Internet" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - IP steht für „Internet-Protokoll“.

### "„" + "Irgendeines" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Welches willst du?“ – „Irgendeines. Das ist mir egal.“

### "„" + "Italienisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was lernst du in der Schule?“ – „Italienisch.“

### "„" + "JUMP" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieser Buchladen hat den Verkauf von „JUMP“ eingestellt.

### "„" + "Jaguar" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sie nannte das Kätzchen „Jaguar“.

### "„" + "Janelle" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Janelle legt deine Reitkleidung bereit.“

### "„" + "Japanisch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich brauche ein Wörterbuch „Japanisch-Englisch“.

### "„" + "Jennifer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Jennifer, kannst du ihre Flagge erkennen?“ rief ich.

### "„" + "Jeruschalajim" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Jerusalem heißt auf Hebräisch „Jeruschalajim“.

### "„" + "Jim" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Jim“, sagte ich, „wie läuft es mit der Räumung des Überlaufs?“

### "„" + "John" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tom.“ „John.“ „Ist mir eine Freude.“ „Ganz meinerseits.“

### "„" + "Juhu" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Als wir den Gipfel erreichten, riefen wir alle „Juhu!“

### "„" + "Juli" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was haben wir jetzt?“ – „Juli.“

### "„" + "Juno" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Raumsonde „Juno“ der NASA ist in eine Bahn um den Jupiter eingetreten.

### "„" + "Kagura" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vom „Kagura-Tanz der Feuergottheit“ höre ich auch zum ersten Mal.

### "„" + "Kalles Speisewagen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Habt ihr schon mal in „Kalles Speisewagen“ diniert?

### "„" + "Kaninchenbau" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Irgendein „Kaninchenbau”-Blödsinn, den du in der Times gelesen hast?

### "„" + "Karten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Karten!“, rief ich.

### "„" + "Kat" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kat hält Wache“, sagte er.

### "„" + "Katze" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „chat“ heißt auf französisch „Katze“.

### "„" + "Keinen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Keinen Penny.“

### "„" + "Keiner" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Keiner wird das je glauben.“ – „Na, Tom hat’s geglaubt.“

### "„" + "Kens" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wessen Hose ist das?“ — „Kens.“

### "„" + "Klein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir nennen diese Stadt „Klein-Kyōto“.

### "„" + "Kleines" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kleines Mädchen?“ Mein Tonfall war vermutlich etwas eisig.

### "„" + "Klimawandel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sollte ich „Klimawandel“ oder „Erderwärmung“ sagen?

### "„" + "Klingt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Klingt für mich nach einem guten Geschäft“, sagte ich.

### "„" + "Kombinieren" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Willst du schwimmen oder segeln gehen?“ – „Kombinieren wir beides miteinander!“

### "„" + "Kommen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kommen viele Raumschiffe hierher?“, fragte der jüngere Mann.

### "„" + "Kopf" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Rosch Haschana bedeutet wörtlich „Kopf des Jahres“.

### "„" + "Kuckuck" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe Tom mit seinem kleinen Kindchen „Kuckuck!“ spielen sehen.

### "„" + "Kunden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ernie“, rief Piotr, „Kunden aus Spencerport.

### "„" + "Kurz" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wann ist Tom angekommen?“ – „Kurz vor Maria.“

### "„" + "König" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Spöttisch „König“ genannt zu werden macht einen noch lange nicht zum König.

### "„" + "Könnt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Könnt ihr mich denn nicht beschützen?“ fragte das Schwesterchen.

### "„" + "Könnte" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Könnte?“, sagte er.

### "„" + "Kümmere" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du bist so schön! Wie heißt du?“ – „Kümmere dich um deinen eigenen Kram!“

### "„" + "La casa de papel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „La casa de papel“ ist die beste spanische Serie.

### "„" + "Laila" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Laila wohnt im Erdgeschoss.“ – „Und Maria?“ – „Im ersten Stock.“

### "„" + "Lamberto" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lamberto“ ist der Name einer großen Hotelkette.

### "„" + "Landungstrupp" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Landungstrupp, antreten.“ Keith hatte ausgewählt, wer mit uns kommen sollte.

### "„" + "Langsam" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Langsam, ich will nicht auf Grund laufen“, antwortete er.

### "„" + "Laus" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Mehrzahl von „Laus“ ist „Läuse“.

### "„" + "Lebt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lebt er mit einer Prinzessin zusammen?“ fragte Gerda.

### "„" + "Lego" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es gibt keine englischen Sätze auf Tatoeba, die das Wort „Lego“ enthalten.

### "„" + "Leicester" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Leicester“ wird „Lester“ ausgesprochen.

### "„" + "Lester" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Leicester“ wird „Lester“ ausgesprochen.

### "„" + "Lieber" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lieber Vater,“ antwortete er, „ich bin ein Drechsler geworden.“

### "„" + "Liebt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Liebt dich Tom?“ – „Ich glaube schon.“

### "„" + "Liegt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo ist Bello?“ – „Liegt der nicht in seinem Körbchen?“

### "„" + "Lietuva" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Litauen heißt auf Litauisch „Lietuva“.

### "„" + "Links" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo sind die Toiletten?“ – „Links.“

### "„" + "Los" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Los!“ rief sie und gab ihm einen sanften Schubs.

### "„" + "Lottoscheine" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Lottoscheine sind verlorenes Geld.“ – „Nicht, wenn man gewinnt!“

### "„" + "Luka" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Luka mag mich nicht.“ – „Aber Jonas mag dich doch.“ – „Der ist mir egal!“

### "„" + "Läuse" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Mehrzahl von „Laus“ ist „Läuse“.

### "„" + "Löwen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Löwen“ haben gegen die „Falken“ einen leichten Sieg errungen.

### "„" + "Mach" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mach ich“, sagte sie.

### "„" + "Mache" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mache ich immer“, antwortete ich.

### "„" + "Machst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Machst du gute Fortschritte?“ – „Es geht nur schleppend voran.“ – „Immerhin!“

### "„" + "Macht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie dumm von mir!“ – „Macht nichts. Passiert ständig.“

### "„" + "Madeira" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Madeira“ ist ein Weinname.

### "„" + "Majdas" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wessen Armbänder sind das?“ — „Majdas.“

### "„" + "Manchmal" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Manchmal denke ich über Scheidung von ihm nach.“ – „Du machst wohl Witze!“

### "„" + "Mann" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mann über Bord!“

### "„" + "Marias" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Marias Käsekuchen schmeckte scheußlich.“ – „Tom fand ihn köstlich.“

### "„" + "Marika" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Marika und ich sind beste Freundinnen.“

### "„" + "Marsch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Marsch in die Städte“ dauerte viel länger, als ich gedacht hätte.

### "„" + "Mary" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Satz enthält zwei Subjekte, nämlich „Tom“ und „Mary“.

### "„" + "Matrix" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie oft hast du „Matrix“ gesehen?

### "„" + "Matrose" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Matrose Jacobson, kennen Sie diesen Mann?“, fragte ich.

### "„" + "Matrosin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Matrosin Mildredsdotr, kennen Sie diesen Mann?“

### "„" + "Mau" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mau“, sagte die Katze und sah mich an.

### "„" + "Mavericks" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Du nennst sie „Mavericks“; ich nenne sie „Cowboys“.

### "„" + "Ma’am" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ma’am.“

### "„" + "Meer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieses Wort bedeutet „Meer“.

### "„" + "Mehr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hast du verstanden?“ — „Mehr oder weniger.“

### "„" + "Meiner" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mein Vater trinkt keinen Alkohol.“ – „Meiner auch nicht.“

### "„" + "Meister" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Freund hat mich als „Meister der durchgemachten Nächte“ bezeichnet.

### "„" + "Meisterin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Freundin hat mich als „Meisterin der durchgemachten Nächte“ bezeichnet.

### "„" + "Meldungen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Meldungen!“, rief ich und versuchte, meine Stimme nicht überschlagen zu lassen.

### "„" + "Menschenfresser" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Anwohner nennen diesen Fluss „Menschenfresser“ und fürchten ihn.

### "„" + "Merkur" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die ersten Raumanzüge der NASA wurden für das Programm „Merkur“ entwickelt.

### "„" + "Mich" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wen kümmert’s?“ – „Mich!“

### "„" + "Michael" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Michael“ ist ein männlicher, aber „Michaela“ ein weiblicher Vorname.

### "„" + "Michaela" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Michael“ ist ein männlicher, aber „Michaela“ ein weiblicher Vorname.

### "„" + "Mindestens" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mindestens einen halben Zentner“, antwortete er.

### "„" + "Mindy" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mindy?“, erklang eine Stimme.

### "„" + "Mistral" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Wind, der häufig in der Provence weht, heißt „Mistral“.

### "„" + "Mittagessen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Mittagessen ist immer gut“, erwiderte Vicki.

### "„" + "Moment" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Moment“, sagte David.

### "„" + "Muiriel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Passwort lautet „Muiriel“.

### "„" + "Méjico" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Den Mexikanern gefällt es nicht, wenn man „Méjico“ anstatt „México“ schreibt.

### "„" + "México" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Den Mexikanern gefällt es nicht, wenn man „Méjico“ anstatt „México“ schreibt.

### "„" + "Mörderabsätzen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Manchmal spricht man bei arg hochhackigen Frauenschuhen von „Mörderabsätzen“.

### "„" + "Müssten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Müssten wir also ein paar Tage warten?“, fragte Dedicata.

### "„" + "Nachrichten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nachrichten“, antwortete ich.

### "„" + "Nachti" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „So, ich hau mich hin. Bis morgen!“ – „Nachti, Tom!“

### "„" + "Nanu" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nanu, wo willst du denn so plötzlich hin?“ – „Auf die Toilette.“

### "„" + "Nationalen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom ist ein Mitglied der „Nationalen Schusswaffenvereinigung“.

### "„" + "Ne" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Kommst du?“ – „Ne, laß mal! Ich bleibe hier.“ – „Wie du willst.“

### "„" + "Neue" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Neue Bücher“?

### "„" + "Niemand" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wer hat Ihnen geholfen?“ – „Niemand. Ich habe alles selbst gemacht.“

### "„" + "Nihon" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Japan wird auf Japanisch auch „Nihon“ genannt.

### "„" + "Nimmst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nimmst du diesen Bus?“ – „Nein, ich warte auf den nächsten.“

### "„" + "Null" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo ist Tom?“ – „Null Ahnung.“

### "„" + "Ob" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Heute oder morgen?“ – „Ob heute oder morgen, das ist mir ganz egal.“

### "„" + "Ode" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Beethoovens „Ode an die Freude“ ist die offizielle Hymne der Europäischen Union.

### "„" + "Okay" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tom, setz dich.“ – „Okay.“

### "„" + "Open Table" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom hat sich über „Open Table“ einen Tisch in dem Restaurant reservieren lassen.

### "„" + "Palast" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wo ist der „Palast der schönen Künste“?

### "„" + "Panzerkreuzer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Panzerkreuzer Potjomkin“ ist Sergei Eisensteins großartigster Film.

### "„" + "Paraguyaer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was für Landsleute waren unsere Nachbarn?“ – „Paraguyaer.“

### "„" + "Pass" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pass auf dich auf“, sagte er.

### "„" + "Perlen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schöne Kleider“ sagte die eine, „Perlen und Edelsteine“ die zweite.

### "„" + "Pfirsichbaum" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pfirsichbaum“ heißt auf Altenglisch „persoctrēow“.

### "„" + "Philosophie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Philosophie“ ist ein griechisches Wort, das „Liebe zur Weisheit“ bedeutet.

### "„" + "Photoshop" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom kann recht gut mit „Photoshop“ umgehen.

### "„" + "Pieps" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pieps!“ sagte die Maus.

### "„" + "Pinterest" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maria hat bei „Pinterest“ ein Bild von ihren Keksen hochgeladen.

### "„" + "Pochi" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mike nennt seinen Hund „Pochi“?

### "„" + "Politik" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er liest nur die Rubriken „Wirtschaft“ und „Politik“.

### "„" + "Quak" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Quak, quak!“ sagte die Ente.

### "„" + "Queen Elizabeth" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich taufe dieses Schiff auf den Namen „Queen Elizabeth“.

### "„" + "Querdenker" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Toms Rufzeichen lautet „Querdenker“.

### "„" + "Rap" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Frauen werden in vielen Liedern des Genres „Rap“ herabgewürdigt.

### "„" + "Rauchen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Im Restaurant hängt ein Schild, auf dem „Rauchen verboten!“ steht.

### "„" + "Recnet" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Recnet“, sagte Vincent.

### "„" + "Regeln" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lies bitte unsere „Regeln und Richtlinien“.

### "„" + "Renaissance" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was bedeutet „Renaissance“?

### "„" + "Reparierst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Reparierst du es nicht einfach an deinen freien Tagen?“, fragte Piotr.

### "„" + "Rory" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rory“, sagte ich, „das sollte für dein Abendessen reichen.“

### "„" + "Rosehip Way" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rosehip Way?“

### "„" + "Rosetta" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rosetta“ wurde von der Europäischen Weltraumorganisation gebaut.

### "„" + "Rossija" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Russland heißt auf Russisch „Rossija“.

### "„" + "Rote" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Rote Drache“ ist eine allseits bekannte Glücksspiel- und Opiumhöhle.

### "„" + "Rovers" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bist du für „City“ oder die „Rovers“?

### "„" + "Ruder" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ruder mittschiffs!“, rief ich.

### "„" + "Ruhe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ruhe!“, rief ich.

### "„" + "Räume" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Räume bitte dein Zimmer auf!“ „Wieso denn, es ist doch eh ganz sauber!“

### "„" + "Rückwärts" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Rückwärts an den Kai?“ fragte er.

### "„" + "Sachbücher" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sachbücher?“

### "„" + "Sage" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo ist Tom?“ – „Sage ich nicht!“ – „Ach, bitte!“

### "„" + "Satellit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Welche verschiedenen Bedeutungen können Sie für das Wort „Satellit“ finden?

### "„" + "Schaffen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schaffen Sie es, alle Bücher zu verkaufen, die Sie finden?“, fragte ich.

### "„" + "Schauen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schauen Sie!“

### "„" + "Schickt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schickt er ein Schiff nach Herculaneum und kauft dort überzählige Exemplare?“

### "„" + "Schießpulver" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der häufigste davon ist „Schießpulver“.

### "„" + "Schießt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Dürfen wir dir einige Fragen stellen?“ – „Schießt los!“

### "„" + "Schiff" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schiff“, verbesserte Sparhawk sie.

### "„" + "Schlag" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was machen wir denn jetzt mit unserem angebrochenen Abend?“ – „Schlag was vor!“

### "„" + "Schlampe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schlampe!“

### "„" + "Schneit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schneit es hier oft?“ – „Fast jeden Tag.“

### "„" + "Schwer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wer sagt denn jetzt die Wahrheit, Tom oder Maria?“ – „Schwer zu sagen.“

### "„" + "Schwuchtel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sagt weiter „Schwuchtel“, ja?

### "„" + "Schwul" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schwul“?

### "„" + "Schön" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du siehst toll aus. Hast du abgenommen?“ – „Schön wär’s!“

### "„" + "Schönen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schönen Dank, Wolf.“

### "„" + "Schönes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Schönes Schiff“, sagte er.

### "„" + "Science" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Studie wurde im Journal „Science“ veröffentlicht.

### "„" + "Sechzehn" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie alt bist du?“ „Sechzehn.“

### "„" + "Sehr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie geht es dir?“ – „Sehr gut.“

### "„" + "Seht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Seht her!“, sagte David.

### "„" + "Seid" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Seid ihr denn nicht verwundet?“ fragten die Reiter.

### "„" + "Seidenbettwäsche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Seidenbettwäsche?“, fragte sein Assistent.

### "„" + "Seien" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Seien Sie nicht so ein Dummkopf, Professor!“ rief ich aus.

### "„" + "Seine" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Seine Zehen riechen gar deliziös“, dachte der Hund.

### "„" + "Seit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie lange ist deine Mutter schon krank?“ – „Seit etwa zwei Wochen.“

### "„" + "Selbsterhaltung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kannst du mit dem Begriff „Selbsterhaltung“ etwas anfangen?

### "„" + "Senat" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Begriffe „Senat“ und „Zoll“ waren mir beide neu.

### "„" + "Setz" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Setz dich“, sagte Keith.

### "„" + "Setzen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Setzen Sie sich und probieren Sie eine dieser Zigarren“, sagte er.

### "„" + "Sieben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sieben Millionen!“, rief jemand aus dem Saal.

### "„" + "Sieht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sieht wirklich gut aus“, sagte ich.

### "„" + "Silizium" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Am „Silizium-Kreisel“ sind zahlreiche junge IT-Unternehmen ansässig.

### "„" + "Silver" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Silver Moon?“, fragte Clark.

### "„" + "Skype" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom und Maria sprechen häufig über „Skype“ miteinander.

### "„" + "Sneewittchen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sneewittchen soll sterben,“ rief sie, „und wenn es mein eignes Leben kostet.“

### "„" + "Soccer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Soccer“ nennen die Europäer „football“.

### "„" + "Sofort" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sofort“, stimmte ich zu.

### "„" + "Sonntag" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Domingo“ ist das spanische Wort für „Sonntag“.

### "„" + "Spam" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das einzige, was ich einen Hund je verschmähen sah, ist „Spam“.

### "„" + "Spencerport" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Spencerport“ stand eindeutig nicht auf der Liste.

### "„" + "Spieglein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Spieglein, Spieglein an der Wand, wer ist die schönste im ganzen Land?“

### "„" + "Spielst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Spielst du Fußball?“ – „Nein, aber ich spiele Tennis.“

### "„" + "Sport" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich lese alle Rubriken außer die Rubrik „Sport“.

### "„" + "Sprechen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sprechen Sie Deutsch?“ – „Nein.“

### "„" + "Spricht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Spricht er hin und wieder von mir?“ – „Nö.“

### "„" + "Springe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Springe über den Graben!“ – „Ich springe, wenn du springst!“

### "„" + "Spüren" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Spüren Sie etwas?“ – „Gar nichts.“

### "„" + "Statt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Statt mit richtigem Geld?“

### "„" + "Stefan" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was ist denn das für ein Geräusch?“ – „Stefan übt seine Tanzschritte.“

### "„" + "Steh" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Steh auf!“ – „Ich bin auf!“

### "„" + "Steigt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Steigt denn vom Pferd und zieht!“ forderte Bucklaw.

### "„" + "Stelldichein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stelldichein mit dem Tod“ ist ein Krimi von Agatha Christie.

### "„" + "Sternenschiffer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „Astronaut“ kommt aus dem Lateinischen und bedeutet „Sternenschiffer“.

### "„" + "Sternfahrer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Sternfahrer tun den Menschen manchmal schreckliche Dinge an“, sagte sie.

### "„" + "Sternschnuppen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Leute sprechen bei Meteoren oft von „Sternschnuppen“.

### "„" + "Steuerbordleinen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Steuerbordleinen straffen!“, rief ich.

### "„" + "Stimme" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Stimme Amerikas“ sendet aus Washington.

### "„" + "Stimmten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stimmten diese Aussagen?“, wiederholte er.

### "„" + "Stinklangweilig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie findest du den Film?“ – „Stinklangweilig.“

### "„" + "Stolperte" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stolperte herum“ trifft es genau; ich fand die Frage nicht.

### "„" + "Stärke" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Stärke“ bezieht sich nicht nur auf unseren Körper.

### "„" + "Störe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Störe ich?“ – „Du nie.“

### "„" + "Suchst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Suchst du etwas?“ „Nein, ich schaue nur.“

### "„" + "Suomi" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Finnland heißt auf Finnisch „Suomi“.

### "„" + "Super" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ihr macht ein Spiel? Kann ich mitmachen?“ – „Klar.“ – „Super!“

### "„" + "Süßes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom ist zu alt für „Süßes oder Saures!“

### "„" + "Tag" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Einen schönen „Tag des Satzes“!

### "„" + "Tanias" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wessen Ohrringe sind das?“ — „Tanias.“

### "„" + "Tee" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tee?“ – „Ja, gerne. Danke!“

### "„" + "Tempo" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tempo drosseln!“, rief ich schließlich.

### "„" + "The Slits" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ari Up (Ariane Forster) war die Frontfrau von „The Slits“.

### "„" + "Times" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich lese die „Times“.

### "„" + "Tinder" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Viele sind bei „Tinder“ nur auf eine kurze Beziehung aus.

### "„" + "Tippen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich wusste, dass man das „Tippen“ nannte, hatte ihn aber noch nie dabei gesehen.

### "„" + "Tischlein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Weltflucht heißt gewöhnlich nicht „Tischlein, deck dich“.

### "„" + "Tod" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das ist nicht „Tod eines Handlungsreisenden”.

### "„" + "Toilette" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - An der Tür war ein großes T für „Toilette“.

### "„" + "Traust" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Darf ich es mal versuchen?“ – „Traust du dir das denn zu?“

### "„" + "Trikolore" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „Trikolore“ ist nicht die einzige Trikolorenflagge auf der Welt.

### "„" + "Trinkst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Trinkst du deinen Tee mit Milch oder Zucker?“ – „Nein, mit nichts von beidem!“

### "„" + "Trinkt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Trinkt Tom noch?“ – „Nein, er ist seit zwei Jahren trocken.“

### "„" + "Trumpete" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Clinton kann Saxophon spielen und Trump die „Trumpete“.

### "„" + "Tsubasa" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der „Tsubasa“ ist ein sehr schneller Zug.

### "„" + "Tu" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Tu’s nicht, Anne!“ flehte Diana. „Du fällst herunter und stürzt dich zu Tode!“

### "„" + "Twitter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Man kann dem Papst jetzt auf „Twitter“ folgen.

### "„" + "UdSSR" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „UdSSR“ steht für „Union der Sozialistischen Sowjetrepubliken“.

### "„" + "Ungefähr" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie lange werden die Renovierungsarbeiten dauern?“ – „Ungefähr zwei Wochen.“

### "„" + "Union" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „UdSSR“ steht für „Union der Sozialistischen Sowjetrepubliken“.

### "„" + "Unmöglich" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Freund schüttelte den Kopf, als wollte er „Unmöglich!“ sagen.

### "„" + "Unser" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weißt du, was heute für ein Tag ist?“ — „Unser Hochzeitstag!“

### "„" + "Unter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Unter der Brücke“, antwortete sie.

### "„" + "Unternehmensengel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maria betätigt sich als „Unternehmensengel“.

### "„" + "Venedig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Stadt Recife ist als „Venedig Brasiliens“ bekannt.

### "„" + "Verdoppeln" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verdoppeln Sie die Wetterwache“, befahl ich.

### "„" + "Vergiss" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vergiss, dass ich das gesagt habe.“

### "„" + "Vergiß" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was willst du?“ – „Vergiß es!“

### "„" + "Verlass" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verlass mich nicht!“ – „Es ist zu spät, Tom!“

### "„" + "Verlasse" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verlasse mich bitte nicht!“ – „Es ist zu spät, Maria!“

### "„" + "Vernünftig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vernünftig!“, sagte Jill.

### "„" + "Verschwinde" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn jemand „Verschwinde!“ zu dir sagt, was tust du dann?

### "„" + "Versprochen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Versprochen?“ – „Ja, versprochen!“

### "„" + "Verstanden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Verstanden“, antwortete ich.

### "„" + "Verstecken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir haben nur „Verstecken“ gespielt.

### "„" + "Versteht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Versteht ihr, welche Möglichkeiten ihr habt?“, fragte Keith.

### "„" + "Vertrau" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vertrau mir, Hans!“ rief sie.

### "„" + "Vertraut" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vertraut ihr mir?“ – „Nein.“

### "„" + "Vicki" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vicki, bring uns flussaufwärts, sofern es nicht zu flach ist“, antwortete ich.

### "„" + "Vieles" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vieles ist irgendwie mit ihm verflochten“, erwiderte David.

### "„" + "Vielfältigkeiten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das sind zu viele „Vielfältigkeiten“, Chris.

### "„" + "Vier" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie viele Finger halte ich hoch, Winston?“ – „Vier.“

### "„" + "Viertausend" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wieviel würde das kosten?“ – „Viertausend Dinar.“

### "„" + "Vom" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du schon einmal „Vom Winde verweht“ gelesen?

### "„" + "Vor" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich habe aufgehört zu rauchen.“ － „Wann?“ － „Vor zwei Tagen.“

### "„" + "Vorgestern" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wann bist du denn wiedergekommen?“ – „Vorgestern.“

### "„" + "Vorher" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Vorher“?!

### "„" + "Vorsicht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf dem Karton ist ein Aufkleber mit der Aufschrift „Vorsicht! Zerbrechlich!“

### "„" + "Voyage" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Gérard de Nerval schrieb „Voyage en Orient“ (Reise im Orient).

### "„" + "Wahlkampfspenden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was in aller Welt – nein, nicht auf unserer Welt – waren „Wahlkampfspenden“?

### "„" + "Wahrhaftig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wahrhaftig, Watson, Sie übertreffen sich selbst“, sagte Holmes.

### "„" + "Wahrheit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wahrheit klinget von deinem Munde.“ – „Warum redest du so, Tom?“

### "„" + "Warnung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - An jedem Zaun in meiner Nachbarschaft hängt ein Schild „Warnung vor dem Hunde!“

### "„" + "Wegen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Warum ist alles weiß?“ – „Wegen des Schnees.“

### "„" + "Weh" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weh!“ sagte er. „Was soll aus mir werden?“

### "„" + "Weine" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weine nicht“, sagte sie.

### "„" + "Weit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo ist denn meine Brille?“ – „Weit weg kann sie nicht sein.“

### "„" + "Weiter" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weiter nichts als einen Esel.“

### "„" + "Weiterfeuern" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weiterfeuern“, befahl ich den Schiffsarmbrustmannschaften.

### "„" + "Wen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wen kümmert’s?“ – „Mich!“

### "„" + "Werden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Werden die Liberalen die Wahlen gewinnen?“ – „Das ist nicht zu erwarten.“

### "„" + "Weswegen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Weswegen bist du denn böse?“ – „Ich bin doch nicht böse. Ich habe zu tun.“

### "„" + "Whisky" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Whisky?“ – „Nein, danke. Ich muß bei klarem Verstand bleiben.“

### "„" + "Wildtiere" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wildtiere sind keine Roboter“, sagt sie.

### "„" + "Wirt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wirt Chiang, Sie haben Ihre Sache absolut perfekt gemacht.“

### "„" + "Wirtschaft" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er liest nur die Rubriken „Wirtschaft“ und „Politik“.

### "„" + "Wissen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Habe ich das wirklich getan?“ – „Wissen Sie das denn nicht mehr?“

### "„" + "Wohl" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wohl gesprochen“, brachte er hervor.

### "„" + "Wohnst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wohnst du bei Tom?“ – „Er ist es, der bei mir wohnt.“

### "„" + "Wolltest" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wolltest du das sagen?“ – „Ja, du hast es auf den Punkt gebracht.“

### "„" + "Womit" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ich war beschäftigt.“ – „Womit denn?“

### "„" + "Worauf" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Worauf ruht dein Blick?“ — „Auf gar nichts.“

### "„" + "Wort" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Beseda“ bedeutet auf Slowenisch „Wort“.

### "„" + "Würden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Würden Sie mir bitte den Zucker geben?” „Hier, bitte.”

### "„" + "Yeoman" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Yeoman, tragen Sie meinen Wachwechsel und die Uhrzeit ins Logbuch ein.“

### "„" + "Zahlen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zahlen Sie zusammen oder getrennt?“ – „Zusammen bitte!“

### "„" + "Zehn" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wann fährt der Zug?“ – „Zehn vor.“

### "„" + "Zehnundvierzig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie alt bist du?“ - „Zehnundvierzig.“

### "„" + "Zoll" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die Begriffe „Senat“ und „Zoll“ waren mir beide neu.

### "„" + "Zuerst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zuerst bis hundert zählen, Herr Henrison.“

### "„" + "Zur" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zur Großmutter.“

### "„" + "Zurück" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zurück dauert es viel länger.“

### "„" + "Zusammen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zahlen Sie zusammen oder getrennt?“ – „Zusammen bitte!“

### "„" + "Zusätzliche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zusätzliche Männer an der Gangway?“, fragte ich.

### "„" + "Zweihundert" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wie viel hast du locker gemacht?“ – „Zweihundert Euro.“

### "„" + "Zwillinge" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Zwillinge“, sagte Clorinda.

### "„" + "abschicken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe vergessen, auf „abschicken“ zu klicken.

### "„" + "also" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir hatten schon andere Besucher“, sagte ich, „also verstehe ich eure Fragen.

### "„" + "an" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich weiß nicht, ob dieser Lichtschalter auf „an“ oder „aus“ steht.

### "„" + "aufwachen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Gegenteil von „aufwachen“ ist „einschlafen“.

### "„" + "aus" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich weiß nicht, ob dieser Lichtschalter auf „an“ oder „aus“ steht.

### "„" + "ausreichend" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Er bekam ein „ausreichend“.

### "„" + "bitte" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sagen Sie „bitte“ .

### "„" + "bleiben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe für „bleiben“ gestimmt.

### "„" + "bunt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Motto für den Antiextremismus-Tag war „bunt statt braun“ .

### "„" + "chat" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „chat“ heißt auf französisch „Katze“.

### "„" + "denkt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn KI „denkt“ oder sich dem Denken annähert, wer sind wir dann?

### "„" + "der" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieses Eichhörnchen soll fortan den Namen „der Knuffige“ tragen.

### "„" + "dicken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir bezeichnen New York als den „dicken Apfel“.

### "„" + "dreizehn" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „dreizehn“ oder „dreißig“ gesagt?

### "„" + "dreißig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „dreizehn“ oder „dreißig“ gesagt?

### "„" + "eBay" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Denken Sie ernsthaft darüber nach, das auf „eBay“ zu verkaufen?

### "„" + "ein" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ein bisschen“ und „ein kleines bisschen“ – das ist fast das gleiche.

### "„" + "einschlafen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Gegenteil von „aufwachen“ ist „einschlafen“.

### "„" + "entdecken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Verwende nicht „entdecken“, wenn du eigentlich „erfinden“ meinst.

### "„" + "er" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Amanda“, sagte Jerry, „er ist vom Topp des Großmasts gesprungen.

### "„" + "es" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ach nein,“ antwortete sie, „es ist kein Riese, sondern ein garstiger Frosch.“

### "„" + "falschen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Beim Übersetzen muss man vor „falschen Freunden“ auf der Hut sein!

### "„" + "fettes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe Tom als „fettes Schwein“ tituliert.

### "„" + "football" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Soccer“ nennen die Europäer „football“.

### "„" + "fröhliche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist schon wieder „fröhliche Stunde“.

### "„" + "gefallen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Lass dir meine Seite auf Facebook doch bitte „gefallen“!

### "„" + "gehen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ein grünes Licht bedeutet „gehen“.

### "„" + "habe" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Außerdem“, fügte ich hinzu, „habe ich etwas Geld zur Verfügung.

### "„" + "haber" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie lautet die Befehlsform von „haber“?

### "„" + "habt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Das ist auch nicht die rechte,“ sprach er, „habt ihr keine andere Tochter?“

### "„" + "haizivs" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hai“ heißt auf Lettisch „haizivs“.

### "„" + "hat" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Nun,“ sagte der alte Schneider, „hat die Ziege ihr gehöriges Futter?“

### "„" + "helfen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nach „lieben“ ist „helfen“ das schönste Zeitwort der Welt.

### "„" + "hier" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mit „hier“ meinte ich Australien.

### "„" + "hopfenlosen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Boldi befand sich in einer „hopfenlosen Situation“.

### "„" + "kaisha" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie sagt man „kaisha“ auf Englisch?

### "„" + "kranken" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Man nennt es „kranken Humor“.

### "„" + "kurz" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der dritte zu kurz, „kurz und dick hat kein Geschick.“

### "„" + "könnt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Schwesterchen weinte und sagte „könnt ihr denn nicht erlöst werden?“

### "„" + "lang" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der andere zu lang, „lang und schwank hat keinen Gang.“

### "„" + "lass" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Margrethe“, sagte ich, „lass mir ein großes Stück Kuchen und ein Glas Wein da.

### "„" + "lasst" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Als es der König hörte, sprach er „lasst ihn herauf kommen.“

### "„" + "leben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom folgte immer der Devise „leben und leben lassen“.

### "„" + "leiderlicherweise" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Leider gibt es das Wort „leiderlicherweise“ nicht.

### "„" + "lieben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nach „lieben“ ist „helfen“ das schönste Zeitwort der Welt.

### "„" + "manchmal" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Was meinst du mit „manchmal“?

### "„" + "meine" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mir auch recht,“ antwortete die Bäurin, „meine Äpfel will ich schon los werden.

### "„" + "morgen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Da sprach der König „morgen soll noch einmal gejagt werden.“

### "„" + "männliche" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es gibt keine „männliche“ Art zu lernen.

### "„" + "n" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Aus irgendeinem Grund habe ich „m“ gelesen und nicht „n“.

### "„" + "nichts" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wie kamen wir mit „nichts mehr auf Lager“ zurecht?

### "„" + "nokedli" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Kilonis isst gern „nokedli“, ein ungarisches Nudelgericht.

### "„" + "nun" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Gretel weinte bittere Tränen und sprach zu Hänsel „nun ists um uns geschehen.“

### "„" + "oh" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Gebrauch von „oh“ im Englischen ist ziemlich kompliziert.

### "„" + "otoshidama" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Es ist das, was wir ein „otoshidama“ nennen.

### "„" + "ovale" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das „ovale Büro“ befindet sich im Weißen Haus.

### "„" + "persoctrēow" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Pfirsichbaum“ heißt auf Altenglisch „persoctrēow“.

### "„" + "r" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tom spricht den Buchstaben „r“ falsch aus.

### "„" + "rausschmeißt" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn der Chef einen „rausschmeißt“, heißt das, dass man entlassen ist.

### "„" + "rosa" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Genitiv Singular von „rosa“ lautet „rosae“.

### "„" + "rosae" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Genitiv Singular von „rosa“ lautet „rosae“.

### "„" + "schlaf" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Du“, sagte ich und gab ihm einen kleinen Stoß mit dem Knie, „schlaf nicht ein!“

### "„" + "schuldig" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mein Klient will auf „schuldig“ plädieren.

### "„" + "schwul" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zwischen „schwül“ und „schwul“ besteht nur ein kleiner Unterschied.

### "„" + "schwül" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Zwischen „schwül“ und „schwul“ besteht nur ein kleiner Unterschied.

### "„" + "seht" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Captain“, sagte Saunders, „seht euch die Tischlampe genau an.“

### "„" + "sieben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Die „sieben“ ließen natürlich James und Janet außen vor.

### "„" + "soap" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „soup“ oder „soap“ gesagt?

### "„" + "solange" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ja“, sagte ich, „solange derjenige, der uns dreizehn schuldet, auch zahlt.

### "„" + "sollte" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was“, fragte er, „sollte man mit jemandem machen, der alle hintergangen hat?“

### "„" + "sollten" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wenn wir es noch einmal machen“, sagte Kat, „sollten wir es besser angehen.

### "„" + "soup" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du „soup“ oder „soap“ gesagt?

### "„" + "strawberries" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Warum heißen Erdbeeren im Englischen „strawberries“?

### "„" + "umstrittener" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alles kann und sollte übersetzt werden, einschließlich „umstrittener“ Sätze.

### "„" + "variabel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - In der heutigen Zeit ist „variabel“ ein Schimpfwort.

### "„" + "von" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Ja,“ sagte Sneewittchen, „von Herzen gern,“ und blieb bei ihnen.

### "„" + "vorläufige" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Drei Tage später erhielt Suellen die erste „vorläufige Klageankündigung“.

### "„" + "vorvorvorletztes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dieses Satzes vorvorvorletztes Wort ist „vorvorvorletztes“.

### "„" + "warum" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was soll das?“ rief der andere, „warum wirfst du mich?“

### "„" + "weise" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das nenne ich „weise Voraussicht“.

### "„" + "werden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hier also,“ fuhr sie fort, „werden die Belege aufbewahrt.“

### "„" + "wuff" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Der Hund macht „wuff-wuff”.

### "„" + "wurden" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Außerdem“, sagte ich, „wurden sie verfolgt.

### "„" + "zerbrechlich" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Auf der Schachtel ist ein Aufkleber mit der Aufschrift „zerbrechlich“.

### "„" + "zurücksetzen" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hast du auf „zurücksetzen“ gedrückt?

### "„" + "Üben" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Was tun Sondereinsatzkräfte sonst noch außer Üben und Kämpfen?“ — „Üben.“

### "„" + "Überall" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Wo tut’s dir denn weh?“ – „Überall.“

### "„" + "Übertreibung" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - „Hyberbel“ ist ein hochtrabender Ausdruck für „Übertreibung“.

### "„" + "Şiş" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Maria hat ihren Neffen „Şiş kebap“ gemacht.

### "„" + "злой" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Das Wort „злой“ ist das einzige einsilbige russische Eigenschaftswort.

### "‟" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Wo ist mein Klassenzimmer?‟ - „In der dritten Etage.‟

### "‟" + "„" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - „Hast du gegessen?‟ „Ja.‟

### "…" + "alles" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Wir müssen es überweisen …alles wird gut, Fräulein.

### "…" + "durch" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - WIE GEHT ES IHR …durch die einfache Luftbewegung.

### "‹«" + "Im" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - ‹« Im Raum herrschte völlige Stille.

### "‹“" + "Nachdenklich" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - ‹“ Nachdenklich betrachtete er die Einladung.

### "›" + "Schiffsmeisterin" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - ›Schiffsmeisterin‹?

### "›" + "im" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Crescent Dock war ›im Bau‹, aber jahrelang war dort nichts geschehen.

### "›Angebot" + "‹" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ich habe genug Gemurmel gehört, um leicht zu erraten, was ihr ›Angebot‹ ist.

