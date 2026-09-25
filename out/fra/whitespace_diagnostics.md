# Whitespace Prediction Diagnostics

Total predictions: 4697187
Total errors: 1240
Accuracy: 99.97%

## Error Patterns (sorted by frequency)

### "'" + "?" (27 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - Alors, tu disais 'meilleures intentions' ?
  - C'est donc le célèbre 'Livre des Frères' ?
  - C'est donc le fameux 'Livre des Frères' ?

### "," + "'" (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est qui, 'on' ?
  - Comment ça, 'encore' ?
  - Comment ça, 'nous' ?

### "," + "p'" (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors, p'tit gars ?
  - Fais pas le malin, p'tit con.
  - Hein, p'tit ?

### "," + "«" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ah, « tu ne verras pas d'ici » ?
  - C'est moi, « L'Élu ».
  - C'est qui, « moi » ?

### "ça" + "," (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est ça , fais-lui un garrot au bras.
  - C'est ça , ramènes ton gros cul chez maman.
  - Comment tu sais ça, toi ?

### "," + "non" (6 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - C’est intéressant, non ?
  - Elle est super, non ?
  - Joli, non ?

### "--" + "!" (6 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - H-Hey-- !
  - Où est-ce que-- ! ?
  - Quoii-- ! ?

### "non" + "?" (6 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - C’est intéressant, non ?
  - Elle est super, non ?
  - Joli, non ?

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

### "can" + "'t" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - But I can't spend my life at the laundromat.
  - I can't believe it.
  - The holiday feast, we can't miss that.

### "nous" + "'" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est qui 'nous' ?
  - Comment ça, 'nous' ?
  - Qu'est-ce que tu veux dire par 'nous' ?

### "pas" + "." (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Celà n'arrivera pas .
  - Ne l'encourage pas .
  - Ne tirez pas .

### "pas" + ";" (4 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ceux qui savent ne parlent pas ; ceux qui parlent ne savent pas.
  - Ne le leur explique pas ; tu perds ton temps.
  - Ne le leur expliquez pas ; vous perdez votre temps.

### " " + "!" (4 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je dois prêter main-forte à Rengoku ! !
  - Suzume ! !
  - Venez vite ! !

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

### "," + "hein" (3 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Et toujours la même expression, hein ?
  - Magnifique, hein ?
  - Vous ne comprenez pas, hein ?

### "," + "monsieur" (3 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Comment fait-on pour séparer son âme en deux, monsieur ?
  - Quel stylo, monsieur ?
  - Savez vous ce que vous prendrez ce soir, monsieur ?

### "-" + "le" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dites, surveillez- le discrètement.
  - Il y a trop de machisme en tauromachie, reconnaissez- le.
  - Un fantôme hante l'Europe - le fantôme du communisme.

### "Allez" + "," (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allez , Enfilez vos Exo-packs !
  - Allez , pas de blague, personne ne veut être blessé, d'accord ?
  - Allez , tu n'as pas le niveau.

### "J" + "‘" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J‘ai fêté mon anniversaire dans un restaurant.
  - J‘aime celui-ci.
  - J‘aime les soupes avec de nombreux végétaux.

### "bien" + "," (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tout va bien , je maitrise.
  - Très bien , regardez autour de vous.
  - Ça sonne bien, non ?

### "d" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce tableau est considéré comme un chef-d'œuvre.
  - Je me casse d'ici.
  - Je veux que tu sortes d'ici.

### "et" + "je" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - D'ailleurs, je ne suis pas amateur etje vous assure que ça m'est interdit.
  - Je souffle un instant etje prépare à manger.
  - Je touche le chèque etje te rends tout mon dû.

### "hein" + "?" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et toujours la même expression, hein ?
  - Magnifique, hein ?
  - Vous ne comprenez pas, hein ?

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

### "là" + "." (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hé, chef, regardez qui est là .
  - Ici, mets ta patte là .
  - Seul un miracle pourrait le sortir de là .

### "monsieur" + "?" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Comment fait-on pour séparer son âme en deux, monsieur ?
  - Quel stylo, monsieur ?
  - Savez vous ce que vous prendrez ce soir, monsieur ?

### "on" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est qui, 'on' ?
  - Comment ça, 'on' ?
  - Qui ça, 'on' ?

### "t" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et vers où compte-t'il la faire marcher ?
  - L'a-t'il déjà dit ?
  - Pourquoi y-a't'il un tel fouillis ici, ma fille ?

### "toi" + "," (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et toi , Mark, tu as hâte de devenir papa ?
  - Relève-toi, bordel !
  - Replies toi , sors de là.

### "toi" + ";" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Calme-toi ; il ne fait que te taquiner.
  - Je ne priais pas contre toi ; je priais pour toi.
  - Je ne te parle pas à toi ; je parle au singe.

### "vous" + "." (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ca ne devrait pas être trop dur pour vous .
  - Non, vous, réveillez vous .
  - Regardez tout autour de vous .

### "…" + "de" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - De quoi…de champignons.
  - Et ce Borovskikh……de l'industrie locale ?
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### "…" + "vous" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai besoin de…vous consulter.
  - Je…je…vous plains sincèrement.
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

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

### "," + "cours" (2 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Milkha, cours !
  - Paul, cours !

### "," + "ici" (2 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - En avez-vous tous fini, ici ?
  - Les seuls criminels portent les uniformes nazis, ici !

### "," + "papa" (2 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Qu'est-ce qui se passe, papa ?
  - Tu entends, papa ?

### "-" + "!" (2 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - La poule fait cot cot- !
  - Merd-- !

### "-" + "ce" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Est- ce que je dois entrer par derrière ?
  - Est- ce que tu veux regarder un film ?

### "-" + "moi" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Excusez- moi.
  - Regarde- moi.

### "-" + "nous" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comment allons- nous nous battre ?
  - Qui sommes-nous, à quelle fin sommes- nous ici, où courrons-nous aveuglément ?

### "-" + "t" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - A- t-il passé d'autres soirées en votre compagnie ?
  - Comment va- t-elle ?

