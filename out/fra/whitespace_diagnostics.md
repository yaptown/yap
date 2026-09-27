# Whitespace Prediction Diagnostics

Total predictions: 4675772
Total errors: 643
Accuracy: 99.99%

## Error Patterns (sorted by frequency)

### "'" + "?" (27 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - Alors, tu disais 'meilleures intentions' ?
  - C'est donc le célèbre 'Livre des Frères' ?
  - C'est donc le fameux 'Livre des Frères' ?

### "," + "p'" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors, p'tit gars ?
  - Fais pas le malin, p'tit con.
  - Hein, p'tit ?

### "," + "'" (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est qui, 'on' ?
  - Comment ça, 'encore' ?
  - Comment ça, 'nous' ?

### "," + "«" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ah, « tu ne verras pas d'ici » ?
  - C'est moi, « L'Élu ».
  - C'est qui, « moi » ?

### "--" + "!" (6 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - H-Hey-- !
  - Où est-ce que-- ! ?
  - Quoii-- ! ?

### "a" + "ida" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Elle l'aida elle-même car personne d'autre ne voulait le faire.
  - Elle l'aida à nouer sa cravate car il ignorait comment faire.
  - Il m'aida à porter le sac.

### "presser" + ";" (5 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tu n'aurais pas dû te presser ; tu es arrivé trop tôt.
  - Tu n'aurais pas dû te presser ; tu es arrivée trop tôt.
  - Vous n'auriez pas dû vous presser ; vous êtes arrivée trop tôt.

### "," + "c’" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mener la mission, c’est notre devoir.
  - Tanjiro, si tu peux encore bouger, c’est le moment !
  - Tous pour un, un pour tous, c’est notre devise.

### "--" + "Je" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Donc, de toute façon, je-- Je ne sais pas.
  - Il a dit que la philosophie-- Je frime, donc arrête-moi.
  - J'étais marié avec-- Je peux m'asseoir là ?

### "nous" + "'" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est qui 'nous' ?
  - Comment ça, 'nous' ?
  - Qu'est-ce que tu veux dire par 'nous' ?

### "pas" + ";" (4 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ceux qui savent ne parlent pas ; ceux qui parlent ne savent pas.
  - Ne le leur explique pas ; tu perds ton temps.
  - Ne le leur expliquez pas ; vous perdez votre temps.

### "'" + "en" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est 'île' en espagnol mais à l'envers.
  - Comment dit-on 'ami' en elfe ?
  - Comment dit-on 'planque' en français ?

### "'" + "à" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Place un 'X' à l'endroit opportun.
  - Si on donnait une touche 'irlandaise' à ces cafés ?
  - T'aurais pas un 'R' à me prêter ?

### "J" + "‘" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J‘ai fêté mon anniversaire dans un restaurant.
  - J‘aime celui-ci.
  - J‘aime les soupes avec de nombreux végétaux.

### "can" + "'t" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - But I can't spend my life at the laundromat.
  - The holiday feast, we can't miss that.
  - The holiday feast, you can't miss that.

### "d" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce tableau est considéré comme un chef-d'œuvre.
  - Je me casse d'ici.
  - Je veux que tu sortes d'ici.

### "il" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce n'est pas : 'Où est-il' ?
  - Qui 'il' ?
  - Qui ça, 'il' ?

### "libre" + ";" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Linux est un système d'exploitation libre ; tu devrais l'essayer.
  - Linux est un système d'exploitation libre ; vous devriez l'essayer.
  - Personne n'est libre ; même les oiseaux sont enchaînés au ciel.

### "on" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est qui, 'on' ?
  - Comment ça, 'on' ?
  - Qui ça, 'on' ?

### "toi" + ";" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Calme-toi ; il ne fait que te taquiner.
  - Je ne priais pas contre toi ; je priais pour toi.
  - Je ne te parle pas à toi ; je parle au singe.

### "…" + "de" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - De quoi…de champignons.
  - Et ce Borovskikh……de l'industrie locale ?
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### "'" + "au" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je croyais qu'on avait choisi 'M' au hasard.
  - Je fais un 'chip' au-dessus de l'eau.