### "-" + "tout" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'amour, c'est comme la rubéole - tout le monde doit en faire l'expérience.
  - Mais avec le vieux Tony, rien à lire, rien à écrire - tout dans la tête.

### "-" + "vous" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Avez- vous conclu ?
  - Quoi qu'il puisse arriver, adressez- vous à Pierre, il vous secourra.

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
  - Et je suis descendu comme d'habitude-- un vieux rêflexe.
  - Regardez, Monsieur -- un droïde.

### "Allez" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Allez; allons-y.
  - Allez; rentrons a la maison.

### "Alors" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors , arrête-le quand il sortira en ville.
  - Alors , voici mon bunker.

### "Frères" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est donc le célèbre 'Livre des Frères' ?
  - C'est donc le fameux 'Livre des Frères' ?

### "Monsieur" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Monsieur , je regrette.
  - Monsieur , tous les escorts se sont enfuis ou ont été abattus.

### "Nation" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Avant la première de 'La Fierté de la Nation'.
  - Le film s'appelle 'La fierté de la Nation'.

### "Non" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non , non , non !
  - Non , non !

### "a" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est ce qu'il y a , Beto ?
  - Qu'est ce qu'y a , baltringue ?

### "allé" + "?" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Où es-tu allé ?
  - Où tout le monde est-il allé ?

### "aujourd'hui" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne peut y avoir de crises aujourd'hui ; mon emploi du temps est déjà complet.
  - Il pleut aujourd'hui ; où ai-je donc mon parapluie ?

### "aussi" + "?" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Celle-là aussi ?
  - Ça aussi ?

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

### "cours" + "!" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Milkha, cours !
  - Paul, cours !

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

### "est" + "juste" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'estjuste que c'est faux, malhonnête et ce n'est pas moi.
  - C'estjuste que c'est un peu trop difficile.

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

### "ici" + "?" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - En avez-vous tous fini, ici ?
  - Pourquoi ce rendez-vous ici ?

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

### "lui" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai suivi l'entrainement d'avatar avec lui .
  - Non, jamais entendu parler de lui .

### "moi" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Celui qui avait formé Matias c'était moi , mon pote.
  - Tu as besoin de moi , Guido ?

### "n" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - L'avenir du rock 'n' roll.
  - Let's rock'n'roll !

### "non" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hmm non , je te manquerais.
  - Non , non , non !

### "nous" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Juste entre nous; êtes-vous amoureuse de ma sœur ?
  - Juste entre nous; êtes-vous amoureux de ma sœur ?

### "papa" + "?" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qu'est-ce qui se passe, papa ?
  - Tu entends, papa ?

### "pas" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'y crois pas , laisses moi voir eh oui, tu sais ce que c'est.
  - Ne vous arrétez pas , allez y !

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

### "que" + "j'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je ne me souviens pas de la moitié de ce quej'ai fait.
  - Tout ce quej'ai appris, je l'ai su par des tiers et par hasard.

### "règles" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - L'anarchie n'est pas un manque de règles ; c'est un manque de dirigeants.
  - Rien à foutre des règles ; j'ai du fric !

### "scientifique" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis scientifique , vous vous en souvenez ?
  - Tommy était le scientifique , pas moi.

### "toi" + "?" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Est-ce que ça a un sens pour toi ?
  - Sont-elles venues avec toi ?

### "utile" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce logiciel n'est pas utile ; supprime-le.
  - Ce logiciel n'est pas utile ; supprimez-le.

### "va" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On y va, on y va , on y va !
  - Ça va, Lance ?

### "vous" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Calmez-vous ; il ne fait que vous taquiner.
  - Ne dites jamais du mal de vous ; vos amis en diront toujours assez.

### "vous" + "?" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Elle est bien tenue, d'après vous ?
  - Mes hommes se sont-ils bien occupés de vous ?

### "y" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Equipe mech allez y .
  - Rassemblez les , allons y .

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

### "—" + "moi" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Arrête—moi ça.
  - Excusez—moi !

### "…" + "c'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est…c'est un petit appartement !
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "…" + "d'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Oh…d'un concombre.
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### "…" + "euh" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Elles vous parlent …euh…d’amour ?
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### "" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qu'elle violence ? !

### " meu" + "f" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est vraiment le truc classe à faire quand on a dépucélé une meuf.

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

### ")" + "ce" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qu'est)ce qui s'est passé ?

### "," + "Benni" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ouvre la porte, Benni !

### "," + "Capitaine" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Voulez-vous que je prenne les commandes, Capitaine ?

### "," + "Carmelino" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Non, Carmelino !

### "," + "Fabinho" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Oh, Fabinho !

### "," + "Frank" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Hé, Frank !

### "," + "Jasmeet" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Salut, Jasmeet !

### "," + "Jim" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tu savais, Jim ?

### "," + "Jordan" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tu veux me baiser, Jordan ?

### "," + "Kimmie" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quatre maudites fois, Kimmie !

### "," + "Lamar" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Qu'allez-vous faire, Lamar ?

### "," + "Lance" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ça va, Lance ?

### "," + "Marla" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Réponds, Marla !

### "," + "Marty" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Yo, Marty !

### "," + "Mikiya" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quoi, Mikiya ?

### "," + "Paulo" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Allô, Paulo ?

### "," + "Pippin" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Et toi, Pippin ?

### "," + "Tignasse" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Et toi, Tignasse ?

### "," + "Tom" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Que fais-tu ce soir, Tom ?

### "," + "Zico" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tu veux nous faire peur, Zico ?

### "," + "alors" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ne seraient-ils pas malheureux, alors ?

### "," + "bordel" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Relève-toi, bordel !

### "," + "d'" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - C'est quoi, d'ailleurs ?

### "," + "d’" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Que faites-vous chez toi, d’ordinaire ?

### "," + "enfoiré" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Où est ce fils de pute, enfoiré ?

### "," + "général" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Pourquoi ne commencez-vous pas, général ?

### "," + "je" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et moi,je le voyais à la mienne.

### "," + "lui" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Qui, lui ?

### "," + "là" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tu vois le barbu, là ?

### "," + "maintenant" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - As-tu du temps, maintenant ?

### "," + "maman" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Pas vrai, maman ?

### "," + "mec" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Que foutent ces mines ici, mec ?

### "," + "pignouf" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Le vôtre pue du cul, pignouf !

### "," + "pourriture" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Regarde-moi dans les yeux, pourriture !

### "," + "quoi" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ils s'amènent toujours la nuit, et nous, quoi ?

### "," + "recule" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tanjiro, recule !

### "," + "toi" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Comment tu sais ça, toi ?

### "," + "épouvantail" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Regarde-toi toi-même, épouvantail !

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

### "-" + "en" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Croyez- en mon expérience.

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

### "-" + "ils" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si je n'étais pas revenu, où auraient- ils eu l'argent pour manger ?

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

### "-" + "lui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si tu n'es pas occupé, tiens- lui compagnie.

### "-" + "petite" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est ma petite- petite-petite-fillotte.

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

### "-" + "toi" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Suicide- toi, suicide-toi, suicide-toi.

### "-" + "trois" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un-deux- trois-un..

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

### "-" + "à" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il lui faut du repos et ceci - à prendre trois fois parjour.

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
  - Je sais quej'ai dit-- Oui, c'est vrai.

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

### "--" + "ça" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Avec votre permission, Sire, Je dois vérifier l'intégrité-- ça va.

### "<" + "courriel" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Envoyez-nous votre CV détaillé à <courriel>.

### "?" + "" (1 occurrences)
- Predicted: None
- Actual: NarrowNbsp
- Examples:
  - Qu'elle violence ? !

### "ATO" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils faut qu'ils envoient les ATO , pour sécurisez une zone d'accès.

### "Allez" + "jusqu'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Allezjusqu'au lycée !

### "Allô" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allô, Paulo ?

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

### "Andrei" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Après-demain, nous allons voir Andrei .

### "Arrête" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Arrête—moi ça.

### "Asseyez" + "–" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Asseyez–vous.

### "Aussi" + "vite" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Aussi vite ?

### "Avant" + "j'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Avantj'étais amie avec Volodia et maintenantje sors avec Nikita.

### "Benni" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ouvre la porte, Benni !

### "Big Apple" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - M'sieur, vous voulez apprendre le 'Big Apple' ?

### "Black Kite" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Maintenant c'est une opération Black Kite , Tu ne réponds qu'à moi.

### "Bon" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bon , ça y est, le Rouge ?

### "Brouhaha" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Musique triste Brouhaha .

### "Capitaine" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Voulez-vous que je prenne les commandes, Capitaine ?

### "Carmelino" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Non, Carmelino !

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

### "Comment" + "décider" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Comment décider ?

### "Comment" + "va" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Comment va ?

### "Confirmez" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Confirmez .

### "Copan" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est un immeuble qui ressemble beaucoup au Copan , je trouve.

### "Coupez" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Coupe pas tant que je dis pas 'Coupez'.

### "Courir" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Courir , rester , alors quoi ? !

### "Des" + "frites" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Des frites ?

### "Des" + "moustiques" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Des moustiques ?

### "Des" + "signatures" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Des signatures ?

### "Des" + "souhaits" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Des souhaits ?

### "Des" + "truands" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Des truands ?

### "Dhanbad" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dhanbad ..

### "Doc" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qui vous a dit qu'on l'appelle 'Doc' ?

### "Double Lien" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Oui, j'ai regardé Double Lien .

### "Doucement" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Doucement , faites ce qu'on vous dit Jake Ok ?

### "Du" + "Polynectar" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Du Polynectar ?

### "Du" + "liquide" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Du liquide ?

### "Désir" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Virna Lisi en vedette dans la première d'Amour, Haine et Désir .

### "ELLE" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - COMMENT VA-T-ELLE …par le simple mouvement de l'air.

### "Europe" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un fantôme hante l'Europe - le fantôme du communisme.

### "Excusez" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Excusez—moi !

### "Fabinho" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Oh, Fabinho !

### "Frank" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Hé, Frank !

### "FÉLICITATIONS" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - FÉLICITATIONS …car aujourd'hui, Oz écrit un nouveau chapitre.

### "Führer" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mon Führer !

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

### "Hé" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hé, Frank !

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

### "Jasmeet" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Salut, Jasmeet !

### "Jim" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tu savais, Jim ?

### "Jones" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - M. Jones ?

### "Jordan" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tu veux me baiser, Jordan ?

### "Kimmie" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quatre maudites fois, Kimmie !

### "Kingsbury" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Porte de Kingsbury !

### "Kitaro" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tenez, du porc pané de chez Kitaro ; mais pas extra.

### "La" + "carte" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - La carte !

### "La" + "constipation" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - La constipation ?

### "La" + "garce" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - La garce !

### "La" + "princesse" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - La princesse ?

### "La" + "solution" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - La solution ?

### "La Taverne" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - T'iras peut-être à 'La Taverne'.

### "Lamar" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qu'allez-vous faire, Lamar ?

### "Lance" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ça va, Lance ?

### "Le" + "corps" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Le corps ?

### "Le" + "portable" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Le portable !

### "Le" + "travail" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Le travail ?

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

### "Lui" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lui , il est sympa, c'est Bobby Hogan.

### "M" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je croyais qu'on avait choisi 'M' au hasard.

### "M." + "Jones" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - M. Jones ?

### "Ma" + "montre" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ma montre !

### "Main" + "coupée" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Main coupée ?

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

### "Marla" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Réponds, Marla !

### "Marty" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Yo, Marty !

### "Merde" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Merde ..

### "Mexique" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Es-tu déjà allé au Mexique ?

### "Mikiya" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quoi, Mikiya ?

### "Mme" + "Powell" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Mme Powell ?

### "Mon" + "Führer" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Mon Führer !

### "Mon" + "amour" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Mon amour !