### "'" + "et" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai fait 'rappel' et c'était toi.
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "-" + "!" (2 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - La poule fait cot cot- !
  - Merd-- !

### "-" + "tout" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'amour, c'est comme la rubéole - tout le monde doit en faire l'expérience.
  - Mais avec le vieux Tony, rien à lire, rien à écrire - tout dans la tête.

### "--" + "Tu" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est-- Tu sais.
  - J'appelle parce que-- Tu sais ce qui est arrivé au collège ?

### "--" + "un" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et je suis descendu comme d'habitude-- un vieux réflexe.
  - Regardez, Monsieur -- un droïde.

### "Allez" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Allez; allons-y.
  - Allez; rentrons à la maison.

### "Frères" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est donc le célèbre 'Livre des Frères' ?
  - C'est donc le fameux 'Livre des Frères' ?

### "Nation" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Avant la première de 'La Fierté de la Nation'.
  - Le film s'appelle 'La fierté de la Nation'.

### "a" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est ce qu'il y a , Beto ?
  - Qu'est ce qu'y a , baltringue ?

### "aujourd'hui" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne peut y avoir de crises aujourd'hui ; mon emploi du temps est déjà complet.
  - Il pleut aujourd'hui ; où ai-je donc mon parapluie ?

### "bien" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Mettre tout en équilibre, c'est bien ; mettre tout en harmonie, c'est mieux.
  - Ton ordinateur ne marche pas bien ; supprime les logiciels inutiles.

### "cheese" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dis 'cheese'.
  - Dites 'cheese'.

### "devenir" + "Jedi" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le fils de Skywalker ne doit pas devenirJedi.
  - Pourquoi veux-tu devenirJedi ?

### "en" + "fuir" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Elle a essayé de s'enfuir.
  - M'en aller, m'enfuir, me terrer quelque part !

### "filles" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il a trois filles ; une est mariée, mais pas les autres.
  - Surveille les filles ; elles ne savent pas nager.

### "flèche" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Le temps file comme une flèche ; un fruit s'effiloche comme une banane.
  - Le temps vole comme une flèche ; les drosophiles aiment une banane.

### "fumer" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je suis surpris de vous voir fumer ; vous ne le faisiez pas.
  - Je suis surprise de vous voir fumer ; vous ne le faisiez pas.

### "grand-mère" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Bien sûr, grand-mère ; allons te coucher.
  - Bien sûr, grand-mère ; allons te mettre au lit.

### "i" + "I" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mais qu'iI finisse ce qu'iI a commencé.
  - Mais qu'iI finisse ce qu'iI a commencé.

### "jeune" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tu es jeune ; tu ne peux pas prendre ta retraite.
  - Vous êtes jeune ; vous ne pouvez pas prendre votre retraite.

### "l" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il a fait un sans-faute à l'examen.
  - On a fini les travaux juste avant l'hiver.

### "le" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ignore-le ; c'est juste un gosse embêtant qui veut attirer l'attention.
  - Ignorez-le ; c'est juste un gosse embêtant qui veut attirer l'attention.

### "là" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hé, chef, regardez qui est là .
  - Seul un miracle pourrait le sortir de là .

### "marine" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il n'y a rien de comparable à un ex marine .
  - Mais pas avec une simple paie de marine .

### "nous" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Juste entre nous; êtes-vous amoureuse de ma sœur ?
  - Juste entre nous; êtes-vous amoureux de ma sœur ?

### "pas" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ils ont des yeux, mais ne voient pas; des oreilles, mais n'entendent pas.
  - Ne me demande pas; je suis un chat.

### "plaît" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dis 's'il te plaît'.
  - Tu as dit 's'il te plaît' ?

### "plaît" + ".." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Apportez-moi un café, s'il vous plaît ..
  - Une bière, s'il vous plaît ..

### "ressemble" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - En fait c'est lui qui te ressemble .
  - Je lui ressemble .

### "règles" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - L'anarchie n'est pas un manque de règles ; c'est un manque de dirigeants.
  - Rien à foutre des règles ; j'ai du fric !

### "utile" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce logiciel n'est pas utile ; supprime-le.
  - Ce logiciel n'est pas utile ; supprimez-le.

### "vous" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Calmez-vous ; il ne fait que vous taquiner.
  - Ne dites jamais du mal de vous ; vos amis en diront toujours assez.

### "«" + "Tom" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils l'appellent « Tom ».
  - Son père l'appelle « Tom ».

### "«" + "craigslist" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je l'ai vendu sur « craigslist ».
  - Je l'ai vendue sur « craigslist ».

### "«" + "ouistiti" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dis « ouistiti ».
  - Dites « ouistiti » !

### "«" + "soit" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dis « soit », tu dis.
  - Tu dis « soit ».

### "«" + "tu" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ah, « tu ne verras pas d'ici » ?
  - Un « tiens » vaut mieux que deux « tu l'auras ».

### "«" + "vieille" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C’est impoli de l’appeler « vieille peau ».
  - Dis plutôt « vieille dame ».

### "…" + "vous" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je…je…vous plains sincèrement.
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "'" + "Bonne" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un dur, ce ' Bonne nuit Anderson', alors fais gaffe.

### "'" + "assez" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai trouvé 'Gretchen Ross' assez cool.

### "'" + "d'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'entends juste le 'bip' d'une balise.

### "'" + "dans" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Moi et le mot 'devoir' dans la même phrase.

### "'" + "exigent" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les chevaliers qui disent 'Ni' exigent un sacrifice.

### "'" + "ne" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sauf que 'toi' ne veut plus rien dire.

### "'" + "roll" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'avenir du rock 'n' roll.

### "'" + "sur" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai un 'cube du destin' sur moi.

### "'" + "un" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai 'fait une nana' un tas de fois.

### "'" + "vous" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Par 'ces gars' vous voulez dire mes clients ?

### "'i" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - On va mettre les points sur les 'i'.

### "," + "je" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et moi,je le voyais à la mienne.

### "-" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous n'allaitons pas - - dans le sein de la faiblesse.

### "-" + "Ces" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ces cartes-- Ces cartes sont pourries.

### "-" + "Comment" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Norm, j'ai entendu du bien de toi- Comment se porte ton na'vi ?

### "-" + "Es" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quoi-- Es-tu fâchée ?

### "-" + "Non" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - N- Non, trois.

### "-" + "avec" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je-Je voulais dire avec- avec l'assaisonnement.

### "-" + "bien" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne suis pas illettré, mais cependant-- bien, alors.

### "-" + "c'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le meilleur remède pour le cœur - c'est la bonne vieille aspirine.

### "-" + "ce" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Est- ce que tu veux regarder un film ?

### "-" + "dans" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous n'allaitons pas - - dans le sein de la faiblesse.

### "-" + "debout" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Debout- debout !

### "-" + "des" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Clochers d'église - des entonnoirs renversés pour conduire les prières au ciel.

### "-" + "enfin" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pas toi, bien sûr ; tu es une femme - enfin presque.

### "-" + "et" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le télé-évangéliste a un public important - et bercé d'illusions.

### "-" + "j'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pendant un moment, j'aimais vraiment le cola - j'en buvais tous les jours.

### "-" + "je" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - N'approchez pas, je- je m'en servirai s'il le faut.

### "-" + "la" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La qualité d'image est vraiment mauvaise - la résolution est si basse.

### "-" + "le" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un fantôme hante l'Europe - le fantôme du communisme.

### "-" + "plat" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Son ventre me rappelle les cartes postales du Japon - plat et joli.

### "-" + "quelqu'un" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis quelqu'un d'ambitieux - quelqu'un qui sait très bien ce qu'il veut.

### "-" + "rien" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pardonnez toujours à vos ennemis - rien ne les agace plus que cela.

### "-" + "t" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - A- t-il passé d'autres soirées en votre compagnie ?

### "-" + "trouver" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ou--je ne sais pas-- trouver un emploi à plein temps ?

### "-" + "tu" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tes pantalons sont trop longs - tu vas marcher dessus.

### "-" + "À" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Attendez- À terre monsieur !

### "-" + "à" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il lui faut du repos et ceci - à prendre trois fois par jour.

### "--" + "C'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cette aventure était bien plus-- C'était très bien.

### "--" + "Comment" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Des métaphores ce sont-- Comment pourrais-je expliquer ?

### "--" + "Douglas Kelley" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis -- Douglas Kelley.

### "--" + "En" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je veux faire un élevage moderne-- En plein air !

### "--" + "Kirill Matféevitch" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je sais -- Kirill Matféevitch.

### "--" + "Laissons" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Laissons-- Laissons tomber, d'accord ?

### "--" + "Non" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je veux dire, je voudrais-- Non.

### "--" + "Oui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je sais que j'ai dit-- Oui, c'est vrai.

### "--" + "Puisque" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai pas de mulet-- Puisque tu me prêtes le tien.

### "--" + "Super" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je pense que c'est-- Super.

### "--" + "je" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le secret-- je l'ai vu derrière la porte !

### "--" + "que" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lola, que-- que diable fais-tu ?

### "--" + "quels" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les filets-- quels filets ?

### "--" + "qui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Leur pouvoir vient de leur leader-- qui est en quelque sorte leur cerveau.

### "--" + "thérapie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je te jetterais dans l'océan-- thérapie de choc.

### "<" + "courriel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Envoyez-nous votre CV détaillé à <courriel>.

### "Allez" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allez , pas de blague, personne ne veut être blessé, d'accord ?

### "Amour" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tout l'univers obéit à l'Amour ; aimez, aimez, tout le reste n'est rien.

### "Anderson" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Un dur, ce ' Bonne nuit Anderson', alors fais gaffe.

### "Big Apple" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - M'sieur, vous voulez apprendre le 'Big Apple' ?

### "Brouhaha" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Musique triste Brouhaha .

### "Chasseur de juifs" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qu'on vous appelle le 'Chasseur de juifs'.

### "Cherche" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Cherche ; trouve ; découvre !

### "Chuchotements" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Chuchotements .

### "Confirmez" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Confirmez .

### "Coupez" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Coupe pas tant que je dis pas 'Coupez'.

### "Doc" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qui vous a dit qu'on l'appelle 'Doc' ?

### "ELLE" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - COMMENT VA-T-ELLE …par le simple mouvement de l'air.

### "Et" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et … bien.

### "Europe" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un fantôme hante l'Europe - le fantôme du communisme.

### "FÉLICITATIONS" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - FÉLICITATIONS …car aujourd'hui, Oz écrit un nouveau chapitre.

### "Gretchen Ross" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai trouvé 'Gretchen Ross' assez cool.

### "Humains" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ou créer des esprits libres, et les appeler 'Humains'.

### "J'" + "ai" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J' ai fait un aller-retour la lune.

### "Japon" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Son ventre me rappelle les cartes postales du Japon - plat et joli.

### "Kitaro" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tenez, du porc pané de chez Kitaro ; mais pas extra.

### "La Taverne" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - T'iras peut-être à 'La Taverne'.

### "Lily" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ça s'appelle 'L'océan de Lily'.

### "Loyal" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et le second prénom, ce sera 'Loyal' ?

### "M" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je croyais qu'on avait choisi 'M' au hasard.

### "Mamie" + "eeee" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Jolie Mamieeeee !

### "Marcelo" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dis-moi, tu ne t'appelles pas Marcelo , n'est-ce pas ?

### "Marcuse" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Après ça, on a lu Marcuse; on est devenus marxistes.

### "Merde" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Merde ..

### "Monsieur" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regardez, Monsieur -- un droïde.

### "Ni" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Les chevaliers qui disent 'Ni' exigent un sacrifice.

### "O." + "C." (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Précédemment dans The O.C.

### "Ou" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ou'est-ce que c'est ?

### "Point" + "trait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Point trait point point.

### "R" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - T'aurais pas un 'R' à me prêter ?

### "Rabbiosu" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Rabbiosu ; rageur, furieux.

### "Sarfaraz" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis Sarfaraz ..

### "Shadow" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous m'avez dit 'Sauf pour M. Shadow'.

### "URGENCE" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - MESSAGE D'URGENCE … s'est divisé à l'infini pour se répandre sur le globe.

### "Urban Legends" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Notre prochaine fonctionnalité est «Urban Legends».

### "Vieux Garçon de Greer County" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'était 'Le Vieux Garçon de Greer County'.

### "X" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Place un 'X' à l'endroit opportun.

### "Y" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Les hommes ont un chromosome X et un Y ; les femmes, deux X.

### "abandonnée" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Maria Goretti, seule et abandonnée; un destin de femme.

### "activité" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Excellente activité .

### "additionne" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Qui travaille seul additionne ; qui travaille ensemble multiplie.

### "ah" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dites 'ah'.

### "aller" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne veut pas y aller ; moi non plus.

### "ambitieux" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis quelqu'un d'ambitieux - quelqu'un qui sait très bien ce qu'il veut.

### "ami" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment dit-on 'ami' en elfe ?

### "amour" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce qui compte, c'est l'amour ; tout le reste est superflu.

### "amour" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Nous sommes nés de l'amour; l'amour est notre mère.

### "annulée" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est une sensation merveilleuse d'être 'annulée'.

### "appartient" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce livre m'appartient ; j'ai moi-même écrit mon nom à l'intérieur.

### "apprenti" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - En vous quittant, j'étais l'apprenti; maintenant, c'est moi le maître.

### "araignée" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne l'appellerais pas une araignée ; je l'appellerais un monstre.

### "argent" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'où vient cet argent , si vous ne vendez pas quelque chose ?

### "argent" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils peuvent vous arranger la moelle épinière si vous avez de l'argent .

### "argent" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne volera pas mon argent ; j'ai confiance en lui.

### "arêtes" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce poisson est plein d'arêtes ; ne le mange pas.

### "assez" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous dites 'assez', qu'entendez-vous par là ?

### "assidûment" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il a étudié assidûment; autrement il aurait échoué de nouveau.

### "autres" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Elle ne prête pas attention aux autres ; en d'autres termes, elle est égoïste.

### "baronne" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Peut-on vous appeler autrement que 'Mme la baronne' ?

### "bas" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu ne resteras pas une nuit de plus là-bas ..

### "beaucoup" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Merci beaucoup ; c'était parfait.

### "berger" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Le Seigneur est mon berger ; je ne manquerai de rien.

### "besoin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Elle n'en aurait pas eu besoin ; elle allait bientôt mourir.

### "bible" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quelle bible , c'est une novela.

### "bien" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Très bien ..

### "bip" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'entends juste le 'bip' d'une balise.

### "blague" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Non, je blague; appelez-moi Papi.

### "blesser" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Oui, ça va la blesser; mais il faut voir sur le long terme.

### "bleu" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Les appels qui ont été faits du 'Boeuf bleu'.

### "bon" + "homme" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Allez, mon petit bonhomme.

### "bonbon" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - La presse a appelé ça 'la défense bonbon'.

### "bonne" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ma vue n'est pas bonne ; je dois porter des lunettes.

### "bébé" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai dis 'oh oui bébé', viens voir.

### "c" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le mieux que tu puisses faire, c'est essayer.

### "cabot" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Marie n'a pas de cabot; elle a un chaton.

### "cadeau" + ":" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alors il lui a acheté un cadeau: ce chaton.

### "cannabis" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je vais à la boutique de cannabis ; tu veux venir avec moi ?

### "cardinal" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Un est un nombre cardinal ; premier est un nombre ordinal.

### "ceci" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il lui faut du repos et ceci - à prendre trois fois par jour.

### "changement" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - L'univers est soumis au changement ; notre vie est ce que nos pensées en font.

### "chasse" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Papa est parti à la 'chasse'.

### "chat" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ma mère sait à peine compter ou épeler 'chat'.

### "chaton" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tom a un chien et Marie un chaton ; les deux animaux s’entendent bien.

### "cheveux" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il est grand temps de vous faire couper les cheveux ; ils sont trop longs.

### "chiens" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je n'aime pas vraiment les chiens ; Je suis plutôt une personne chat.

### "chip" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je fais un 'chip' au-dessus de l'eau.

### "chose" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je crois qu'il te manque quelque chose .

### "chose" + ":" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Interrogez-vous sur chaque chose: quelle est son essence ?

### "circonstances" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Au diable les circonstances ; je crée des opportunités.

### "cochon" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce n'est pas un cochon ; c'est un singe.

### "cola" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pendant un moment, j'aimais vraiment le cola - j'en buvais tous les jours.

### "colère" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tu ne seras pas puni pour ta colère ; Tu seras puni par ta colère.

### "communautaire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous avons une auto-protection communautaire .

### "conviens" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne suis pas brave, j’en conviens ; mais je ne suis pas superstitieux.

### "cornes" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Les chevaux n'ont pas de cornes ; les vaches et les moutons en ont.

### "correcte" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - La phrase est correcte ; je la formulerais cependant autrement.

### "couleur" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je n'ai jamais vu un métal de cette couleur ; c'est probablement un alliage.

### "courriel" + ">" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Envoyez-nous votre CV détaillé à <courriel>.

### "cristal" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai trouvé un cristal ; regardez-le !

### "cœur" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le meilleur remède pour le cœur - c'est la bonne vieille aspirine.

### "destin" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai un 'cube du destin' sur moi.

### "destin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je suis le maître de mon destin ; je suis le capitaine de mon âme.

### "devoir" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Moi et le mot 'devoir' dans la même phrase.

### "ding" + "eringeding" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ding-ding-ding-ding-dingeringeding !

### "dis" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Faites ce que je dis; ne faites pas ce que je fais.

### "dit" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le sage sait ce qu'il dit; le sot dit ce qu'il sait.

### "diversité" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ils aiment la diversité ; ils n'aiment pas rester au même endroit.

### "désastre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un désastre .

### "effet" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le deuxième acte s'appelle 'l'effet'.

### "effrayée" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Marie a été effrayée ; mais Tom n'a pas eu peur.

### "encore" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment ça, 'encore' ?

### "enfin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - L'aube parut enfin ; la longue nuit s'achevait.

### "ennemis" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pardonnez toujours à vos ennemis - rien ne les agace plus que cela.

### "ennui" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Une morale nue apporte de l'ennui ; le conte fait passer le précepte avec lui.

### "ennuyeux" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tu me trouves ennuyeux ; admets-le.

### "entêté" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne suis pas entêté ; je suis tenace.

### "es" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'étais ce que tu es ; tu seras ce que je suis.

### "est" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "faim" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne peut pas avoir faim ; il vient de déjeuner.

### "fait" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle veut savoir ce qu'elle a fait .

### "fait" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il s'est excusé, mais le mal était fait; après, c'était trop tard.

### "fardeau" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je sais que je suis un fardeau ; inutile de le répéter.

### "faut" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - On l'appelle le 'jeune homme comme il faut'.

### "femme" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pas toi, bien sûr ; tu es une femme - enfin presque.

### "fille" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Vous aimez ma fille ; mais êtes-vous sûr qu’elle vous aime ?

### "foire" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Nous allons à la foire ; viens-tu ?

### "froid" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il fait froid; j'ai la chair de poule.

### "frère" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'ai oublié le nom de votre frère; comment se nomme-t-il ?

### "fulminer" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Rien ne sert de fulminer ; s'indigner suffirait.

### "garde du corps" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu sais pas écrire 'garde du corps' ?

### "gars" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Par 'ces gars' vous voulez dire mes clients ?

### "gauche" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tournez à gauche quand je dis 'gauche'.

### "gauche" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je ne sens plus mon pied gauche; je n'y ai plus de sensation.

### "gaz" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - On ne peut pas voir le gaz ; il est invisible.

### "gentillesse" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mais votre 'gentillesse', elle nous rend petits comme ça.

### "girafe" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "gouvernement" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ensuite, qu'est-ce qui constitue le 'meilleur gouvernement' ?

### "grec" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ils sont en grec ; on ne peut pas les lire.

### "gâteau" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ça ne sert à rien de chercher le gâteau; je l'ai déjà mangé.

### "génotype" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et comme vous avez le même génotype , vous pourriez enfiler ses chaussures.

### "ha" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et je sais que la posture «ha».

### "heureuse" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Dis-toi que je suis une femme heureuse; j’ai trouvé ma place.

### "hier" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je l'ai envoyé hier; tu devrais le recevoir demain.

### "homme" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dites juste 'conséquences pour la santé de l'homme'.

### "homme" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je suis un très vieil homme ; de quel âge, je l'ignore.

### "horrible" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Pas spécialement horrible; non, non, pas du tout.

### "ici" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Elle n'est pas ici ; elle est chez le médecin.

### "ils" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qui ça, 'ils' ?

### "immense" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Le parc était immense ; on s'y est presque perdus.

### "important" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le télé-évangéliste a un public important - et bercé d'illusions.

### "inspecteur" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sur ma plaque, il y a 'inspecteur'.

### "inspiration" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Parce que je suis en panne d'inspiration; et j'ai besoin de changement.

### "intentions" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alors, tu disais 'meilleures intentions' ?

### "international" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - On va devoir se servir du 'langage international'.

### "irlandaise" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Si on donnait une touche 'irlandaise' à ces cafés ?

### "irrésistible" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Oui, on a appelé ça aussi 'impulsion irrésistible'.

### "issue" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - L'entrée principale semble être la seule issue; pas d'autre sur votre niveau.

### "jeunesse" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai lu beaucoup de livres dans ma jeunesse ; je suis un érudit à ma manière.

### "jour" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Seize heures font un jour ; huit heures font une nuit.

### "juifs" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alors tu es 'Le chasseur de juifs'.

### "lapin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il m'a posé un lapin; je l'ai attendu toute la soirée !

### "ligne" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Elle est en ligne ; qu'est-ce que je fais ?

### "limites" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sans limites …méchante sorcière.

### "lire" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ce n'est pas que je n'aime pas lire; c'est juste que je n'en ai pas le temps.

### "lis" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je lis; tu écris.

### "lit" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Maintenant je lis, tu lis et il lit ; nous lisons tous.

### "loin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Fais l'effort d'aller un peu plus loin ; ce n'est pas surpeuplé.

### "longs" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tes pantalons sont trop longs - tu vas marcher dessus.

### "loup" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai une faim de loup ; où puis-je trouver quelque chose à manger ?

### "lui" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je suis jaloux de lui ; tu l'aimes plus que moi.

### "là" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - C'est bien, là ; ni trop lourd ni trop léger.

### "là" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Erina est là; il faut que je sorte.

### "m" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et là, il dit : 'Tape-m'en quatre.'

### "maintenant" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il est trois heures maintenant ; je reviendrai dans une heure.

### "mais" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il y a un 'mais'.

### "maison" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La lettre que Tom reçut disait qu'il devait rentrer au plus tôt à la maison .

### "manger" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne vis pas pour manger ; je mange pour vivre.

### "marines" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Des marines .

### "mauvaise" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La qualité d'image est vraiment mauvaise - la résolution est si basse.

### "maître" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Le temps est un grand maître ; le problème, c'est qu'il tue ses élèves.

### "me" + "ure" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et maintenant, ils attendent tous qu'à mon tour, je meure.

### "merci" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je voudrais dire «merci» à Tom.

### "message" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'entends parfaitement le message ; c'est de foi que je manque.

### "mien" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce livre est le mien ; mon nom est écrit à l'intérieur.

### "mieux" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Si ça marche, tant mieux ; sinon, on aura essayé.

### "moi" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il sait que c'est votre autre 'moi'.

### "moi" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Celui qui avait formé Matias c'était moi , mon pote.

### "mort" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il n'est pas encore mort; on l'apporte ici.

### "moulin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ma vie est comme l'eau qui a passé le moulin; elle ne fait pas tourner de roue.

### "mourir" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'aime, je puis mourir ; j'ai vécu le meilleur et le plus beau des rêves !

### "musique" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je n'aime pas leur musique ; je trouve la voix de la chanteuse stridente.

### "musique" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'écoute toujours de la musique; je ne peux pas vivre sans.

### "n" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - L'avenir du rock 'n' roll.

### "nana" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai 'fait une nana' un tas de fois.

### "neige" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ça allait devenir 'L'année de la grande neige'.

### "noir" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il a peur du noir ; ne l'y laisse pas.

### "non" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Malheureusement non ; au contraire.

### "nu" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qu'entendez-vous par 'nu'.

### "nuit" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Oui, c'est une grande nuit; vous avez raison.

### "numéro" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Laissez-nous votre numéro ; nous vous rappellerons.

### "objets" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mes nanas sont des 'femmes objets', d'après elle.

### "obligée" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous n'y avez pas été 'obligée', non ?

### "oui" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Elle hochait la tête pour dire oui; et je restais couchée.

### "outrageusement" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Elle s'habille si outrageusement ; ça a l'air complètement ridicule !

### "où" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je parle au capitaine 'je-ne-sais-d'où'.

### "pacte" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - La première partie s'appelle 'le pacte'.

### "pain" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne beurre même pas mon pain ; je considère que c'est cuisiner.

### "pardon" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Vous demandez en vain le pardon ; votre acte ne peut pas être pardonné.

### "parlent" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elles vous parlent … euh… d’amour ?

### "parler" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu te rappelles qu'on a parlé de 'parler' ?

### "pas" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous n'allaitons pas - - dans le sein de la faiblesse.

### "passe" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La vie qui passe; c'est toujours la même chose.

### "patrie" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - On croit mourir pour la patrie; on meurt pour les industriels.

### "pauvre" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - D'une part, je suis pauvre ; d'autre part, je suis occupé.

### "payés" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est pour ça qu'ils sont payés .

### "personnelle" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Cette lettre est personnelle ; je ne veux pas que quelqu'un d'autre la lise.

### "peu" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le vrai bonheur coûte peu; s'il est cher, il n'est pas d'une bonne espèce.

### "piscine" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "planque" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment dit-on 'planque' en français ?

### "plaît" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Change de chaîne, s'il te plaît ; cette musique est insupportable.

### "point" + "point" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Point trait point point.

### "poison" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tout est poison et rien n'est sans poison; la dose seule fait le poison.

### "porte" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ouvre la porte , OK ?

### "possible" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous avez oublié deux mots très importants 'si possible'.

### "pot" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ne tournez-pas autour du pot ; nous avons un problème, n'est-ce pas ?

### "pote" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je te fais une remise 'vieux pote'.

### "potes" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Entre nous, on s'appelait 'les potes'.

### "pourquoi" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ne demandez pas pourquoi ; faites-le, tout simplement.

### "pouvoir" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le temps n’est pas un «pouvoir».

### "poète" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce n'est pas un poète ; il écrit de la prose.

### "promis" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je lui revaudrai ça l'an prochain, promis .

### "propre" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qu'il soit sympa, un peu rangé, et propre; peint en blanc.

### "putain" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Arrêtez la voiture putain .

### "père" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est pas facile, les rapports père—fille.

### "pêcher" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'aime pêcher ; c'est une façon très relaxante de passer la journée.

### "qu" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Un bonjour sincère vaut mieux qu'un long discours.

### "que" + "je" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu veux queje vienne ?

### "quitté" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'avais deux ans quand le bonheur nous a quitté; maman, papa et moi.

### "rabbit" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je viens d'acheter une pièce de la 'rabbit'.

### "raide" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai marché sur une corde raide ; je ne sais pas si je vais réussir.

### "raison" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Quand j'ai dit 'j'ai toujours raison' ?

### "rappel" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai fait 'rappel' et c'était toi.

### "recevrez" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Demandez et vous recevrez ; votre joie sera parfaite.

### "rien" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Mais nous ne savons vraiment rien ; car la vérité se trouve tout au fond.

### "roi" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est toi qui as dit 'roi'.

### "rouge" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et son pseudo-jargon militaire style 'Code rouge'.

### "rubéole" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'amour, c'est comme la rubéole - tout le monde doit en faire l'expérience.

### "rythmée" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - La musique électronique est rythmée ; elle sonne comme des battements du cœur.

### "récemment" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne l'ai pas vu récemment ; passe-lui le bonjour.

### "réussi" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne lui en veux pas d'avoir réussi ; elle a travaillé dur pour ça.

### "sais" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je sais -- Kirill Matféevitch.

### "sais" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Oui, je sais; t'es amoureuse de Richie ce que je trouve malsain et dégoûtant.

### "sales" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tes chaussures de sport sont sales ; retire-les avant d'entrer.

### "savoir-faire" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est quoi une 'marque de savoir-faire' ?

### "sens" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - La paix est un mot vide de sens ; c'est une paix glorieuse qu'il nous faut.

### "sentencieux" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il est si facile d'être sentencieux ; il est si dur d'être frivole.

### "serveurs" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Le bar Zailaiba embauche des serveurs ; es-tu intéressé ?

### "seul" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il était tout seul ; pas un chat n'était en vue.

### "sincère" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Elle est trop sincère; parfois ça me blesse.

### "smoking" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Non, je ne veux pas de smoking; j'ai demandé un costume !

### "soldat" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Quand on monte l'échelle, on trouve le 'soldat'.

### "suis" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis -- Douglas Kelley.

### "suis" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne sais pas où je suis ; pouvez-vous m'aider ?

### "survivront" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Les forts survivront ; les faibles périront.

### "séparions" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il faut que nous nous séparions ; le jour ne va pas tarder à paraître.

### "sûr" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Pas toi, bien sûr ; tu es une femme - enfin presque.

### "sûrs" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il y aura une réunion avec des gens 'sûrs'.

### "t" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et vers où compte-t'il la faire marcher ?

### "tapette" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le dernier mot qu'il a entendu était 'tapette'.

### "tard" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Nous le ferons plus tard ; ce n'est pas urgent.

### "tasse" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "temps" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Nous ne devons pas perdre de temps ; nous avons quelque chose à faire.

### "test" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qu'est-ce que tu veux dire par 'test' ?

### "they" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dites encore, they live or they've lived.

### "toi" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sauf que 'toi' ne veut plus rien dire.

### "toi" + ":" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Souviens-toi: ouvre le avant de le manger.

### "tours" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Les enfants adoraient me jouer des tours; et je présume que Dieu aussi.

### "tous" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un rêve pour tous; c'est que nous soyons tous unis !

### "tout" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a des bannières, des affiches, on trouve tout ..

### "trait" + "point" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Point trait point point.

### "transformation" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Que vouliez-vous dire par le terme 'transformation', docteur ?

### "transférer" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On doit la transférer …tout ira bien, Fräulein.

### "traquer" + "Jabba" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nous allons traquerJabba et le chasseur de primes.

### "travailler" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je n'ai pas envie de travailler; et si nous allions plutôt au cinéma ?

### "trouve" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Cherche ; trouve ; découvre !

### "vie" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Avec les animaux je veux passer ma vie ; ils sont si bonne compagnie !

### "vies" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai trois vies ; je peux en perdre jusqu'à deux.

### "vivifie" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La solitude vivifie; l'isolement tue.

### "vivre" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - En espagnol, ça veut dire 'Je te laisse vivre'.

### "voiture" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous m'avez dit 'un accident de voiture'.

### "voler" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Vivre signifie séduire et voler; tourbilloner et rayonner.

### "vous" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce qui fait de vous … un moins que rien.

### "vrai" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Rien n'est vrai ; tout est permis.

### "vue" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce n'est pas mon point de vue ; ce n'est que ma traduction !

### "y" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Rassemblez-les, allons-y .

### "you" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - We hope you've enjoyed learning your new language.

### "zone" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Quoi d'autre, 'hors de la zone' ?

### "«" + "Abide With Me" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du clerc épiscopalien écossais Henry Francis Lyte, « Abide With Me ».

### "«" + "Avatar" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Connaissez-vous le film « Avatar » ?

### "«" + "Der Stürmer" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fondateur du journal national antisémite « Der Stürmer ».

### "«" + "Docteur" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils s'adressèrent à moi par « Docteur ».

### "«" + "Fonds Monétaire International" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - FMI signifie « Fonds Monétaire International ».

### "«" + "Ha" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La première exclamation de John était, « Ha-ha !

### "«" + "Hummers" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les « Hummers » sont des gouffres à carburant.

### "«" + "Joyeuse" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'épée, on l'a baptisée « Joyeuse ».

### "«" + "L'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est moi, « L'Élu ».

### "«" + "Le" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est appelé « Le ménage ».

### "«" + "Les" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les enfants vont maintenant chanter « Les quatre Questions ».

### "«" + "Maman" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'appelle toujours ma mère « Maman ».

### "«" + "On" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Va falloir qu'on fasse ça tous les soirs pendant deux semaines « On » ?

### "«" + "Partenaires en Crise" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bienvenue à « Partenaires en Crise », un atelier pour duos au bord du désastre.

### "«" + "R" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai finalement appris comment rouler mes « R » !

### "«" + "Saint-Sylvestre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le dernier jour de l'année est appelé « Saint-Sylvestre ».

### "«" + "Strauss" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça se prononce « Strauss ».

### "«" + "a" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tes « o » ressemblent à des « a ».

### "«" + "ah" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Puis dites « ah ».

### "«" + "au" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a quitté la maison sans dire « au revoir ».

### "«" + "bagage" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous avons des interprétations différentes de ce que signifie le mot « bagage ».

### "«" + "bien" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dis « bien sûr » beaucoup trop souvent, n'est-ce pas ?

### "«" + "brillant" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que vous êtes un « brillant spécialiste des têtes ».

### "«" + "bœuf" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le pluriel de « bœuf » est « bœufs ».

### "«" + "bœufs" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le pluriel de « bœuf » est « bœufs ».

### "«" + "disparu" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai dit « disparu ».

### "«" + "d’" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lors de son combat contre Rengoku-san, Akaza a parlé « d’esprit combatif ».

### "«" + "exilés" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le chef des « exilés » de la nation.

### "«" + "givre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On appelle l'eau gelée « glace », et la rosée solidifiée « givre ».

### "«" + "glace" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On appelle l'eau gelée « glace », et la rosée solidifiée « givre ».

### "«" + "homard" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comment dis-tu « homard » en français ?

### "«" + "hommage" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quel est le mot juste pour « hommage », Sharon ?

### "«" + "ici" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quel est simplement le mystère de ce lieu appelé « ici » ?

### "«" + "idiot" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y’a marqué « idiot » sur mon front ?

### "«" + "il" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est-ce que tu veux dire par « il y a un problème », Lou ?

### "«" + "je" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle me fit un clin d’œil qui semblait signifier « je t'aime ».

### "«" + "l'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il baptisa le navire « l'Hirondelle ».

### "«" + "la" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si on s'arrête ici, on se retrouvera à « la case départ » !

### "«" + "lobster" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comment dis-tu « lobster » en français ?

### "«" + "merci" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu devrais au moins dire « merci ».

### "«" + "moi" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est qui, « moi » ?

### "«" + "mon" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'aurai, comme vous dites, « mon jour au tribunal ».

### "«" + "ni" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dis « ni l'un ni l'autre », tu dis.

### "«" + "non" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne prends pas « non » pour une réponse.

### "«" + "nègre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom a utilisé le mot « nègre ».

### "«" + "o" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tes « o » ressemblent à des « a ».

### "«" + "oui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La réponse est « oui ».

### "«" + "pape" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom ne connaît pas la différence entre « pape » et « pope ».

### "«" + "pope" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tom ne connaît pas la différence entre « pape » et « pope ».

### "«" + "science" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hitler qualifiait la physique quantique de « science juive ».

### "«" + "smoothie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je n'aime même pas prononcer le mot « smoothie ».

### "«" + "solution" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non, « solution finale ».

### "«" + "synonyme" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Y a t-il un autre mot pour « synonyme » ?

### "«" + "tant" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que voulez-vous dire par « tant de personnes » ?

### "«" + "tiens" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un « tiens » vaut mieux que deux « tu l'auras ».

### "«" + "travailler" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu connais ce mot, « travailler » ?

### "«" + "trobairitz" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Une femme troubadour est appelée communément « trobairitz ».

### "«" + "une" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est ce qu'on appelle « une encyclopédie sur pattes ».

### "«" + "viré" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Être licencié, c'est être « viré » ?

### "«" + "état" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - ESPT signifie « état de stress post-traumatique ».

### "État" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que nous ne pouvons pas admettre, c'est qu'un représentant de l'État ..

### "ça" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Pour moi ça'a pas d'allure.

### "ça" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne t'ai pas amené ici pour ça , mon pote.

### "ça" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Me regarde pas comme ça; y a rien de tel.

### "écrire" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais avec le vieux Tony, rien à lire, rien à écrire - tout dans la tête.

### "égalité" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Nous sommes sur le même plan d'égalité ; pas d'infériorité ni de supériorité.

### "église" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Clochers d'église - des entonnoirs renversés pour conduire les prières au ciel.

### "étranger" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il n'y a pas de pays étranger; c'est le voyageur qui seulement est étranger.

### "évidemment" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il est très mal soigné, évidemment; il commence à souffrir beaucoup.

### "île" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est 'île' en espagnol mais à l'envers.

### "—" + "fille" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est pas facile, les rapports père—fille.

### "…" + "assis" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hier vous…assis !

### "…" + "boire" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ils aiment là où… humidité… pour…boire.

### "…" + "bonjour" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu peux m'expliquer, quelle…bonjour… quelle mouche t'a piqué hier ?

### "…" + "c'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "…" + "car" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - FÉLICITATIONS …car aujourd'hui, Oz écrit un nouveau chapitre.

### "…" + "d'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### "…" + "et" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et…et où… ?

### "…" + "euh" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### "…" + "il" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bonjour, c'est Novoseltsev… …il crache ?

### "…" + "j'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je…j'ai l'air malade ?

### "…" + "je" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je…je…vous plains sincèrement.

### "…" + "mais" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Non…mais on ne peut pas trouver autre chose que de la séduire ?

### "…" + "méchante" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sans limites …méchante sorcière.

### "…" + "oui" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - O…oui.

### "…" + "par" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - COMMENT VA-T-ELLE …par le simple mouvement de l'air.

### "…" + "tout" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - On doit la transférer …tout ira bien, Fräulein.

### "…" + "une" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Traite-la comme…une femme.

### "…" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bonjour, c'est Novoseltsev… …il crache ?

### "………" + "cinq" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Trente………cinq !