### "Mon" + "peigne" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Mon peigne !

### "Monsieur" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regardez, Monsieur -- un droïde.

### "N" + "Alors" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - NAlors pourquoi tu n'utilises pas la tienne ?

### "N" + "Arrête" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - NArrête de me donner des ordres !

### "NE" + "t" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - NEt mon ardent désir d'étudier ? !

### "Ni" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Les chevaliers qui disent 'Ni' exigent un sacrifice.

### "Non" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non ..

### "O." + "C." (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Précédemment dans The O.C.

### "Omaticaya" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis tombé amoureux de la forêt, du peuple Omaticaya .

### "Ou" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ou'est-ce que c'est ?

### "PK" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis PK .

### "Paris" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les étudiants appellent à la révolution culturelle à Paris .

### "Paulo" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Allô, Paulo ?

### "Peer Sai" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'année dernière, j'ai prié pourl'âme de Daljeet sur la tombe de Peer Sai .

### "Pippin" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et toi, Pippin ?

### "Plus" + "fort" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Plus fort ?

### "Point" + "trait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Point trait point point.

### "Polynectar" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Du Polynectar ?

### "Pourquoi" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pourquoi , on est quel jour ?

### "Powell" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mme Powell ?

### "Quel" + "bus" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quel bus ?

### "Quel" + "froid" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quel froid !

### "Quel" + "signal" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quel signal ?

### "Quel" + "sort" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quel sort ?

### "Quel" + "vacarme" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quel vacarme !

### "Quelle" + "nana" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quelle nana ?

### "Quelle" + "technique" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Quelle technique !

### "Qui" + "paie" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Qui paie ?

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

### "Rendez" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Rendez—lui directement.

### "Réponds" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Réponds, Marla !

### "Sarfaraz" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis Sarfaraz ..

### "Se" + "marier" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Se marier ?

### "Se" + "pencher" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Se pencher !

### "Ses" + "collants" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ses collants ?

### "Shadow" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous m'avez dit 'Sauf pour M. Shadow'.

### "Si" + "loin" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Si loin ?

### "Six Un Un" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eh bien, voilà un point de contrôle illégal sur la Six Un Un , Caporal Pearson.

### "Son" + "frère" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Son frère ?

### "Tignasse" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et toi, Tignasse ?

### "Tom" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Que fais-tu ce soir, Tom ?

### "Ton" + "peigne" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Ton peigne ?

### "Ton" + "témoin" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ton témoin ?

### "Trop" + "beau" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Trop beau !

### "Très" + "flattée" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Très flattée !

### "Très" + "profond" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Très profond !

### "URGENCE" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - MESSAGE D'URGENCE … s'est divisé à l'infini pour se répandre sur le globe.

### "Un" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un , deux, trois.

### "Un" + "agent" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un agent ?

### "Un" + "amant" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un amant ?

### "Un" + "meurtrier" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un meurtrier ?

### "Un" + "partenaire" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un partenaire ?

### "Un" + "sacrifice" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un sacrifice ?

### "Un" + "service" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un service ?

### "Un" + "shampoing" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Un shampoing !

### "Un" + "traître" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Un traître !

### "Une" + "amie" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Une amie ?

### "Une" + "caution" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Une caution ?

### "Une" + "douche" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Une douche ?

### "Une" + "mission" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Une mission ?

### "Une" + "ostéopathe" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Une ostéopathe ?

### "Une" + "sorcière" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Une sorcière ?

### "Urban Legends" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Notre prochaine fonctionnalité est «Urban Legends».

### "Vagues" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vagues .

### "Vieux Garçon de Greer County" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'était 'Le Vieux Garçon de Greer County'.

### "Vous" + "osez" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Vous osez !

### "Workman" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Workman .

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

### "Zico" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tu veux nous faire peur, Zico ?

### "^" + "iné" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sieur a^iné, je meurs de faim.

### "a" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Pourquoi y-a't'il un tel fouillis ici, ma fille ?

### "a" + "^" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sieur a^iné, je meurs de faim.

### "abandonnée" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Maria Goretti, seule et abandonnée; un destin de femme.

### "abord" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tout d'abord , tu es de combien ?

### "acier" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si vous voulez survivre il vous faudra adopter un moral d'acier .

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

### "agent" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un agent ?

### "agitation" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voyez l'agitation .

### "ah" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dites 'ah'.

### "aide" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais j'ai besoin d'aide .

### "ailleurs" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - C'est quoi, d'ailleurs ?

### "aimé" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu l'as beaucoup aimé .

### "aise" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il n'y a rien de tel que la bonne vieille école pour vous mettre à l'aise .

### "aller" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne veut pas y aller ; moi non plus.

### "aller" + "jusqu'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu vas pas allerjusqu'au bout de ce plan débile ?

### "alors" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Qu'y a-t-il alors ?

### "alors" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ne seraient-ils pas malheureux, alors ?

### "amant" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un amant ?

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

### "amie" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Une amie ?

### "amour" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mon amour !

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

### "anna" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Donne-moi une anna .

### "annulée" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est une sensation merveilleuse d'être 'annulée'.

### "appartient" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je crois que le pouvoir leur appartient .

### "appartient" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce livre m'appartient ; j'ai moi-même écrit mon nom à l'intérieur.

### "apparu" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Dans quelles circonstances le défaut est-il apparu ?

### "apporté" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qu'as-tu apporté ?

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

### "argent" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne volera pas mon argent ; j'ai confiance en lui.

### "argent" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Comment êtes-vous venus en possession de tout cet argent ?

### "arrivé" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Ça t'est déjà arrivé ?

### "arrivée" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - À quelle heure y es-tu arrivée ?

### "arrêter" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Papa lui demanda d'arrêter .

### "arêtes" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce poisson est plein d'arêtes ; ne le mange pas.

### "as" + "senti" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - T'as senti ?

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

### "assiettes" + "Julie Andrews" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Un modèle d'assiettesJulie Andrews pour la Bourse Bradford ?

### "atout" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai étudié la langue Fremen, je serai un atout .

### "au" + "millet" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Et des pains au millet ?

### "aussi" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tout comme la police, je croyais moi aussi ..

### "autres" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Elle ne prête pas attention aux autres ; en d'autres termes, elle est égoïste.

### "aux" + "cellules" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tout le monde aux cellules !

### "avancent" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Les travaux avancent ?

### "avec" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ne jouez pas avec , Vous pourriez devenir aveugle.

### "avis" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Que s'est-il passé, à votre avis ?

### "babu" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hé Piku, ton Paanchu babu .

### "baronne" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Peut-on vous appeler autrement que 'Mme la baronne' ?

### "bas" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et pas d'erreur les gars , ils sont là bas .

### "batteries" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'espère que tu sais utiliser une radio sans batteries , opérateur Jones.

### "beau" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Trop beau !

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

### "bien" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors tout va bien .

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
  - Non, je blague; appelez moi Papi.

### "blazer" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pouvez-vous lui enlever son blazer , s'il vous plaît.

### "blesser" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Oui, ça va la blesser; mais il faut voir sur le long terme.

### "blessé" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On emmène le blessé .

### "bleu" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Les appels qui ont été faits du 'Boeuf bleu'.

### "bon" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est bon , je m'en occupe.

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

### "bordel" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Relève-toi, bordel !

### "bras" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Rentrez vos bras , et vos mains.

### "bruyante" + "bête" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - La bruyante bête !

### "bus" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quel bus ?

### "bébé" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai dis 'oh oui bébé', viens voir.

### "bé !" + "" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et ils sont déjà sur place… avec leur bébé !

### "bête" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La bruyante bête !

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

### "capacités" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je commence à douter de tes capacités .

### "cardinal" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Un est un nombre cardinal ; premier est un nombre ordinal.

### "carte" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La carte !

### "caution" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Une caution ?

### "ce" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui est-ce , Frederick ?

### "ce" + "correct" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Est-ce correct ?

### "ce" + "voyage" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Et ce voyage ?

### "ceci" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il lui faut du repos et ceci - à prendre trois fois parjour.

### "cellules" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tout le monde aux cellules !

### "ces" + "jours" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je le prends les mardis et jeudis comme tu es en congé cesjours-là.

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

### "chose" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allez, fais quelque chose ..

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

### "cocada" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est de la cocada .

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

### "collants" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ses collants ?

### "colère" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Tu ne seras pas puni pour ta colère ; Tu seras puni par ta colère.

### "commercial" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Centre commercial , hein ?

### "communautaire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous avons une auto-protection communautaire .

### "comprendre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Étant un noble guerrier moi-même, je n'arrive pas à comprendre .

### "conjure" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mon frère , je t'en conjure , n'attaque pas les créatures du ciel.

### "connaissez" + "Jeremy" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous connaissezJeremy ?

### "connection" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous voici dans la chambre de connection .

### "conscience" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous ne souhaitez pas avoir ce sang sur la conscience .

### "considérable" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce qui est considérable , je présume.

### "constipation" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La constipation ?

### "contrat" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous aimerions vous demander de reprendre son contrat .

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

### "corps" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le corps ?

### "correct" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Est-ce correct ?

### "correcte" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - La phrase est correcte ; je la formulerais cependant autrement.

### "costauds" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a une rangée externes de colonnes, Des matériaux vraiment costauds .

### "couleur" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je n'ai jamais vu un métal de cette couleur ; c'est probablement un alliage.

### "coupée" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Main coupée ?

### "courir" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu dois courir , OK ?

### "courriel" + ">" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Envoyez-nous votre CV détaillé à <courriel>.

### "cousins" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Vous êtes des cousins ?

### "cristal" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai trouvé un cristal ; regardez-le !

### "cul" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le vôtre pue du cul, pignouf !

### "cœur" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le meilleur remède pour le cœur - c'est la bonne vieille aspirine.

### "d" + "â" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et tout sera sec en un clin dâ€™œil.

### "de" + "Josie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je voulais faire partie deJosie et les Pussycats.

### "de" + "Kingsbury" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Porte de Kingsbury !

### "de" + "jouer" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Arrêtez de jouer !

### "de" + "régénérer" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Arrête de régénérer !

### "de" + "sécurité" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Des agents de sécurité !

### "de" + "́ja" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - As-tu déjà ramassé un champignon vénéneux ?

### "des" + "cousins" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Vous êtes des cousins ?

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

### "deux" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comment fait-on pour séparer son âme en deux, monsieur ?

### "devoir" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Moi et le mot 'devoir' dans la même phrase.

### "dignité" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le turban est ta dignité , préserve-la.

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

### "doc" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Désolé doc , il faut qu'il tienne bon jusqu'à demain matin.

### "douche" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Une douche ?

### "du" + "monde" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Le meilleur attrapeur du monde !

### "décider" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Comment décider ?

### "déjà" + "fugué" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Il a déjà fugué ?

### "démon" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est un démon .

### "dépuce" + "́l" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est vraiment le truc classe à faire quand on a dépucélé une meuf.

### "désastre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un désastre .

### "ec le" + "ur bé" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et ils sont déjà sur place… avec leur bébé !

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

### "en" + "est" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ça en est !

### "encore" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment ça, 'encore' ?

### "enfants" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Les femmes et les enfants ?

### "enfin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - L'aube parut enfin ; la longue nuit s'achevait.

### "enfoiré" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Où est ce fils de pute, enfoiré ?

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

### "entrée" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Comment es-tu entrée ?

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

### "espace" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mec, t'es dans l'espace .

### "est" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ça en est !

### "est" + "long" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Que ce pont est long !

### "est" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "et" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et je descendais l'avenue Libertador, et ..

### "eux" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est arrivé quand vous avez utilisé des mitrailleuses sur eux .

### "expression" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et toujours la même expression, hein ?

### "faim" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il ne peut pas avoir faim ; il vient de déjeuner.

### "faire" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tu me demandes pas ce que je vais faire ?

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

### "feu" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dès que prêts, feu .

### "file" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Suivez la file .

### "fille" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Vous aimez ma fille ; mais êtes-vous sûr qu’elle vous aime ?

### "film" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le film .

### "flattée" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Très flattée !

### "fleurs" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Abeilles cruelles, suçant toute vie de ces pauvres fleurs .

### "foire" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Nous allons à la foire ; viens-tu ?

### "fois" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fais-en beaucoup, on risque de recommencer pas mal de fois .

### "fort" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Plus fort ?

### "fortune" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et ça vaut une fortune .

### "fourvoyé" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Pourquoi me suis-je fourvoyé ?

### "frites" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Des frites ?

### "froid" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quel froid !

### "froid" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il fait froid; j'ai la chair de poule.

### "fruits" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a fait l'andouille et maintenant il en récolte les fruits .

### "frère" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mon frère , je t'en conjure , n'attaque pas les créatures du ciel.

### "frère" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'ai oublié le nom de votre frère; comment se nomme-t-il ?

### "frère" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Son frère ?

### "fugué" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il a déjà fugué ?

### "fulminer" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Rien ne sert de fulminer ; s'indigner suffirait.

### "fête" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Combien de temps êtes-vous restés à la fête ?

### "fête" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Cette salope de la fête ?

### "garce" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La garce !

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

### "gars" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et pas d'erreur les gars , ils sont là bas .

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

### "gauche" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Droite ou gauche ?

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

### "grève" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors on fait la grève .

### "guerre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sur terre ces hommes étaient des bêtes de guerre .

### "gâteau" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ça ne sert à rien de chercher le gâteau; je l'ai déjà mangé.

### "génial" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est génial .

### "génotype" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et comme vous avez le même génotype , vous pourriez enfiler ses chaussures.

### "général" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Pourquoi ne commencez-vous pas, général ?

### "ha" + "»" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et je sais que la posture «ha».

### "harnais" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Enlevez vos harnais .

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

### "hier" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Où sont-ils allés hier ?

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

### "hélicoptère" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On a pris l'hélicoptère .

### "ici" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Les seuls criminels portent les uniformes nazis, ici !

### "ici" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jake c'est la folie ici , tout le monde est sur le pied de guerre.

### "ici" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On le voit d'ici .

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

### "infirmier" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Jarhead est l'opérateur-radio et l'infirmier .

### "inquiéter" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La rencontre inachevée, pourquoi s'en inquiéter ..

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

### "intéressant" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C’est intéressant, non ?

### "investissement" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Votre frère représentait un immense investissement .

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

### "je" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non, madame, je ..

### "jeunesse" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'ai lu beaucoup de livres dans ma jeunesse ; je suis un érudit à ma manière.

### "jouer" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Arrêtez de jouer !

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

### "la" + "fête" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Cette salope de la fête ?

### "la" + "part" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - De la part ?

### "la" + "police" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - De la police ?

### "lapin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il m'a posé un lapin; je l'ai attendu toute la soirée !

### "le" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Débranchez le .

### "leader" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ici leader , nous entrons dans le courant du vortex.

### "les" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Rassemblez les , allons y .

### "les" + "enfants" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Les femmes et les enfants ?

### "libre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'étais libre .

### "lien" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le premier vol scelle le lien , tu ne peux attendre.

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

### "liquide" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Du liquide ?

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

### "loin" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Si loin ?

### "long" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Que ce pont est long !

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

### "lui" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est lui , poursuivez le !

### "lui" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je suis jaloux de lui ; tu l'aimes plus que moi.

### "lui" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qui, lui ?

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

### "là" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tu vois le barbu, là ?

### "là" + "aussi" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Celle-là aussi ?

### "m" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et là, il dit : 'Tape-m'en quatre.'

### "ma" + "main" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Lâche ma main !

### "main" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Lâche ma main !

### "maintenant" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il est trois heures maintenant ; je reviendrai dans une heure.

### "maintenant" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - As-tu du temps, maintenant ?

### "maintenant" + "je" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Avantj'étais amie avec Volodia et maintenantje sors avec Nikita.

### "mais" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il y a un 'mais'.

### "mais" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non mais ..

### "maison" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La lettre que Tom reçut disait qu'il devait rentrer au plus tôt à la maison .

### "maman" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Pas vrai, maman ?

### "manger" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne vis pas pour manger ; je mange pour vivre.

### "marchera" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ça marchera ?

### "marier" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Se marier ?

### "marine" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il n'y a rien de comparable à un ex marine .

### "marines" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Des marines .

### "mascarade" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je préfère un bon combat à toute cette mascarade .

### "matin" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - À quelle heure vous êtes-vous réveillés ce matin ?

### "mauvaise" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La qualité d'image est vraiment mauvaise - la résolution est si basse.

### "maxi" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sous une maxi .

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

### "mec" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Que foutent ces mines ici, mec ?

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

### "mes ?" + "" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce qu'on veut savoir, c'est comment vous en avez abattu un… sans armes ?

### "meurtrier" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un meurtrier ?

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

### "millet" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et des pains au millet ?

### "mission" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Une mission ?

### "moi" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il sait que c'est votre autre 'moi'.

### "moi" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous ne m'aurez pas, moi .

### "monde" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le meilleur attrapeur du monde !

### "monde" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dehors il y a le vrai monde , et le rêve ici.

### "monde" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ca va être un nouveau départ dans un nouveau monde .

### "monsieur" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non monsieur , c'est le courant.

### "montre" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ma montre !

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

### "mourir" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils n'avaient pas à mourir .

### "mourir" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - J'aime, je puis mourir ; j'ai vécu le meilleur et le plus beau des rêves !

### "moustiques" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Des moustiques ?

### "munitions" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ne nous donnera plus de munitions .

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

### "même" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regarde-toi toi-même, épouvantail !

### "nana" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai 'fait une nana' un tas de fois.

### "nana" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quelle nana ?

### "ne" + "veux" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je neveux pas me battre contre vous.

### "neige" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ça allait devenir 'L'année de la grande neige'.

### "nmiolomhobbit pa" + "rler." (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Laisse le… canmiolomhobbit parler.

### "noir" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il a peur du noir ; ne l'y laisse pas.

### "nom" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On grandit bercé par ce nom , mais je n'ai jamais pensé que j'irais un jour.

### "non" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non non non non .

### "non" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Malheureusement non ; au contraire.

### "nourrir" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J’ai une famille moi, ça me fait des bouches à nourrir .

### "nous" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils s'amènent toujours la nuit, et nous, quoi ?

### "nous" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bienvenue à Pandora, content de vous avoir parmi nous .

### "nous" + "jouer" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sommes-nous en colloque ou allons-nousjouer au golf ?

### "ns ar" + "mes ?" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce qu'on veut savoir, c'est comment vous en avez abattu un… sans armes ?

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

### "ongles" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pendant que Cathy se fait les ongles .

### "ordinaire" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Que faites-vous chez toi, d’ordinaire ?

### "osez" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Vous osez !

### "ostéopathe" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Une ostéopathe ?

### "ou" + "gauche" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Droite ou gauche ?

### "oui" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mon opinion professionelle, oui , monsieur.

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

### "pactole" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Visez moi ce gros pactole .

### "paie" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qui paie ?

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

### "parents" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dès que Munni sera avec ses parents ..

### "parlent" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elles vous parlent …euh…d’amour ?

### "parler" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu te rappelles qu'on a parlé de 'parler' ?

### "parlé" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tu lui as vraiment parlé ?

### "part" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - De la part ?

### "partenaire" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un partenaire ?

### "partie" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Quand est-elle partie ?

### "partir" + "jeunes" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mais ils doivent partirjeunes.

### "parviennent" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - S'ils y parviennent , tout sera fini.

### "pas" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous n'allaitons pas - - dans le sein de la faiblesse.

### "passe" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est-ce qui se passe , les gars ?

### "passe" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La vie qui passe; c'est toujours la même chose.

### "passé" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Alors, que s'est-il passé ?

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

### "peigne" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mon peigne !

### "peigne" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ton peigne ?

### "pencher" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Se pencher !

### "percée" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mon frère, je vais faire une percée , suis moi derrière.

### "personnelle" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Cette lettre est personnelle ; je ne veux pas que quelqu'un d'autre la lise.

### "personnellement" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je préfèrerai la lui donner à elle, personnellement ..

### "peu" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La pousser un peu , vous voyez ?

### "peu" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le vrai bonheur coûte peu; s'il est cher, il n'est pas d'une bonne espèce.

### "pieds" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quand je l'ai retrouvée, il manquait un pieds .

### "pignouf" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le vôtre pue du cul, pignouf !

### "piscine" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "plaint" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qui se plaint ?

### "plan" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Quel est le but central de ce plan ?

### "planque" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment dit-on 'planque' en français ?

### "plat" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est stable quand il est maintenu à plat .

### "plaît" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Change de chaîne, s'il te plaît ; cette musique est insupportable.

### "pleut" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regardez comme il pleut .

### "pluie" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils nous pissent dessus en nous faisant croire que c'est de la pluie .

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

### "police" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - De la police ?

### "portable" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le portable !

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

### "pouce" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le pouce .

### "pour" + "jolie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Joli muffin pourjolie madame.

### "pour" + "y" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Feins d'exclure la nana de ton club et elle fera tout poury entrer.

### "pourquoi" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ne demandez pas pourquoi ; faites-le, tout simplement.

### "pourriture" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Regarde-moi dans les yeux, pourriture !

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

### "princesse" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La princesse ?

### "profond" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Très profond !

### "promis" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je lui revaudrai ça l'an prochain, promis .

### "puis" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et puis , tout a changé.

### "puissant" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dieu tout-puissant , qui sait tout.

### "putain" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Arrêtez la voiture putain .

### "pute" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Où est ce fils de pute, enfoiré ?

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

### "que" + "tout" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Plus que tout ?

### "quitté" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'avais deux ans quand le bonheur nous a quitté; maman, papa et moi.

### "quoi" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ils s'amènent toujours la nuit, et nous, quoi ?

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

### "recule" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Tanjiro, recule !

### "refuser" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Alors pourquoi refuser ?

### "rencontrés" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Comment vous êtes-vous tous deux rencontrés ?

### "rendez-vous" + "ici" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Pourquoi ce rendez-vous ici ?

### "rentré" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Comment es-tu rentré ?

### "ressemble" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu leur ressemble , tu parles comme eux et bientôt ils nous feront confiance.

### "rester" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Courir , rester , alors quoi ? !

### "rien" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Mais nous ne savons vraiment rien ; car la vérité se trouve tout au fond.

### "rler." + "" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Laisse le… canmiolomhobbit parler.

### "rock" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Let's rock'n'roll !

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

### "régénérer" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Arrête de régénérer !

### "réincarné" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - En quoi serez-vous réincarné ?

### "réponse" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Comment êtes-vous parvenus à cette réponse ?

### "réussi" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne lui en veux pas d'avoir réussi ; elle a travaillé dur pour ça.

### "sac" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - T'es censé atterrir sur ce putain de sac .

### "sacrifice" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un sacrifice ?

### "sais" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je sais -- Kirill Matféevitch.

### "sais" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je sais .

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

### "savoir" + "jusqu'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Pouvait-on savoirjusqu'où ça irait.

### "savoir-faire" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est quoi une 'marque de savoir-faire' ?

### "scientifiques" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Encore des trucs merdiques de scientifiques .

### "se" + "plaint" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Qui se plaint ?

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

### "senti" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - T'as senti ?

### "serait" + "jamais" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sans lui on se seraitjamais connus.

### "serveurs" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Le bar Zailaiba embauche des serveurs ; es-tu intéressé ?

### "service" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un service ?

### "seul" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Il était tout seul ; pas un chat n'était en vue.

### "shampoing" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un shampoing !

### "signal" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quel signal ?

### "signatures" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Des signatures ?

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

### "solution" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La solution ?

### "sorcière" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Une sorcière ?

### "sort" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quel sort ?

### "sorti" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Comment es-tu sorti ?

### "soudaine" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ton rire est une vague argentée soudaine .

### "souhaits" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Des souhaits ?

### "suis" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis -- Douglas Kelley.

### "suis" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je sais que je te manques, mais je suis ..

### "suis" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Je ne sais pas où je suis ; pouvez-vous m'aider ?

### "suite" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - De suite .

### "suivant" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Plateau suivant .

### "survivront" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Les forts survivront ; les faibles périront.

### "sécurité" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Des agents de sécurité !

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

### "table" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Vous prenez une table ?

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

### "technique" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quelle technique !

### "temps" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si vous le désirez, avec le temps , vous serez en mesure de lui pardonner.

### "temps" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous allons y passer du temps .

### "temps" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Nous ne devons pas perdre de temps ; nous avons quelque chose à faire.

### "tenir" + "jusqu'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ordre de tenirjusqu'au bout, bordel de merde !

### "terre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous venez de la terre .

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

### "toi" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Comment tu sais ça, toi ?

### "tomber" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La Catalogne est sur le point de tomber .

### "toujours" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle vous écoutait toujours .

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

### "tout" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Plus que tout ?

### "traduire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu me ferais l'honneur de traduire .

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

### "transparence" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - La viscosité de ses sécrétions et leur transparence ?

### "traquer" + "Jabba" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Nous allons traquerJabba et le chasseur de primes.

### "travail" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le travail ?

### "travailler" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je n'ai pas envie de travailler; et si nous allions plutôt au cinéma ?

### "travaux" + "avancent" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Les travaux avancent ?

### "traître" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Un traître !

### "trouve" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Cherche ; trouve ; découvre !

### "truands" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Des truands ?

### "trucs" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a encore quelques trucs , je crois.

### "tu" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Que pensais-tu ?

### "tu" + "apporté" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Qu'as-tu apporté ?

### "tuer" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils sont très difficiles à tuer .

### "témoin" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Ton témoin ?

### "une" + "table" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Vous prenez une table ?

### "ur bé" + "bé !" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et ils sont déjà sur place… avec leur bébé !

### "utile" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les livres sont vieux, mais très utile .

### "va" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Comment va ?

### "va" + "Charlie" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment ça vaCharlie ?

### "vacarme" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Quel vacarme !

### "vais" + "faire" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Tu me demandes pas ce que je vais faire ?

### "vide" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Détendez vous et faites le vide .

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

### "ville" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Êtes-vous arrivées en ville ?

### "vite" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Aussi vite ?

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

### "votre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voici le votre .

### "vous" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Souvenez vous , des endroits comme le site que vous venez juste de détruire.

### "vous" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce qui fait de vous … un moins que rien.

### "voyage" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et ce voyage ?

### "vrai" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est pas vrai , vous regardiez votre écran.

### "vrai" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Rien n'est vrai ; tout est permis.

### "vraiment" + "parlé" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Tu lui as vraiment parlé ?

### "vue" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Space
- Examples:
  - Ce n'est pas mon point de vue ; ce n'est que ma traduction !

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
  - C'est appelé « Le ménage.

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
  - Ca se prononce « Strauss ».

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

### "Ça" + "aussi" (1 occurrences)
- Predicted: Space
- Actual: Nbsp
- Examples:
  - Ça aussi ?

### "Ça" + "marchera" (1 occurrences)
- Predicted: Space
- Actual: NarrowNbsp
- Examples:
  - Ça marchera ?

### "État" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que nous ne pouvons pas admettre, c'est qu'un représentant de l'État ..

### "â" + "€™œil" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et tout sera sec en un clin dâ€™œil.

### "ça" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Pour moi ça'a pas d'allure.

### "ça" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'aime ça .

### "ça" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Me regarde pas comme ça; y a rien de tel.

### "école" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Pour le journal de l'école ?

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

### "émancipée" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est une école émancipée .

### "épouvantail" + "!" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Regarde-toi toi-même, épouvantail !

### "étape" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est la derniere étape .

### "éteintes" + "?" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: Nbsp
- Examples:
  - Pourquoi les lumières se sont-elles éteintes ?

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

### "être" + "juste" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je suggère d'alterner dejour en jour pour êtrejuste.

### "île" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est 'île' en espagnol mais à l'envers.

### "–" + "vous" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Asseyez–vous.

### "—" + "fille" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est pas facile, les rapports père—fille.

### "—" + "lui" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Rendez—lui directement.

### "…" + "C'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Youra…C'est pas le moment, je t'assure Youra, c'est pas le moment !

### "…" + "Mais" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Voilà…Mais là, je suis calme.

### "…" + "allez" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Bien…allez-y.

### "…" + "assis" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Hier vous…assis !

### "…" + "bien" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Peut-être, cela explique qu'il soit si…bien nourri.

### "…" + "bonjour" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu peux m'expliquer, quelle…bonjour… quelle mouche t'a piqué hier ?

### "…" + "car" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - FÉLICITATIONS …car aujourd'hui, Oz écrit un nouveau chapitre.

### "…" + "d’" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Elles vous parlent …euh…d’amour ?

### "…" + "et" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et…et où… ?

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

### "…" + "qu'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qu'il…qu'il entre.

### "…" + "tout" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - On doit la transférer …tout ira bien, Fräulein.

### "…" + "une" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Traite la comme…une femme.

### "…" + "voilà" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Servez-vous…voilà, des tomates.

### "…" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bonjour, c'est Novoseltsev… …il crache ?

### "… av" + "ec le" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et ils sont déjà sur place… avec leur bébé !

### "… ca" + "nmiolomhobbit pa" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Laisse le… canmiolomhobbit parler.

### "… sa" + "ns ar" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce qu'on veut savoir, c'est comment vous en avez abattu un… sans armes ?

### "………" + "cinq" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Trente………cinq !

