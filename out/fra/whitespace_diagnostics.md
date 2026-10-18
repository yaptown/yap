# Whitespace Prediction Diagnostics

Total predictions: 4915753
Total errors: 3124
Accuracy: 99.94%

## Error Patterns (sorted by frequency)

### "," + """ (254 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - " Votre nom m'est très familier, M. Holmes, " dit-il en souriant.
  - "Affamé", en anglais, "hunger".
  - "Ainsi, "nul autre que toi ne sera mon héritier !

### "dit" + """ (118 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ainsi il a dit "non".
  - As-tu dit "oui" ?
  - Comment est-ce qu'on dit "good luck" en français ?

### "dire" + """ (83 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Au pas de course", ça veut dire "cours".
  - "Meuh meuh", ça veut dire "vachement".
  - "Surimi" en japonais, ça veut dire "nique ta race".

### "de" + """ (81 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "B" au lieu de "D" et "D" au lieu de "B".
  - "B" au lieu de "D" et "D" au lieu de "B".
  - Ainsi que l'énigmatique sourire de "Mona Lisa".

### "est" + """ (64 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Maintenant, ils savent ce que c'est "d'avoir un zombie comme père.
  - Alors c'est "non" ?
  - Alors le mot gagnant, c'est "cool".

### "pas" + """ (51 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est "pourquoi" qui compte, et non pas "qui".
  - C'est pas "drôle de spécialité".
  - C'est pas "on y va".

### "le" + """ (47 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est le "Koganei Maru".
  - C'est le "new look".
  - C'est moi, le "prochain mari".

### "un" + """ (47 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors, je peux avoir un "A" ?
  - C'est ce qu'on appelle un "clinch".
  - C'est un "Mobile".

### """ + "est" (41 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ah" est une interjection.
  - "Attention" est mon deuxième prénom.
  - "Betty" est juste un surnom.

### """ + "et" (37 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Air" et "Heir" sont homophones entre eux.
  - "Auto" pour "soi-même" et "crate" pour "pouvoir".
  - "B" au lieu de "D" et "D" au lieu de "B".

### "la" + """ (37 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allez à la "Maison de l'Uniforme".
  - Alors qui est prêt pour la "chromagnificence" ?
  - As-tu déjà écouté la "Neuvième" de Beethoven ?

### """ + "de" (26 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "La Montagne magique" de Thomas Mann.
  - As-tu déjà écouté la "Neuvième" de Beethoven ?
  - Basé sur le roman "Dracula" de Bram Stoker.

### "." + "»" (26 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Aimez-vous les gâteaux ?» «Oui, j'aime ça.»
  - «Bon ! dit-il, voilà mon souper.»
  - «C'est un incident, madame, rien de plus.»

### "appelle" + """ (25 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est ce qu'elle appelle "lui voler son eau".
  - C'est pour ça qu'on m'appelle "œil de chaman".
  - Ce n'est pas ce que j'appelle "faire tout notre possible".

### """ + "en" (24 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "D'accord parfait" en dix leçons !
  - "Surimi" en japonais, ça veut dire "nique ta race".
  - C'est "Bonjour" en japonais.

### "par" + """ (24 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est bien ce que j'entends par "relation".
  - Commençons par "Quel est l'essentiel ?"
  - Et c'est ce que voulais dire par "fais-moi peur".

### "!" + "»" (22 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Dis juste «Bonjour !» et ils vont penser que tu es français !
  - «Bon, pensa-t-il, je me figurerai que nous sommes en carnaval !»
  - «Bon ! se dit Passepartout, cela va mal pour les gentlemen du Reform-Club !»

### """ + """ (22 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bilbon m'a dit qu'une grande histoire" "n'avait pas de fin."
  - "C'est la beauté du Sud," "l'élégance de Nankin."
  - "Depuis l'époque antique," "la rivière a coulé avec élégance."

### "mot" + """ (22 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dès que j'entends le mot "bonheur", je tique.
  - Eh bien, je n'aime pas le mot "confession".
  - Elle a parlé du mot "secret."

### "," + "«" (20 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ah, « tu ne verras pas d'ici » ?
  - C'est moi, « L'Élu ».
  - C'est qui, « moi » ?

### "Le" + """ (20 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le "Aventure de la Liberté" est ballotté par le vent !
  - Le "Cavour" de Donnafugata !
  - Le "Cha-Cha" !

### "à" + """ (20 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Andreï t'a appris à "appeler".
  - Ca ressemble à "rat" et "patate".
  - Donc Adèle, tu reprends à "De ce vice".

### "?" + "»" (19 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Aimez-vous les gâteaux ?» «Oui, j'aime ça.»
  - «Cette voiture est-elle à vous ?», demanda la police aux filles.
  - «Combien de temps va durer la réunion ?» «Deux heures.»

### "du" + """ (19 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est l'arbre du "miracle" ?
  - C'est l'auteur de l'article du "Spiegel".
  - C'est quelqu'un du "Deux Moulins" ?

### "et" + """ (19 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Air" et "Heir" sont homophones entre eux.
  - "Auto" pour "soi-même" et "crate" pour "pouvoir".
  - "B" au lieu de "D" et "D" au lieu de "B".

### "les" + """ (18 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors là, faut que tu remplaces tous les "on" par des "nous".
  - C'est comme si vous me demandiez qui a inventé les "blagues de Toto".
  - C'est ça, avec les "amis".

### "»" + "«" (18 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - «Aimez-vous les gâteaux ?» «Oui, j'aime ça.»
  - «Combien de temps va durer la réunion ?» «Deux heures.»
  - «Crois-tu vraiment cela ?» «Je le sais.»

### "des" + """ (17 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors là, faut que tu remplaces tous les "on" par des "nous".
  - Alors, c'est des "lotantiques".
  - C'est des "Nice".

### "une" + """ (17 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Becky est une "elle".
  - C'est quoi une "expérience simulée" ?
  - C'est une "future salope".

### """ + "à" (16 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Après ses "oui oui" à la con ?
  - Demain, je serai nommé "docteur jubilaire" à la cathédrale de Lund.
  - Dis "bonjour" à ta sœur de ma part.

### """ + "ou" (15 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Le bordel est fermé" ou "Nouveau propriétaire" et son nom à elle.
  - "Les trois héros de la police" ou "Le sourire de nos héros" !
  - "Mon colonel" ou "Monsieur".

### "dis" + """ (15 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors, dis "rétameur" !
  - C'est ça, dis "papa".
  - Candidat jeune, tu dis "trop jeune", vieux, "trop vieux".

### "ou" + """ (15 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Le bordel est fermé" ou "Nouveau propriétaire" et son nom à elle.
  - "Les trois héros de la police" ou "Le sourire de nos héros" !
  - "Mon colonel" ou "Monsieur".

### "écrit" + """ (15 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est écrit "urgent".
  - Il a écrit "Connard".
  - Il a écrit "J'ai envie de baiser" !

### """ + "pour" (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Auto" pour "soi-même" et "crate" pour "pouvoir".
  - "Auto" pour "soi-même" et "crate" pour "pouvoir".
  - "H" pour humain et pour hélas.

### "appeler" + """ (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce n'est pas normal de s'appeler "Arbres".
  - Cette famille qui arrête pas de m'appeler "foutu youpin" ?
  - Donc tu peux m'appeler "commandant de bord".

### "pour" + """ (14 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Auto" pour "soi-même" et "crate" pour "pouvoir".
  - "Auto" pour "soi-même" et "crate" pour "pouvoir".
  - As-tu assez de gentillesse pour "lui" ?

### "appelait" + """ (13 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Auparavant l'acide sulfurique s'appelait "huile de vitriol".
  - Elle m'appelait "maman".
  - Entre nous, on s'appelait "les potes".

### "Pas" + """ (12 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pas "Naati".
  - Pas "bichon" ?
  - Pas "cette fois" ?

### "ce" + """ (12 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'était quoi ce "hum" ?
  - Et c'est où, ce "coin pourri" ?
  - Et ce "S" et "R" sont tous deux inversés.

### "ça" + """ (12 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est ça "musubi".
  - C'est ça "musubi, le nœud", le fil du Temps.
  - Comment ça "Wesh Abdel" ?

### """ + "dans" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Almaty" signifie "Le Père des Pommes" dans plusieurs langues turques.
  - Excepté le mot "bite" dans leur nom.
  - Joseph peut dire "Je t'aime" dans quinze langues.

### "," + "p'" (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors, p'tit gars ?
  - Fais pas le malin, p'tit con.
  - Hein, p'tit ?

### "que" + """ (11 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il exige que "Volga" soit averti de sa présence.
  - Je pense que "grande" est une exagération.
  - Je pense que "patate" est le mot le plus amusant.

### "," + "'" (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est qui, 'on' ?
  - Comment ça, 'encore' ?
  - Comment ça, 'nous' ?

### "Et" + """ (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et "Azad" signifie liberté.
  - Et "Katherine Clifton" ?
  - Et "Long Beach Mike" ?

### "Pourquoi" + """ (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pourquoi "Chaussettes" ?
  - Pourquoi "Citron" ?
  - Pourquoi "Mandarine" ?

### "au" + """ (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bienvenue au "Jeu de la Vérité".
  - Donc, nous jouons au "Chaton dans le sac".
  - Il était en train de pisser au "Café Sanborn's" !

### "être" + """ (10 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est une sensation merveilleuse d'être "annulée".
  - D'être "le bouclier et l'épée du parti."
  - En avant pour être "happy".

### """ + "dit" (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - " Votre nom m'est très familier, M. Holmes, " dit-il en souriant.
  - "Ah ouais," dit-elle.
  - "Désolé, je ne sais pas vraiment quoi faire de ça," dit son docteur.

### "Un" + """ (9 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un "Mobile" ?
  - Un "devoir".
  - Un "fermeur" ?

### """ + "ne" (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Absolument" ne l'est pas.
  - "Bienveillante" ne peut être qu'un mot.
  - Le "H" ne se prononce pas en français.

### "Dis" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dis "Bye, maman".
  - Dis "Je suis désolé" ?
  - Dis "Kiki", je t'en prie.

### "Les" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les "Chants" d'Aleardi ?
  - Les "Fighting Owls" en avant !
  - Les "cheese inspectors" nous ont écrasés.

### "dans" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et bienvenue dans "mon" église.
  - Et dans "couleuvre", y'a "cool".
  - Il a doublé une voix dans "Avatar".

### "quoi" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est quoi "Bobbi" ?
  - C'est quoi "Watsu" ?
  - C'est quoi "l'externalisation" ?

### "signifie" + """ (8 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Almaty" signifie "Le Père des Pommes" dans plusieurs langues turques.
  - "Se suicider" signifie "essayer" et "réussir".
  - "Szerelem" signifie "amour".

### "«" + "Je" (8 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Crois-tu vraiment cela ?» «Je le sais.»
  - «Es-tu sûr de vouloir que je te le dise ?» «Je suis tout ouïe !»
  - «Je deviens trop grand pour jouer avec des insectes», dit le garçon.

### """ + "a" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Adieu ma concubine" a une longue histoire.
  - "C'est le nez de mon oncle à l'envers" a-t-il dit.
  - "El Niño de Valencia" a partagé l'affiche avec vous pendant des mois.

### """ + "n'" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Brown dans le mille" n'existe plus.
  - "Le Jugement dernier" n'est-il pas fait pour cela ?
  - "Plaire" n'est pas le mot juste.

### """ + "que" (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est le fameux "Akira" que tu voulais tant rencontrer.
  - C'est le plat chinois "singulier" que je sais cuisiner.
  - Et voici la plus célèbre "débutante" que je voudrais épouser.

### "appelé" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a été porté par Louis XVI, et on l'a appelé "Le Cœur de la mer".
  - Il m'a appelé "Définitif".
  - L'hymne de l’Écosse est appelé "La fleur de l'Écosse".

### "avec" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Pitre" rime avec "huître".
  - Et je devrais affronter le procureur avec "Unité, Corps, Dieu, Patrie" ?
  - Mauricio, qu'est-ce que vous voulez dire avec "il n'y a plus de deal" ?

### "son" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Combien de temps encore avant qu'il n'atteigne son "Ghost" ?
  - Elle en avait marre de son "travail" comme mère au foyer.
  - Et si un cyber-cerveau pouvait générer son "Ghost" ?

### "toujours" + """ (7 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est toujours "non".
  - Il m'appelle toujours "Augustine".
  - Il me dit toujours "Bonjour" quand je le vois.

### """ + "au" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "B" au lieu de "D" et "D" au lieu de "B".
  - "B" au lieu de "D" et "D" au lieu de "B".
  - Il a dit "jour" au lieu de "heure".

### """ + "signifie" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Almaty" signifie "Le Père des Pommes" dans plusieurs langues turques.
  - "Arnaud Ripper" signifie quelque chose pour vous.
  - "Se suicider" signifie "essayer" et "réussir".

### """ + "sont" (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Air" et "Heir" sont homophones entre eux.
  - "Elles" sont jolies.
  - Dites-moi qui "ils" sont.

### "Des" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Des "amis", c'est ce qu'ils disent non ?
  - Des "buns".
  - Des "matzot" !

### "La" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La "caméra cachée".
  - La "gente" princesse ?
  - La "jambe poilue" qui saute, elle aurait du succès à Luanda !

### "Une" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Une "Chevy Impala" la voiture la plus populaire de Californie.
  - Une "Volga" ?
  - Une "balade" ?

### "a" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce rôle t'a "fait" !
  - Ce texte a "Poète" écrit partout.
  - Dans "envie", il y a "vie".

### "ces" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors tu vas en Afrique pour rencontrer ces "Great Hunger" ?
  - Arrête avec tous ces "vous".
  - Goûtez-moi ces "pallies".

### "de" + "." (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ajoutez à cela à nos achats de .
  - Il doit être un temps de .
  - Il est ce que vous faites mieux de .

### "était" + """ (6 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Alors, qui était "Christie Love" ?
  - Elle était donc "l'Indienne", et mon père était "Le beau".
  - J'ai juste dit qu'il s'était "rappelé" de vous.

### """ + "sur" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est pas marqué "banque" sur mon front.
  - Il y a écrit "je crains" sur un papier.
  - J'ai un "cube du destin" sur moi.

### """ + "était" (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Elle" était là.
  - Dehors, cette "Californie" était pluvieuse.
  - Laquelle de ces "techniques imparables" était en cause ?

### "Dans" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dans "Der Spiegel", ouais.
  - Dans "L'Hebdomadaire".
  - Dans "Les Voyages de Gulliver", Swift parle de Laputa, mais c'était une fiction.

### "Quel" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quel "malheur" ?
  - Quel "mec" ?
  - Quel "métier" ?

### "Quoi" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quoi "Fais-moi confiance" ?
  - Quoi "Je sais" ?
  - Quoi "Val" ?

### "a" + "ida" (5 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Elle l'aida elle-même car personne d'autre ne voulait le faire.
  - Elle l'aida à nouer sa cravate car il ignorait comment faire.
  - Il m'aida à porter le sac.

### "appellent" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils s'appellent "Le Peuple des Cendres".
  - Je tue sans exception tous ceux qui m'appellent "cramoisie".
  - Mais maintenant, ils m'appellent "lapin effrayé".

### "comme" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Facile à retenir, comme "Enlacés dans la brise."
  - Il provient de la collection de lord Walpole, comme "La Madone aux perdrix".
  - J'veux pas donner mon bébé à une famille qui se décrit comme "équilibrée".

### "juste" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dis juste "Vande Matram".
  - Ici ils disent juste "véhicule personnel".
  - Je ne suis pas juste "le public".

### "leur" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est leur "exercice incendie".
  - Dis-leur "non".
  - Doug réalisa leur "film" de mariage.

### "moi" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Appelle-moi "Camarade".
  - Appelle-moi "Maman".
  - Peu importe, appelle-moi "Petite".

### "vous" + """ (5 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Connaissez-vous "musubi" ?
  - Et comment demandez-vous "It's going to be nice tomorrow ?"
  - Je voudrais savoir à quoi vous "pense".

### """ + "avec" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bogoss" avec un L.
  - "Va bien" avec le vert.
  - De nos jours, on porte des haut-talons "shoesy" avec une membrane.

### """ + "c'" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Tampopo" c'est bien.
  - "Un baiser et une gifle" c'est bien ça ?
  - Et ici "B" c'est pas "D".

### """ + "ce" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'étais au "Californie" ce soir-là.
  - Je dois "performer" ce soir.
  - On va au "Taksim" ce soir ?

### """ + "comme" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle en avait marre de son "travail" comme mère au foyer.
  - On n'écrit pas "connard" comme ça.
  - Pourquoi tu dis "romantique" comme si c'était un gros mot ?

### """ + "par" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Qu'est-ce que j'entends ?" par Veronica Charles.
  - Alors là, faut que tu remplaces tous les "on" par des "nous".
  - Sauf qu'on a remplacé le mot "amour" par "Noël".

### """ + "quand" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est Amélie qui garde son chat "Rodrigue" quand elle part en voyage.
  - Emploie plus jamais "baiser" quand tu parles de nos femmes !
  - Il me dit toujours "Bonjour" quand je le vois.

### """ + "qui" (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est "pourquoi" qui compte, et non pas "qui".
  - Encore le "R" qui déconne.
  - La "jambe poilue" qui saute, elle aurait du succès à Luanda !

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

### "Ou" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ou "Dieu, as-Tu vu Letterman hier soir ?"
  - Ou "Dieu, si Tu me sors de ce pétrin, j'irai à l'église tous les jours."
  - Ou "Maman chérie" ?

### "cette" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dehors, cette "Californie" était pluvieuse.
  - Mais aussi de cette "Nature morte orientale" !
  - Waouh, elle est énorme cette "Licosus".

### "connais" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu connais "He's my private dancer" ?
  - Tu connais "La Macarena" ?
  - Tu connais "Whiplash" par cœur ?

### "disais" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est pour ça que je disais "danger" !
  - Je me disais "qu'a-t-elle voulu dire".
  - Je me vois appeler là-bas, je disais "Val, c'est ta maman".

### "disent" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils disent "jeune" !
  - Ils disent "notre culture est entre de bonne mains".
  - Les gens disent "les putes n'ont pas de cœur."

### "en" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est bien, il y a de bons mots en "V".
  - Ils viennent toujours en "amis", bien qu'on ne les connaisse pas.
  - La rumeur a gonflé et s'est transformée en "Elle est enceinte".

### "genre" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Du genre "Bonne à baiser" ?
  - Elle posait pour des photos du genre "sang à la une".
  - Le genre "belle-fille idéale".

### "marqué" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est marqué "Trombones".
  - Il est pas marqué "banque" sur mon front.
  - Il y avait marqué "D-FENS".

### "mon" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est "général", mon "innocent".
  - Je peindrai mon "Jugement dernier".
  - Juste un murmure soufflé par mon "Ghost".

### "notre" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cet instinct meurtrier est toujours dans notre "adène".
  - Naturellement, après votre gala de ce soir pour notre "Obergruppenführer".
  - On l'appelle notre "vieille bique".

### "nous" + "'" (4 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est qui 'nous' ?
  - Comment ça, 'nous' ?
  - Qu'est-ce que tu veux dire par 'nous' ?

### "on" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comment dit-on "adieu" en allemand ?
  - Comment dit-on "ami" en elfe ?
  - Comment dit-on "prête-nom" en Angleterre ?

### "opéra" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est l'aria du Baiser de l'opéra "Jose".
  - Il joue le roi dans l'opéra "Adieu ma concubine".
  - Je ne connais pas l'opéra "Le roi gangster et sa pute".

### "terme" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et nous n'employons pas le terme "supplanté".
  - Je préfère le terme "capitaliste de l'aventure".
  - Le terme "cliché" vient de la langue française.

### "ton" + """ (4 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Garde ton "ohé".
  - Mais il n'y aura que ton "Ghost".
  - Oui, c'est ton "pof".

### """ + "Joe Jackson" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Chaussettes" Joe Jackson.
  - "Chaussettes" Joe Jackson !
  - Je lance à "Chaussettes" Joe Jackson !

### """ + "Nouveau Testament" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a eu une douzaine de "nouveaux" Nouveau Testament.
  - Je veux dire, un "nouveau" Nouveau Testament en cent mots.
  - Un "nouveau" Nouveau Testament est sacrilège.

### """ + "demanda" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Combien de clés ?" demanda Pepperberg.
  - "De quoi le professeur a-t-il parlé ?" demanda l'étudiant.
  - "Puis-je vous aider ?" demanda le vendeur.

### """ + "des" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est les Sonic Youth interprétant "Superstar" des Carpenters.
  - Les lignes de la mort sont les "coutures" des choses.
  - Ça m'a quand même "sanctionné" des tendons.

### """ + "dessus" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cherche un signe avec un grand "Un" dessus.
  - Il y a écrit "micro" dessus.
  - Y a écrit "annulé" dessus.

### """ + "du" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Billy's In" du début.
  - Il te montre gueulant "Je suis un dieu doré" du haut d'un toit.
  - Mon petit, je pratique le "kung-fu du feu" du taoïste.

### """ + "l'" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bijou" l'attend.
  - "L'autre" l'a recrachée.
  - Le "non" l'emporte.

### """ + "maintenant" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est "Glinda" maintenant.
  - Je n'ai plus qu'à presser la touche "envoi" maintenant.
  - Parce qu'il y a un "eux" maintenant ?

### """ + "parce" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mon père croyait que je disais "fantôme" parce que j'en avais peur.
  - On l'appelait "la femme qui chante" parce qu'elle chante tout le temps.
  - On les appelle des "rats" parce qu'ils feraient tout pour survivre.

### """ + "qu'" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est la recette "casse mâchoire" qu'elle m'envoie, je la connais.
  - Et ce "gynécée" qu'est-ce que c'est ?
  - Le titre de "roi" qu'ils nous ont donné ?

### """ + "s'" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Au meurtre !" s'écria la malheureuse en poussant un cri perçant.
  - Notre "festoyée" s'embête.
  - Ou alors, la "mère" s'imposait totalement.

### """ + "se" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Bon," se dit-il, "Ça explique l'odeur, du coup."
  - Je n'aurais jamais cru qu'elle voulait "vraiment" se reposer.
  - Quand on était petits, les "compares" se rendaient visite.

### """ + "tout" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est votre accent et vous dites "che" tout le temps.
  - Et arrête de dire "OK" tout le temps !
  - Tu le fais intentionnellement en disant "toi" tout le temps.

### """ + "va" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce "travail" va finir par un mariage.
  - Et maintenant "Il Matto" va nous présenter son attraction la plus périlleuse !
  - Le film du "Non" va perdre de l'élan dans quelques jours.

### """ + "veut" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Moi et moi et moi" veut savoir.
  - En grec, "Iya" veut dire "violette".
  - Et comme nous savons tous, "fourrer" veut dire "bluffer".

### """ + "ça" (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "PommaChina" ça s'appelle.
  - "Timide" ça veut dire la même chose.
  - C'est-y vrai qu'ils ont sorti un nouvel album, "PommaChine" ça s'appelle ?

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

### "." + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Et il vécut heureux jusqu'à la fin de ses jours. "
  - "Il était propre et ses blessures soignées. "
  - "Ils arrivent. "

### "Ce" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce "je vous en prie" m'est resté en mémoire.
  - Ce "paradis" n’existe pas.
  - Ce "travail" va finir par un mariage.

### "Définissez" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Définissez "empêché".
  - Définissez "instable".
  - Définissez "vivant", Stuart.

### "J" + "‘" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J‘ai fêté mon anniversaire dans un restaurant.
  - J‘aime celui-ci.
  - J‘aime les soupes avec de nombreux végétaux.

### "Mais" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais "Arbre" !
  - Mais "capitaine", ça marche.
  - Mais "grosse vache", c'était vilain.

### "Sa" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sa "Schildkröte" est morte.
  - Sa "douleur" ?
  - Sa "mère" !

### "as" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regarde, t'as "bateaux".
  - Tu l'as "cafté" pour quoi ?
  - Tu l'as "cafté" ?

### "bouton" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Chérie, ce bouton "envoyer", c'est comme un bouton de missile.
  - Enfant, appuyez simplement sur le bouton "supprimer" ..
  - Ou j'appuie sur le bouton "envoyer".

### "can" + "'t" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - But I can't spend my life at the laundromat.
  - The holiday feast, we can't miss that.
  - The holiday feast, you can't miss that.

### "cet" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - NOù est cet "Akira" ? !
  - Quelqu'un a-t-il réellement vu cet "Akira" ?
  - Tu files un dollar à cet "azupep".

### "chose" + "." (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je crois qu'il te manque quelque chose .
  - Je ne l'ai jamais vu quelque chose .
  - Patientez, encore une chose .

### "d" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce tableau est considéré comme un chef-d'œuvre.
  - Je me casse d'ici.
  - Je veux que tu sortes d'ici.

### "disait" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bouddha disait "L'absence de passion est la meilleure des vertus."
  - Elle disait "Plus de monde, plus de monde."
  - Et la boulette de pâte Pillsbury disait "Mangez-moi quand vous voulez".

### "disant" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'elle, je n'ai que cette apparence, disant "il fait beau".
  - Il sortit précipitamment du taxi, en disant "gardez la monnaie".
  - Tu le fais intentionnellement en disant "toi" tout le temps.

### "dites" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est votre accent et vous dites "che" tout le temps.
  - Quand vous vous adressez au Duc dites "Monseigneur" ou "Sire".
  - Vous dites "hello" !

### "en" + "fuir" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Elle a essayé de s'enfuir.
  - M'en aller, m'enfuir, me terrer quelque part !
  - Pourquoi ne pas s’emparer du sac et s’enfuir ?

### "entendre" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ils vous serrent et vous avez l'occasion unique d'entendre "vos os se briser.
  - Et dans dix secondes, je vais entendre "elle fait que passer".
  - J'ai cru entendre "tous les quatre".

### "expression" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce fut durant cette période qu'on inventa l'expression "Vive la révolution."
  - Rick, vous est-il déjà arrivé d'entendre l'expression "Le client est roi" ?
  - Vous connaissez l'expression "bouse de vache".

### "ferait" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça ferait "L", ça ferait "l", ça ferait "X".
  - Ça ferait "L", ça ferait "l", ça ferait "X".
  - Ça ferait "L", ça ferait "l", ça ferait "X".

### "il" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ce n'est pas : 'Où est-il' ?
  - Qui 'il' ?
  - Qui ça, 'il' ?

### "là" + "." (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bonne chose que je suis passé par là .
  - Hé, chef, regardez qui est là .
  - Seul un miracle pourrait le sortir de là .

### "mots" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Depuis l'autre côté de la pièce elle forma les mots "je t'aime" avec les lèvres.
  - Les mots "phase finale" ne cessent de me revenir à l’esprit.
  - Quand je dirai les mots "Bien mûr", arrange-toi pour nous laisser seuls.

### "on" + "'" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est qui, 'on' ?
  - Comment ça, 'on' ?
  - Qui ça, 'on' ?

### "plus" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne dirai plus "madame", uniquement "toi".
  - Je voulais faire plus "dame".
  - Nous ne pouvons même plus "prier".

### "qui" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est qui "nous" ?
  - Dites-moi qui "ils" sont.
  - Il y a encore des gens vieux jeu pour qui "mon amie" laisse entendre homo.

### "suis" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Finalement, tu l'as dit, je suis "vieux" !
  - Je suis "dés-holé".
  - Je suis "public relations" à la satisfaction des intéressés.

### "sur" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle travaille sur "Où que tu sois."
  - T'as hésité sur "mariage".
  - Écrivez une rédaction sur "L'amitié".

### "tes" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Arrête tes "oui oui".
  - Tiens, Kumail, pour tes "files".
  - Épargne-moi tes "eh bien" !

### "votre" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Euh, qu'y a-t-il pour votre "sévice" ?
  - Faites-lui donc une analyse du sang, à votre "femme".
  - Il pirate votre "Ghost" et vous manipule.

### "«" + "Bon" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Bon, pensa-t-il, je me figurerai que nous sommes en carnaval !»
  - «Bon ! dit-il, voilà mon souper.»
  - «Bon ! se dit Passepartout, cela va mal pour les gentlemen du Reform-Club !»

### "«" + "Et" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Eh bien, ton frère est vraiment beau !» «Et moi, je suis un cageot ?»
  - «Et pour Monsieur ?» «Allez ! Une blanquette de veau !»
  - «Et qu'est-ce que c'est que ça ?» «Ce sont tes photos.»

### "«" + "J'" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Il va pleuvoir ?» «J'espère que non.»
  - «J'aime voyager.» «Moi aussi.»
  - «J'irai à pied», dit Phileas Fogg.

### "«" + "Pas" (3 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Pas de chance !» pensa-t-il.
  - «Pas un geste, ou vous êtes perdu», était-il écrit.
  - «Ça te dérange pas si je fume ?» «Pas le moins du monde.»

### "étais" + """ (3 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comme je n'étais "personne", j'étudiai l'épée.
  - May disait que j'étais "cool".
  - Que j'étais "normal".

### "…" + "de" (3 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - De quoi…de champignons.
  - Et ce Borovskikh……de l'industrie locale ?
  - Tu peux ramasser un panier entier de souches…euh…de…d'armillaires.

### """ + "Graham" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On doit trouver "Moonlight" Graham "On" ?
  - Êtes-vous "Moonlight" Graham ?

### """ + "appartement" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Cet" appartement est situé sur la plage de Boa Viagem.
  - "Cet" appartement, figure-toi, c'est là que vous avez tous grandi.

### """ + "d'" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est la graine authentique de la "cucurbita" d'Asie.
  - Les "Chants" d'Aleardi ?

### """ + "disait" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Comment ?" disait le pasteur, "avec un communiste ?"
  - "Il reste toujours une chance" disait-il.

### """ + "ici" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elles s'appellent toutes "Missy" ici.
  - Tu parles "French" ici !

### """ + "il" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai regardé "Star Wars" il y a quelques mois.
  - Je faisais des cornichons hier quand il est venu "Pourquoi autant ?" il demande.

### """ + "la" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dis"s'il te plaît" la prochaine fois.
  - Une "Chevy Impala" la voiture la plus populaire de Californie.

### """ + "m'" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce "je vous en prie" m'est resté en mémoire.
  - Que le "vrai pouvoir" m'appelle s'il a envie d'un magazine.

### """ + "mais" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je lui ai dit mille fois "Je t'aime" mais jamais "Je suis désolé".
  - Pas "hm hm" mais travailleur… acharné.

### """ + "me" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "L'Arnaqueur Volant" me rend fou.
  - "Voleur" me semble plus exact.

### """ + "quoi" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Comprendre" quoi ?
  - "Quelque chose" quoi ?

### """ + "sans" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle ne serait même pas capable de dire "bonjour" sans mon aide !
  - Et surtout d'avoir dit "oui" sans réfléchir.

### """ + "t'" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et ses techniques "faibles" t'ont fait gagner les Nationales.
  - Le "châtiment des piments" t'attend.

### """ + "te" (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ton "Ghost" te l'a murmuré ?
  - Tu crois qu'être "cool et drôle" te donne tous les droits ?

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

### "?" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Miss Kelly, connaissez-vous le sens du mot parjurer ? "
  - "Personne ne le sait ? "

### "Allez" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Allez; allons-y.
  - Allez; rentrons à la maison.

### "Au" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Au "Deux Moulins".
  - Au "je reçois les témoignages", là les sanglots l'étouffent.

### "Ces" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ces "choses" n'entreront pas.
  - Ces "fêtes" spontanées semblent se passer partout.

### "Comme" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comme "Love Thing".
  - Comme "Sugar Babies".

### "De" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - De "Sabor".
  - De "terry cloth" ?

### "Frères" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est donc le célèbre 'Livre des Frères' ?
  - C'est donc le fameux 'Livre des Frères' ?

### "Il" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il "gargles" et va au lit tout seul.
  - Il "reprenait des forces".

### "Je" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je "choisissez".
  - Je "fiche" de ce que pensent les autres.

### "Jeff" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voix de Jeff "Bebay", Jeff "Babey".
  - Voix de Jeff "Bebay", Jeff "Babey".

### "Nation" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Avant la première de 'La Fierté de la Nation'.
  - Le film s'appelle 'La fierté de la Nation'.

### "Ni" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ni "bordel", ni "merde" pendant toutes ces années.
  - Ni "sacré".

### "Par" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Par "on", je ne pensais pas toi !
  - Par "très volontaire", je voulais dire "très coordonnée".

### "Pour" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pour "Shô", l'idéogramme avec le mouton, et pour "ta", l'idéogramme "épais".
  - Pour "normalement", j'ai compris.

### "Quelle" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quelle "carte" ?
  - Quelle "occasion" ?

### "Rechercher" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Rechercher "Nicole Raven".
  - Rechercher "stock manquant".

### "Tu" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu "nous" as fait honte.
  - Tu "peux pas" ?

### "a" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est ce qu'il y a , Beto ?
  - Qu'est ce qu'y a , baltringue ?

### "abord" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'abord "Société Légalement Autonome", ensuite le parti "Gadar".
  - D'abord "qui", ensuite, "pourquoi".

### "affaire" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bein, l'affaire "Lazlo" est toujours en cours.
  - Le dernier rapport de l'affaire "Lazlo".

### "appelaient" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils m'appelaient "Father Time".
  - Un vent rouge, que les marins appelaient "l'Océan des Ténèbres".

### "appelais" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dire que je l'appelais "tonton".
  - Tu m'appelais "baby".

### "appellera" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je vais faire un film qui s'appellera "Les salades de l'amour".
  - On s'appellera "La Vague".

### "arrivé" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne savais pas ce qui est arrivé .
  - Peux pas vous dire ce qui est arrivé .

### "aussi" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et t'as aussi "vachement meuh meuh" !
  - Ça, c'est aussi "musubi".

### "autre" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a une maison, un appartement, un autre "chez lui" ?
  - Non, je ne veux pas d'un autre "Carandiru".

### "cela" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le bouddhisme appelle cela "Araya-Shiki".
  - Vous appelez cela "contrôler" ?

### "chanson" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La chanson "Love Thing".
  - Siri, lance la chanson "La brume" de Chung Hoon-hee.

### "chante" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle chante "Carmen".
  - T'attends que je chante "Starsky et Hutch" pour démarrer ?

### "cheese" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dis 'cheese'.
  - Dites 'cheese'.

### "chienne" + "," (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne peux pas obtenir une chienne , voilà pourquoi.
  - Non engataste pas chienne , toute la nuit ?

### "comprends" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu comprends "light" ?
  - Tu comprends "problèmes" ?

### "crie" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et tout le monde crie "Hourra !"
  - Une d'elle écrase le frein et me crie "Monte bel étalon !"

### "célèbre" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cette connasse, c'est la célèbre "Sorcière".
  - Et voici la plus célèbre "débutante" que je voudrais épouser.

### "devenir" + "Jedi" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le fils de Skywalker ne doit pas devenirJedi.
  - Pourquoi veux-tu devenirJedi ?

### "diffuser" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Désormais, on va diffuser "Salut, vieux" cinq fois par semaine.
  - Tu viens de diffuser "Lettre triste" de Yoo Jae-ha.

### "dise" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elles adorent qu'on leur dise "petite fille".
  - Que le témoin dise "Monsieur le Juge" ou "Monsieur le Président".

### "dit" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et le soleil dit; "Bonne journée, cher Erich."
  - Le sage sait ce qu'il dit; le sot dit ce qu'il sait.

### "dossier" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Albert Jerska, le dossier "Engerling".
  - Fin du dossier "Lazlo".

### "enlève" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'accord, on enlève "vraiment".
  - On enlève "vraiment" ?

### "entendu" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai bien entendu "putain de ta mère".
  - Tu l'aurais entendu "chprécher".

### "entre" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis resté coincé entre "carnage" et "massacre".
  - Mais j'avais encore du mal à comprendre le lien entre "nom" et "sens".

### "faire" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il faut faire "expiation".
  - Tu vas faire "ouin-ouin".

### "fais" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Moi, je fais "le tour du monde sans quitter Pékin".
  - Pourquoi tu fais "mm-mm" ?

### "fait" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai dit que papa a appelé tu as fait "Hum".
  - Tu n'as rien dit, tu as fait "hum".

### "fait" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle veut savoir ce qu'elle a fait .
  - Voir exactement ce que m'a fait .

### "fameux" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est le fameux "Akira" que tu voulais tant rencontrer.
  - Ce corps de femme est un fameux "bifteck" !

### "famille" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dans la famille "opérateur", la mère.
  - Dans la famille "électricien", le grand-père.

### "garage" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Take it easy, aller à votre garage .
  - Tous vos trucs dans le garage .

### "grand" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cherche un signe avec un grand "Un" dessus.
  - Les explosifs se sont déclenchés avec un grand "Bang".

### "i" + "I" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Mais qu'iI finisse ce qu'iI a commencé.
  - Mais qu'iI finisse ce qu'iI a commencé.

### "inscrit" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dans les faits, il est inscrit "Appelez votre mort".
  - On nous a dit qu'il est inscrit "Appeler les morts" ?

### "jamais" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Emploie plus jamais "baiser" quand tu parles de nos femmes !
  - Je lui ai dit mille fois "Je t'aime" mais jamais "Je suis désolé".

### "joue" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui joue "la Sandunga" ?
  - Un gramophone joue "Cheek To Cheek".

### "jouer" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle voulait jouer "The Qin Huai View" et elle est morte pour ça.
  - On va plutôt la jouer "cool" que venir, là.

### "l" + "'" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il a fait un sans-faute à l'examen.
  - On a fini les travaux juste avant l'hiver.

### "look" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je voudrais bien voir le look "maquillé".
  - Rien que pour un look "naturel".

### "marine" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il n'y a rien de comparable à un ex marine .
  - Mais pas avec une simple paie de marine .

### "merde" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ceci est une grande merde .
  - Vous êtes-vous plein de merde .

### "moi" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pourrait également être plus amusant pour moi .
  - Prenez-le et d'attente pour moi .

### "mort" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Le frère livrera son frère à la mort "et le père, son enfant.
  - "Tout n'est que froideur, mort "et solitude dans cette famille.

### "même" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est même "L'Espion".
  - Ça m'a quand même "sanctionné" des tendons.

### "nom" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et le nom "Jésus" ?
  - J'aime bien le nom "Garçon qui pleure".

### "nous" + ";" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Juste entre nous; êtes-vous amoureuse de ma sœur ?
  - Juste entre nous; êtes-vous amoureux de ma sœur ?

### "pas" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ne va pas .
  - Je sais qu'elle ne me plaît pas .

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

### "préfère" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je préfère "Bobbi", M. Euclides.
  - Je préfère "mari actuel".

### "regardé" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a trop regardé "La Chute".
  - J'ai regardé "Star Wars" il y a quelques mois.

### "ressemble" + "." (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - En fait c'est lui qui te ressemble .
  - Je lui ressemble .

### "sa" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'emmerde Herzog et sa "motivation" !
  - Mon ami est venu me chercher dans sa "Volga".

### "serait" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La réponse serait "non".
  - Pour l'Histoire, elle serait "l'incoulable Molly Brown".

### "sont" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elles sont "bio".
  - Mais tes artistes préférés sont "nègres".

### "souvent" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a un homme qui demande très souvent "Lettre triste".
  - Je dis trop souvent "divers", Chris.

### "style" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et son pseudo-jargon militaire style "Code rouge".
  - Il fait style "il a rien vu".

### "suite" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ici, il faut se garder de dire tout de suite "j'achète".
  - Phuljhariya, emmène-le à la suite "lune de miel".

### "ta" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ne m'appelle pas ta "duchesse".
  - À ta "rotation" ?

### "the" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - In the "baba".
  - The girl is at the "bich".

### "toi" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Est-ce qu'elle se serre contre toi "et caresse tes tétons ?
  - C'est toi "on dirait maman".

### "trouve" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle le trouve "décoratif".
  - Mais Donald trouve "arrière-grand-père" vexant.

### "tu" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Depuis quand es-tu "Mlle Gennero" ?
  - Es-tu " celui qui soupire " ?

### "voici" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - En attendant, voici "Roobaroo."
  - Marquis, voici "La Naissance de Jean-Baptiste".

### "voir" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'essaie de voir "Génération perdue".
  - Pour aller voir "Le Bal des sirènes."

### "vos" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et virez-moi vos "monsieur".
  - Et épargnez-moi vos "monsieur".

### "vu" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai vu "Soleil du Désert".
  - La nuit dernière, j'ai vu "celui qui soupire."

### "«" + "Allez" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Et pour Monsieur ?» «Allez ! Une blanquette de veau !»
  - «Je peux vous poser une question ?» «Allez-y.»

### "«" + "C'" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «C'est un incident, madame, rien de plus.»
  - «C'est à huit heures et demie que vous comparaîtrez devant le juge Obadiah.»

### "«" + "Cette" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Cette personne».
  - «Cette voiture est-elle à vous ?», demanda la police aux filles.

### "«" + "Dieu" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Dieu ! Qui élit ces gens ?» «Dieu, qui élit ces gens.»
  - «Dieu ! Qui élit ces gens ?» «Dieu, qui élit ces gens.»

### "«" + "Eh" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Eh bien, se dit-il, puisqu'il n'est pas son complice, il m'aidera.»
  - «Eh bien, ton frère est vraiment beau !» «Et moi, je suis un cageot ?»

### "«" + "Est" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Est-ce qu'il vient ?» «Non, je ne pense pas.».
  - «Est-ce que ça va si je me joins à vous ?» «Naturellement !»

### "«" + "Le" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Le nouveau domestique», dit-il.
  - «Le pavillon en berne», dit Mr. Fogg.

### "«" + "Passepartout" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Passepartout.»
  - «Passepartout», reprit Mr. Fogg sans élever la voix davantage.

### "«" + "Que" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Que se passe-t-il ?», murmura une voix depuis l'autre bout de la pièce.
  - «Que voulez-vous sur votre pizza ?» «Tout sauf des anchois.»

### "«" + "Tu" (2 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Tu aimeras ton prochain comme toi-même » est une citation de la Bible.
  - «Tu me trouves comment ?» «Très jolie.»

### "écrire" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il sait à peine écrire "ranger".
  - Je t'ai dit d'aider Pepe à écrire "Tu me manques beaucoup, Papa".

### "émission" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je vais mettre Hitler dans l'émission "Salut, vieux !".
  - Votre émission "Caméra Actuelle".

### "été" + """ (2 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La sonorité a toujours été "John".
  - Mais tu as toujours été "le roi" ?

### "…" + "vous" (2 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je…je…vous plains sincèrement.
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "!" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Pas le choix ! "

### """ + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Enfant, appuyez simplement sur le bouton "supprimer" ..

### """ + "C'" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Est là, n'est pas là" C'est quoi ces absurdités !

### """ + "Demanda" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "La vie peut-elle exister sans péché ?" Demanda le vieux philosophe.

### """ + "Eddie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le seul et unique Clifton "Fou Fou" Eddie à la batterie !

### """ + "Ho Po-Wing" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Repartir à zéro" Ho Po-Wing disait toujours ça.

### """ + "Joe" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ne me l'a pas dit "Chaussettes" Joe ?

### """ + "Joe Wood" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Y avait trop de volontaires, il a fallu se battre "Smokey" Joe Wood !

### """ + "Ködörösêse tî Bêafrîka" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La République Centrafricaine s'appelle " Ködörösêse tî Bêafrîka " en sango.

### """ + "Le" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "L'Éducation Pour Tous" Le régime donne à chaque enfant ce droit.

### """ + "National Zoo" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui veut aller au " National Zoo " demain ?

### """ + "On" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Prêteur fantôme et Poubelle" On dirait des pseudos sur Internet.

### """ + "Oui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Avenue Lénine." Oui.

### """ + "Qureshi" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Hitler" Qureshi !

### """ + "Un" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - " Un jeu stupide."

### """ + "Va" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Romi" Va te faire foutre !

### """ + "Votre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - " Votre nom m'est très familier, M. Holmes, " dit-il en souriant.

### """ + "Wilson" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Stanley "L'Engin" Wilson.

### """ + "Wu" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que veut-il dire par "recherche" Wu ?

### """ + "accepté" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je proclame le "public safety bill" accepté en tant que loi.

### """ + "ai" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Est-ce qu'il vit ?" ai-je demandé.

### """ + "amants" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Mes" amants ?

### """ + "anéantit" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Une femme qui a des couilles" anéantit déjà toute galanterie.

### """ + "appela" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Viens, petit," appela-t-elle,"viens jouer".

### """ + "après" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça me plaît, un peu de "poésie" après souper.

### """ + "as" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu "nous" as fait honte.

### """ + "aujourd'hui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai déclenché le plan "Clausewitz" aujourd'hui.

### """ + "aurait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Tout va bien" aurait suffi.

### """ + "autour" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu vois les "top" autour du feu ?

### """ + "aux" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pourquoi est-ce qu'on appelle l'automne "fall" aux États-Unis ?

### """ + "avant" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nous" avant "moi".

### """ + "bien" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que nous voulons, "c'est du poisson" bien goûteux !

### """ + "brouillent" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les "lunettes" brouillent leur radio pour nous semer.

### """ + "buter" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ton père non plus, il "voulait pas" buter le mien.

### """ + "car" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Shiki est "yin" car c'est une femme.

### """ + "celui" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es-tu " celui qui soupire " ?

### """ + "cinq" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Désormais, on va diffuser "Salut, vieux" cinq fois par semaine.

### """ + "confirme" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Salut, vieux" confirme tout ce que j'en ai entendu.

### """ + "dedans" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a "team" dedans, pour "équipe tactique du narcotrafic".

### """ + "demain" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui veut aller au " National Zoo " demain ?

### """ + "demande" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Capitaine à Nazareth ?" demande la prof.

### """ + "devint" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La "mère" devint jalouse et tua la jeune fille.

### """ + "dirait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Les vieilles filles brillent devant nous" dirait le poète !

### """ + "décrit" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'opéra "Adieu ma concubine" décrit le combat entre les rois Chu et Han.

### """ + "encore" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous répétiez "dix millions" encore et encore.

### """ + "faire" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Emmène "La Petite" faire un tour.

### """ + "flingue" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est "mon" flingue.

### """ + "garçon" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tout le monde disait que c'était un "beau" garçon.

### """ + "hurla" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ne me fais pas peur" hurla-t-elle alors qu'il l'approchait par derrière.

### """ + "interprétée" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Lettre triste" interprétée par Yoo Jae-ha.

### """ + "laisse" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il y a encore des gens vieux jeu pour qui "mon amie" laisse entendre homo.

### """ + "leider" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Verboten" leider sous peine de la mort.

### """ + "là" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "You, you go" là-dedans.

### """ + "malade" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ich bin" malade.

### """ + "maître" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ich bin" maître de l'hôtel.

### """ + "monde" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Notre" monde, espérons.

### """ + "montre" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Une "petite" montre !

### """ + "murmura" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Tais-toi" murmura-t-il.

### """ + "nous" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les "cheese inspectors" nous ont écrasés.

### """ + "n’" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce "paradis" n’existe pas.

### """ + "obéir" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nous" obéir ?

### """ + "officier" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Es ist ein" officier allemand ?

### """ + "ouvert" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dossier opérationnel "Lazlo" ouvert, contre Georg Dreyman, nom de code "Lazlo".

### """ + "partout" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il veut planter des "lotantiques", des "lotantiques" partout !

### """ + "pas" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eh bien en fait, il ne "dit" pas vraiment quelque chose.

### """ + "pendant" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ni "bordel", ni "merde" pendant toutes ces années.

### """ + "piano" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais pas "les" pianos, putain, "du" piano.

### """ + "pianos" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais pas "les" pianos, putain, "du" piano.

### """ + "plus" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est ce que tu "dois" plus que ta famille ?

### """ + "police" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Belles" police classique, stature, force.

### """ + "pourrait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Der Spiegel" pourrait faire une revue dessus.

### """ + "pousse" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le "Jaune d'œuf" pousse là-bas.

### """ + "prend" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Morriss" prend deux "s".

### """ + "princesse" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La "gente" princesse ?

### """ + "proches" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La vache, vous êtes vraiment, "vraiment" proches Greg et toi.

### """ + "près" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Y a écrit "sécurité" près de la sécurité.

### """ + "rend" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La devise "œil pour œil" rend tout le monde aveugle.

### """ + "rendez-vous" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un enterrement est "le" rendez-vous mondain.

### """ + "ressemble" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le sexe du "bufeo" ressemble à celui de la femme.

### """ + "revient" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Petit", "petite" revient de façon récurrente dans l'œuvre.

### """ + "rime" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Pitre" rime avec "huître".

### """ + "répondit" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Pour toujours et à tout jamais !" répondit le petit lapin blanc.

### """ + "sait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Homme De Guerre" sait qui elle est maintenant.

### """ + "sale" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui t'appelle "Papi" sale punk ?

### """ + "sera" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu crois que le Piémontais, le "Roi-Galant-Homme" sera différent ?

### """ + "soit" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il exige que "Volga" soit averti de sa présence.

### """ + "sort" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Superman" sort ce mois-ci au cinéma.

### """ + "sound" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Did you hear the linking "z" sound ?

### """ + "spontanées" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ces "fêtes" spontanées semblent se passer partout.

### """ + "tous" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On les appelle "Le Chagrin et la Pitié" tous les deux.

### """ + "trop" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis arrivé au "Californie" trop tôt.

### """ + "un" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a vu dans le "palantir" un aperçu du plan de l'Ennemi.

### """ + "vexant" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais Donald trouve "arrière-grand-père" vexant.

### """ + "vie" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'aimerais que quelqu'un me dise quoi faire de "cette" vie.

### """ + "vient" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le terme "cliché" vient de la langue française.

### """ + "voiture" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ta" voiture.

### """ + "voulait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pour la première fois, j'ai compris ce que "meilleur ami" voulait dire.

### """ + "écrit" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce texte a "Poète" écrit partout.

### """ + "église" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et bienvenue dans "mon" église.

### ""+"" + "ressemble" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le "+" ressemble au symbole de la division, alors je suis indécise.

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

### "," + "“" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Try to say, “I need a tart.”

### "-" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Donc, je descendais la rue et cette voix m'a dit - "Bonsoir, M. Dowd" !

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
  - Pas toi, bien sûr ; tu es une femme - enfin presque.

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

### ";" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et le soleil dit; "Bonne journée, cher Erich."

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

### "Anderson" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Un dur, ce ' Bonne nuit Anderson', alors fais gaffe.

### "Attends" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Attends "être enterré vivant" ?

### "Ayrton" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Une fois, j'ai demandé à Ayrton "Quand penses-tu arrêter de courir ?"

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

### "Car" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Car "Mieux vaut juger l'ennemi plus puissant qu'il ne semble".

### "Chang" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je m'appelle Chang "Sabre Ailé".

### "Chante" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Chante "Rêvant du Monde".

### "Chasseur de juifs" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qu'on vous appelle le 'Chasseur de juifs'.

### "Chatur" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On appelait Chatur "Silencieux".

### "Chuchotements" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Chuchotements .

### "Clifton" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le seul et unique Clifton "Fou Fou" Eddie à la batterie !

### "Comment" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comment "il n'est pas là" ?

### "Compagnie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les hôtesses sont des artistes de la Compagnie "Cinq continents".

### "Confirmez" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Confirmez .

### "Conyers" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je t'interdis d'appeler Mr. Conyers "Keith".

### "Coupez" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Coupe pas tant que je dis pas 'Coupez'.

### "DISTRICT FÉDÉRAL" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - DISTRICT FÉDÉRAL "Costa et Carpinheiro", en quoi puis-je vous aider ?

### "Davy" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous venez d'appeler Davy "Daisy" ?

### "Depuis" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Depuis "Pandi panda", bof.

### "Deux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Deux "field" pour le prix d'un.

### "Disons" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Disons "Jan".

### "Dites" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dites "Longue vie à la Corée !"

### "Doc" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qui vous a dit qu'on l'appelle 'Doc' ?

### "Désolé" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Désolé ..

### "ELLE" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - COMMENT VA-T-ELLE …par le simple mouvement de l'air.

### "ENFUIT" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - IL S'ENFUIT "Très gentille."

### "Emmène" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Emmène "La Petite" faire un tour.

### "Encore" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Encore "fermé pour lubie" ?

### "Essayez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Essayez "Nouvelle Chancellerie du Reich".

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

### "Fait" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Fait "Ah !"

### "Franchement" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Franchement ..

### "FÉLICITATIONS" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - FÉLICITATIONS …car aujourd'hui, Oz écrit un nouveau chapitre.

### "GLOBO" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Reginaldo LEME COMMENTATEUR TV GLOBO "C'est fini."

### "Genre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Genre "les plus grands héros du monde".

### "Graham" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On doit trouver "Moonlight" Graham "On" ?

### "Great Indian peninsular railway" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Voici, en somme, le tracé à grands points du «Great Indian peninsular railway».

### "Gretchen Ross" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai trouvé 'Gretchen Ross' assez cool.

### "HOMME" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - VOUS VOLEZ LE POULET D'UN HONNÊTE HOMME "Quelqu'un, s'il vous plaît."

### "HYÈNES" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - DES HYÈNES "Quelqu'un, s'il vous plaît."

### "Humains" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ou créer des esprits libres, et les appeler 'Humains'.

### "Hôtel" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hôtel "Atlantic".

### "Immense" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Immense "Licosus".

### "Impossible" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Impossible; il n'y a pas d'homme assez mince pour s'introduire par là.

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

### "Joue" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Joue "Mystification".

### "Juste" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Juste "Borden, Borden, Borden", quand nous savons tous que c'est Strauss.

### "Ködörösêse tî Bêafrîka" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La République Centrafricaine s'appelle " Ködörösêse tî Bêafrîka " en sango.

### "La Taverne" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - T'iras peut-être à 'La Taverne'.

### "Le" + ""+"" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le "+" ressemble au symbole de la division, alors je suis indécise.

### "Li Yun" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Li Yun "Puma Ailé".

### "Lily" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ça s'appelle 'L'océan de Lily'.

### "Lis" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lis "Le ménage".

### "Londres" + "”" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Peut-être qu'un jour tu finiras par venir à Londres ”pour une de tes missions.

### "Loyal" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et le second prénom, ce sera 'Loyal' ?

### "Là" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Là "stop".

### "M" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je croyais qu'on avait choisi 'M' au hasard.

### "Madame" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Madame "Tout le monde", comme tu me l'as demandé.

### "Major en chef" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Édito du Major en chef "Sommes-nous des souris ou des hommes ?"

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

### "Marqué" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Marqué "divers".

### "Merde" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Merde ..

### "Mes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mes "matzot" sont arrivés ?

### "Mettez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mettez "Juif", là.

### "Mi" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je m'appelle Mi "Bras d'acier".

### "Monsieur" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est quoi alors, Monsieur "Je sais tout" ?

### "Monsieur" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regardez, Monsieur -- un droïde.

### "Montons" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Montons "Maison de poupée".

### "National Zoo" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui veut aller au " National Zoo " demain ?

### "Ni" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Les chevaliers qui disent 'Ni' exigent un sacrifice.

### "Notez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Notez "enculé".

### "Notre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Notre "festoyée" s'embête.

### "O." + "C." (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Précédemment dans The O.C.

### "Objectif" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Objectif "Reine de la Méditerranée" en vue !

### "On" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On "moi, mon gosse", j'suis pas con.

### "Ou" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ou'est-ce que c'est ?

### "Oui" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Oui Oui ..

### "Papa" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Papa "caboche" ?

### "Passe Partout Fort Boyard" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Passe Partout Fort Boyard , fini.

### "Passepartout" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Passepartout», reprit Mr. Fogg sans élever la voix davantage.

### "Pharmacie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pharmacie "Cicanuk".

### "Point" + "trait" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Point trait point point.

### "Que" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que "Papa" n'était pas mon père.

### "Quelques" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quelques "tepus", ouais !

### "Quels" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quels "trois" ?

### "Qui" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui "ils" ?

### "R" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - T'aurais pas un 'R' à me prêter ?

### "Ruby" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Je dédie ce livre à ma fille, Ruby "et à mes deux fils.

### "Sans" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sans "A", c'est plus amusant.

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

### "Spot" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Spot "Mme Yolita".

### "Stanley" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Stanley "L'Engin" Wilson.

### "Sung" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Moi, je suis Sung "Aigle de Fer".

### "T-shirt" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois porter un T-shirt "Désolé, Arjun" ?

### "Ta" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ta "Voix" est trop vague.

### "The" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - The "faser".

### "Titanic" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On appelait le Titanic "le Vaisseau des rêves".

### "Ton" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ton "Ghost" te l'a murmuré ?

### "Tous" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tous—Phileas Fogg excepté sans doute—sentaient leur cœur battre d'impatience.

### "Trois" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Trois "désolé" et fin du match.

### "Trop" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Trop "dangerous".

### "URGENCE" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - MESSAGE D'URGENCE … s'est divisé à l'infini pour se répandre sur le globe.

### "Urban Legends" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Notre prochaine fonctionnalité est «Urban Legends».

### "Verrazano" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On appelle le pont Verrazano "passerelle de Guinea".

### "Vieux Garçon de Greer County" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'était 'Le Vieux Garçon de Greer County'.

### "Vis" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vis "blanc".

### "Voilà" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voilà "L'Addiction."

### "Votez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Votez "Non".

### "Vénus" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Certaines personnes surnomment Vénus "la sœur de la Terre".

### "X" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Place un 'X' à l'endroit opportun.

### "abandonnée" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Maria Goretti, seule et abandonnée; un destin de femme.

### "accusations" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ces accusations "me laissent froid.

### "activité" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Excellente activité .

### "ah" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dites 'ah'.

### "ai" + ""intenté" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai "intenté" à ma vie pendant mon séjour à l'hôpital pour les emmerder.

### "aimes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quand tu dis que tu aimes "Fever Dog", ça, c'est le putain de pied !

### "air" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mexès cul et vous aurez l'air .

### "alerte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - État d'alerte "Clausewitz".

### "allergie" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je vais prendre une allergie .

### "alliez" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois vous dire une chose avant que vous alliez .

### "alors" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je t'appellerai alors "grand-mère".

### "ambitieux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Elle tomba aux mains de généraux ambitieux "et de bureaucrates véreux.

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
- Actual: None
- Examples:
  - Nous sommes nés de l'amour; l'amour est notre mère.

### "américain" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Cela parut «très américain» à Passepartout.

### "anglais" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça vient de l'anglais "shoes", qui veut dire "chaussure".

### "annonce" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'enseigne dehors annonce "Lou's Tavern".

### "annulée" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est une sensation merveilleuse d'être 'annulée'.

### "années" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Franchement, nous il ya à peine quatre années .

### "appela" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je n'avais pas de nom, on m'appela "Sans Nom".

### "appelez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que vous appelez "amour" est sordide en comparaison.

### "appeller" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous venez de m'appeller "Coltrane" ?

### "appelles" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est ça que tu appelles "pas long" ?

### "apprenti" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - En vous quittant, j'étais l'apprenti; maintenant, c'est moi le maître.

### "après" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais après "Papa Don't Preach", j'ai décroché.

### "après" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous assurer immédiatement après .

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

### "arme" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ne s'agit pas d'une arme "connue de tous".

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

### "assurance" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si on appelait l'assurance "fin de vie au chaud" ?

### "attendent" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voyons ce que nous nous attendent .

### "au" + ""Pacha" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis allé au "Pacha".

### "aujourd'hui" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que le bouton qui s'ouvre aujourd'hui "aura bientôt trépassé."

### "autant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pourquoi dire autant "Pepsi" ?

### "automne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pourquoi est-ce qu'on appelle l'automne "fall" aux États-Unis ?

### "autres" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne vois pas d'autres "ruines".

### "avait" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a dit qu'il avait "quelque chose à annoncer", lui aussi.

### "avant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nous" avant "moi".

### "avare" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il les aime âprement, comme un avare; jalousement, comme un amoureux.

### "avoir" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Hermann Göring vous semble avoir "l'imagination fertile" ?

### "baignent" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On appelle le liquide dans lequel ils baignent "l'eau des limbes."

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

### "battre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Y avait trop de volontaires, il a fallu se battre "Smokey" Joe Wood !

### "berne" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Le pavillon en berne», dit Mr. Fogg.

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

### "billet" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Bien sûr, je acheté le billet .

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

### "bâtards" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Merde, vous êtes un bâtards .

### "bébé" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai dis 'oh oui bébé', viens voir.

### "bébé" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je résous en un instant, bébé .

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
- Predicted: Nbsp
- Actual: None
- Examples:
  - Alors il lui a acheté un cadeau: ce chaton.

### "caramel" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un caramel "lapin blanc".

### "cas" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous, monsieur, êtes ce qu'on appelle un cas "Grenade".

### "cause" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «La première cause», dit-il.

### "ceci" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il lui faut du repos et ceci - à prendre trois fois par jour.

### "certain" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a été empoisonné par un certain "Frelon".

### "chanter" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - À la demande du marié, je vais chanter "J'ai besoin d'aimer".

### "chanter" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ne dit rien à propos de chanter .

### "chantes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu chantes "Approche-toi" "Approche-toi" à l'intention de Chaddaji ?

### "chantonne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il chantonne "La Macarena".

### "chasse" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Papa est parti à la 'chasse'.

### "chat" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est Amélie qui garde son chat "Rodrigue" quand elle part en voyage.

### "chat" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ma mère sait à peine compter ou épeler 'chat'.

### "chateie" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que personne dans la maison de chateie .

### "chaton" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle a appelé le chaton "Jaguar".

### "chiennes" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils sont venus ici pour baiser quelques chiennes .

### "chinois" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est le plat chinois "singulier" que je sais cuisiner.

### "chip" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je fais un 'chip' au-dessus de l'eau.

### "choisir" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On peut vraiment choisir "détraqué" ?

### "chose" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quelque chose "Venta Pilar".

### "chose" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je pense que les hommes ne veulent qu'une chose , ça y est.

### "chose" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois vous dire une chose ..

### "chose" + ":" (1 occurrences)
- Predicted: Nbsp
- Actual: None
- Examples:
  - Interrogez-vous sur chaque chose: quelle est son essence ?

### "cigares" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça doit être une boîte de cigares "Virginia", vide.

### "cirque" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et aujourd’hui, le numéro de Gala, au célèbre cirque "Medini" !

### "citerne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dans le sens ville un camion citerne "Lait".

### "clairement" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a dit clairement "fourre".

### "classés" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tous leurs cyborgs sont classés "secret".

### "clowns" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils sont même un clowns .

### "code" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dossier opérationnel "Lazlo" ouvert, contre Georg Dreyman, nom de code "Lazlo".

### "codes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vos codes "sacramentaux".

### "cola" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pendant un moment, j'aimais vraiment le cola - j'en buvais tous les jours.

### "collé" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu as copié-collé "jungle" dans le GPS ?

### "combinaison" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Une combinaison "Tout type".

### "combinaison" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'ai lu un «Traité des serrures de combinaison» qu'on lui attribue.

### "comme" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que je ressens, je me sens comme .

### "communautaire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous avons une auto-protection communautaire .

### "complètement" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Frank Poupart, complètement "schizonoïaque" !

### "concentrerai" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Permettez-moi de me concentrer, je me concentrerai .

### "conclusion" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Sa conclusion "dépend donc de lui.

### "concours" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Participez à notre concours "À qui appartient ce visage" !"

### "confiance" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Je vous fais confiance "en me disant que vous respecterez ce bonheur-là.

### "confond" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle confond "serviette" et "cuillère".

### "connu" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous avez vraiment connu "la femme qui chante" ?

### "constamment" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous devez arrêter de dire constamment "non" à Tom.

### "contexte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'essayais de le replacer dans le contexte "La Russie n'est pas l'Allemagne".

### "contrat" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - M'a offert un contrat .

### "coup" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lampes Effacement à coup ..

### "courriel" + ">" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Envoyez-nous votre CV détaillé à <courriel>.

### "crient" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne veux pas de gens qui crient "embrassez la mariée".

### "crier" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il venait de crier "Touu vaa bien", et on s'était sentis en sécurité.

### "croyais" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je me croyais "choisi".

### "cul" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voyons remodeler votre petit cul .

### "cœur" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Criez de tout votre cœur "Vive Marie !"

### "cœur" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le meilleur remède pour le cœur - c'est la bonne vieille aspirine.

### "de" + ""Monte-Cristo"" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - As-tu poursuivi ta lecture de "Monte-Cristo" ?

### "de" + ""Tasogare-doki" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce mot est la racine de "Tasogare-doki", qui veut dire "crépuscule".

### "des" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est donc normal pour eux l'utilisation des .

### "destin" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - J'ai un 'cube du destin' sur moi.

### "deux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Morriss" prend deux "s".

### "devenir" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça allait devenir "L'année de la grande neige".

### "devient" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Après, on devient "soldat" et ça devient sérieux.

### "devins" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je devins "l'homme du moment".

### "devise" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La devise "œil pour œil" rend tout le monde aveugle.

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

### "dirai" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je te dirai "ch-chut" et tu te tairas.

### "dirait" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est enculé, on dirait "La Chose".

### "dire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois quelque chose à dire .

### "directement" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quand il vient, allez-vous demander directement .

### "dirent" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les policiers leur dirent "Stop".

### "dirige" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le doyen a pris sa retraite et dirige "l'hôpital de Munna".

### "dis" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Faites ce que je dis; ne faites pas ce que je fais.

### "dises" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Maintenant, il faut que tu dises "jeune femme".

### "disons" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous disons "mettre au monde".

### "dit" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Donc, je descendais la rue et cette voix m'a dit - "Bonsoir, M. Dowd" !

### "docilement" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "N'entre pas docilement "dans cette douce nuit.

### "dois" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois "performer" ce soir.

### "domestique" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Le nouveau domestique», dit-il.

### "donc" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle était donc "l'Indienne", et mon père était "Le beau".

### "donner" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le simple fait est que je ne peux te donner "pour toujours".

### "dont" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai ici quelques livres, dont "La Vie de Pétrarque".

### "doute" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tous—Phileas Fogg excepté sans doute—sentaient leur cœur battre d'impatience.

### "doutent" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Les Américains doutent "de la victoire des États-Unis au Viêt-nam.

### "drogue" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous connaissez la drogue "cheese" ?

### "du" + ""Koganei Maru" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non, il est le capitaine du "Koganei Maru".

### "dure" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Combien de temps dure "pour toujours" ?

### "désastre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un désastre .

### "désencombré" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - En quarante minutes, le salon fut «désencombré» selon l'expression d'Arsène.

### "désormais" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'abord, sachez que le projet s'appelle désormais "Aquarius".

### "détruire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je, je vais vous détruire .

### "effet" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le deuxième acte s'appelle 'l'effet'.

### "effrayé" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cri effrayé "C'EST LE LOUP !"

### "encore" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Depuis tout petit, quand il s'appelait encore "Petit Dé".

### "encore" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment ça, 'encore' ?

### "enfant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je veux appeler mon enfant "annulé".

### "ennemis" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pardonnez toujours à vos ennemis - rien ne les agace plus que cela.

### "entend" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On entend "God Save the Queen".

### "entendez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si par "bonne", vous entendez "riche", oui, c'est la seconde fortune d'Italie.

### "entends" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle dit "Merci beaucoup", mais j'entends "Vas te faire foutre".

### "entrer" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous savez, je ne vais pas entrer .

### "es" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - T'es "corsico".

### "est" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ramasser des champignons c'est …c'est merveilleux…vous savez.

### "explique" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça explique "Les molles" de Dali.

### "fabuleux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Louis XVI portait le fabuleux "diamant bleu de la Couronne".

### "factures" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Avec elle, vous pouvez payer vos factures , pharmacie et tout.

### "faire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que je pouvais faire .

### "fait" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il s'est excusé, mais le mal était fait; après, c'était trop tard.

### "faits" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Momo, se joint à la présente les faits .

### "famille" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Arrêtez de parler dans ma famille .

### "faut" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - On l'appelle le 'jeune homme comme il faut'.

### "façon" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sur l'enregistrement, il ne le fait pas de façon "fragile".

### "femme" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pas toi, bien sûr ; tu es une femme - enfin presque.

### "fenêtre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Finalement, la fenêtre "Téléchargement du fichier terminé" est apparue.

### "fils" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu as appelé ton fils "Tom".

### "fois" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je lui ai dit mille fois "Je t'aime" mais jamais "Je suis désolé".

### "fois" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais en fait, il était la première fois .

### "fredonne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle fredonne "Petite Musique de nuit".

### "frimer" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il va frimer "Moi, je fais pas la guerre !"

### "froid" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il fait froid; j'ai la chair de poule.

### "frère" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mine est ainsi, mon frère .

### "frère" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'ai oublié le nom de votre frère; comment se nomme-t-il ?

### "gant" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mon mot préféré en allemand est le mot pour «gant».

### "garanties" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous voulons d'autres garanties "qui ne sont pas dans l'accord."

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

### "gauche" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et il est venu sur la gauche .

### "gauche" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je ne sens plus mon pied gauche; je n'y ai plus de sensation.

### "gens" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quand on dit aux gens "Sonny Hayes", ils disent pas que t'es un has been.

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

### "googlé" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai juste googlé "thérapie par l'art" et improvisé le reste.

### "gouvernement" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ensuite, qu'est-ce qui constitue le 'meilleur gouvernement' ?

### "groupe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous connaissez le groupe "The The" ?

### "gueulant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il te montre gueulant "Je suis un dieu doré" du haut d'un toit.

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
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et je sais que la posture «ha».

### "hang" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - L'ivresse du «hang» leur avait été probablement épargnée.

### "haut-talons" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - De nos jours, on porte des haut-talons "shoesy" avec une membrane.

### "heure" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Monsieur Pu Yi et sa famille ont une heure "pour quitter la Cité interdite.

### "heureuse" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Dis-toi que je suis une femme heureuse; j’ai trouvé ma place.

### "heureux" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je serais très heureux .

### "hier" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je l'ai envoyé hier; tu devrais le recevoir demain.

### "homme" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Madame, le mariage de votre fille avec cet homme "est pire qu'une erreur.

### "homme" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Dites juste 'conséquences pour la santé de l'homme'.

### "honnêtement" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Me coupe bientôt, je vous le dis honnêtement .

### "horrible" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Pas spécialement horrible; non, non, pas du tout.

### "huit" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «L'examen se tiendra aujourd'hui en huit», dit le professeur.

### "ici" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et ici "B" c'est pas "D".

### "ici" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je savais que je te trouverais ici .

### "idéogramme" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Pour "Shô", l'idéogramme avec le mouton, et pour "ta", l'idéogramme "épais".

### "il" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ton père non plus, il "voulait pas" buter le mien.

### "ils" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Qui ça, 'ils' ?

### "important" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le télé-évangéliste a un public important - et bercé d'illusions.

### "impressionné" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis désolé, je suis un peu impressionné .

### "inquiète" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Accord, mais elle est inquiète .

### "insectes" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Je deviens trop grand pour jouer avec des insectes», dit le garçon.

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

### "interprétant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est les Sonic Youth interprétant "Superstar" des Carpenters.

### "invités" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voilà pourquoi vous nous avez invités .

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

### "jaguar" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le jaguar "sauvage" ?

### "jouent" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Des musiciens jouent "Ce n'est qu'un au revoir".

### "jouera" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quand se jouera "Une nuit dans la chambre nuptiale" ?

### "juifs" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Alors tu es 'Le chasseur de juifs'.

### "karaoké" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il chante dans des soirées karaoké , où il chante mal.

### "la" + ""Pensione Fiorini" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Enfin, je suis descendue à la "Pensione Fiorini".

### "laid" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ancien, si possible, laid .

### "laisser" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je mentais et je crains de me laisser .

### "lancer" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Lui lancer "Abruti" au visage.

### "lapin" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Il m'a posé un lapin; je l'ai attendu toute la soirée !

### "le" + ""Jardin des Modes" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Paparazzo voulait te faire faire un reportage pour le "Jardin des Modes".

### "lettre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ils ont réparé la lettre "A" ?

### "limites" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sans limites …méchante sorcière.

### "linking" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Did you hear the linking "z" sound ?

### "lion" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais vous pouvez appeler une tel lion "bien entraîné", pas "bien éduqué".

### "lire" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Sur le disque, on entend Marlon Brando lire "L'Homme nu".

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

### "lisez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quand vous lisez "POMME, votre esprit crée l'image d'une pomme."

### "longs" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tes pantalons sont trop longs - tu vas marcher dessus.

### "lui" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je vous appelle et vous restez avec lui .

### "lune" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les gens d'ici appellent la plus grosse lune "la main de Dieu".

### "là" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Erina est là; il faut que je sorte.

### "magazine" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est quelque chose pour le magazine "Der Spiegel".

### "mahout" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le Parsi connaissait parfaitement le métier de «mahout» ou cornac.

### "maintenant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et maintenant "Il Matto" va nous présenter son attraction la plus périlleuse !

### "mais" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nous", c'est bien, mais "mangeons", au passé, ça donne quoi ?

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

### "maisons" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les maisons "sont toutes pareilles."

### "majuscule" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La lettre majuscule "Q".

### "manger" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle m'a regardé comme si je voulais manger .

### "mangés" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il doit être le temps ou sont mangés , pensez !

### "marines" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Des marines .

### "martiaux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Tous les arts martiaux "développent les qualités physiques.

### "mauvaise" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La qualité d'image est vraiment mauvaise - la résolution est si basse.

### "me" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu me "laisses passer" ?

### "me" + "ure" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et maintenant, ils attendent tous qu'à mon tour, je meure.

### "menu" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si tu vas au restau, y a pas sur le menu "chien".

### "merci" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je voudrais dire «merci» à Tom.

### "merveilleux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le merveilleux "Magicien d'Oz."

### "messieurs" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Me voici, messieurs», disait-il.

### "mettez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ben mettez "bureaulier".

### "minutes" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Encore cinq minutes», dit Andrew Stuart.

### "moderne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Changé en grec moderne "Iesous".

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

### "moi" + "“" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - "Ton bébé de moi“, ben non.

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

### "municipale" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est la police municipale .

### "musique" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'écoute toujours de la musique; je ne peux pas vivre sans.

### "mère" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Dis à ta mère "ce que tu jugeras bon.

### "mélangeant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La personne salua Bill, ce à quoi Bill répondit en mélangeant "Tu vas bien ?"

### "méthode" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous connaissez la méthode "Ogino" ?

### "métro" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il habite près de la station de métro "Rotes Tor".

### "mêmes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Un jour, les mensonges tomberont d'eux-mêmes "et la vérité triomphera.

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

### "ne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eh bien en fait, il ne "dit" pas vraiment quelque chose.

### "neige" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Ça allait devenir 'L'année de la grande neige'.

### "ni" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ni "bordel", ni "merde" pendant toutes ces années.

### "nomme" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je le nomme "Mauvaises Nouvelles", et elles font mauvais hôte.

### "nomment" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ces vieux conseillers me nomment "sans valeurs".

### "nommé" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Demain, je serai nommé "docteur jubilaire" à la cathédrale de Lund.

### "non" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La réponse devra être « non . »

### "nous" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est le meilleur métier du monde pour nous .

### "nouveaux" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tyler inventait de nouveaux "devoirs" et les distribuait sous enveloppe scellée.

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

### "numéro" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne peux pas donner mon numéro .

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

### "octave" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Ces cordes donnent la quinte et l'octave», dit Mr. Fogg.

### "océan" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Comme si j'étais de retour au grand océan "De Po Ji".

### "opération" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je me pose la question tous les jours depuis l'opération "Fiasco".

### "opérationnel" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dossier opérationnel "Lazlo" ouvert, contre Georg Dreyman, nom de code "Lazlo".

### "oubliées" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Des vertus oubliées "qui se rejoignent et se reflètent.

### "oui" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Elle hochait la tête pour dire oui; et je restais couchée.

### "où" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je parle au capitaine 'je-ne-sais-d'où'.

### "pacte" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un pacte "si je m'en sors".

### "pacte" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - La première partie s'appelle 'le pacte'.

### "paie" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Je paie», dit ce gentleman.

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

### "parler" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voulez-vous vraiment parler , vous ?

### "parles" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu parles "French" ici !

### "paroles" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Je me suis trompée dans les paroles "des chansons.

### "part" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Toutes les émissions à la télé à part "J'aime Lucy" ne parlent que de meurtres.

### "parti" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'abord "Société Légalement Autonome", ensuite le parti "Gadar".

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

### "patrouille" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Hier soir, une patrouille "est tombée dans une embuscade.

### "payés" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est pour ça qu'ils sont payés .

### "peine" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Passepartout, tout honteux, répétait que «ça n'en valait pas la peine» !

### "peinture" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Patron, où est cette peinture "La fille se lavant les cheveux" ?

### "pensais" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais je pensais "il me joue peut-être un tour".

### "penser" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu dois penser "plus grand" que ta famille.

### "perdu" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Pas un geste, ou vous êtes perdu», était-il écrit.

### "personne" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Voilà ce qu'est une personne "économiquement viable".

### "personne" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Cette personne».

### "petit" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et, rien que ce petit "Allô".

### "petite" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous ne voulez pas aller à des gonzesses, vous êtes petite , ils vous aimeront.

### "petite" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Regardez, imaginez que vous êtes petite .

### "petites" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est à qui les petites "noreilles" ?

### "peu" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un critique les trouvait un peu "européen".

### "peu" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le vrai bonheur coûte peu; s'il est cher, il n'est pas d'une bonne espèce.

### "peur" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allez, nous commençons à avoir peur .

### "peut-être" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et vous vous demandez peut-être "Comment ont-ils fait ça ?"

### "photos" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois rien à voir avec les photos .

### "piano" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On dit qu'il joue du piano "hot".

### "pied" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «J'irai à pied», dit Phileas Fogg.

### "piscine" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "plage" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Musulmans, le soleil, la plage , chameaux.

### "plan" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai déclenché le plan "Clausewitz" aujourd'hui.

### "planque" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Comment dit-on 'planque' en français ?

### "plaît" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ne fais pas ça, s'il vous plaît .

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

### "police" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Parce que vous êtes une bonne police , Bernard !

### "polka" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Polka, polka, polka "Twin Lakes Polka" ?

### "pont" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Ils se sont emparés du pont "et de la deuxième salle.

### "pont" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Sur le pont», qu’est-ce que vous allez foutre ?

### "porno" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il était Rocco Sieffredi, l'acteur porno .

### "porte" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ouvre la porte , OK ?

### "porte" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eh bien, je vais briser la porte .

### "possible" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Vous avez oublié deux mots très importants 'si possible'.

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

### "pourquoi" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et pourquoi "de toute façon", pourquoi tu réagis comme ça ?

### "poussins" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ne perdez pas de temps avec ces poussins .

### "pouvoir" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le temps n’est pas un «pouvoir».

### "premier" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dans le premier "Les Griffes de la nuit", qui meurt en premier ?

### "première" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous pourriez dire que est une première .

### "presse" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et là, moi, je compte jusqu'à trois et je presse "envoi".

### "problème" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais je dois un problème .

### "promis" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je lui revaudrai ça l'an prochain, promis .

### "propre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu douterais de ton propre "Ghost" ?

### "propre" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Qu'il soit sympa, un peu rangé, et propre; peint en blanc.

### "précision" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Vos jours de fécondité sont inscrits avec précision "sur mon petit calculateur.

### "préférais" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je préférais "papa".

### "préféré" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais nous avons préféré "Aquarius".

### "prépariez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ça fait un bout de temps, quand vous prépariez "Sabor", vous et Salvador Malio.

### "prétendu" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Que dis-je de ce "jour de deuil national" ce prétendu "jour de deuil national" ?

### "prêt" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est correct, je suis prêt .

### "puce" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Parce que je suis une puce .

### "punition" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ceci est une sorte de punition .

### "putain" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Arrêtez la voiture putain .

### "pute" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Allez au diable, fils de pute .

### "père" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - As-tu honte d'appeler ton père "Papa" ?

### "père" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - En fait, je vis avec mon père .

### "père" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est pas facile, les rapports père—fille.

### "qu" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Un bonjour sincère vaut mieux qu'un long discours.

### "quantité" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Était seulement une petite quantité .

### "que" + "je" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tu veux queje vienne ?

### "qui" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il pourrait arriver à n'importe qui .

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

### "recette" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est la recette "casse mâchoire" qu'elle m'envoie, je la connais.

### "regarder" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On aimait regarder "Les trois Stooges".

### "regardez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous regardez "Un monde ordinaire".

### "rendra" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que vous trouvez "impossible", elle le rendra "possible".

### "restauré" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La Cinémathèque a restauré "Sabor".

### "resterai" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Je resterai», dit-il.

### "retard" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dépêchez-vous, nous allons être en retard .

### "retourné" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le petit n'a pas retourné , avez-vous ?

### "retrouver" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On n'essaie plus de retrouver "notre Claire".

### "rivière" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les gens du coin nomment cette rivière "la mangeuse d'hommes" et la craignent.

### "roi" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est toi qui as dit 'roi'.

### "roman" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Basé sur le roman "Dracula" de Bram Stoker.

### "rouge" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Et son pseudo-jargon militaire style 'Code rouge'.

### "route" + "—" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Aucun d'eux ne se plaignait de la longueur de la route—, pas même le mort.

### "rubéole" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'amour, c'est comme la rubéole - tout le monde doit en faire l'expérience.

### "récitent" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Les femmes récitent "Je vous salue Marie".

### "répondez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - En silence, répondez "présente" à l'appel de votre nom.

### "répètes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tu répètes "Fuir la nuit" ?

### "répétiez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous répétiez "dix millions" encore et encore.

### "résoudre" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous sommes ce que nous avons un problème à résoudre .

### "rêves" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je rencontrai le gars de vos rêves .

### "rôle" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - En jouant ce rôle "loyalement", j'espère gagner son cœur.

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

### "sale" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Remonter le pantalon, mon cochon sale .

### "santé" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le corps est petite mais pleine de santé .

### "sauveur" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et dès l'abord on traita «notre sauveur» comme un ami d'ancienne date.

### "savoir-faire" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est quoi une 'marque de savoir-faire' ?

### "savouré" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Je n'ai pas encore pleinement savouré "sa souffrance.

### "section" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - La section "nettoyage".

### "seins" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - L'un avec les gros seins .

### "sera" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Entre nous, ce sera "dent pour dent, œil pour œil".

### "seraient" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Quelles seraient "nos affaires" ?

### "ses" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Après ses "oui oui" à la con ?

### "seul" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Rapide, je ne suis pas seul .

### "siffle" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elle siffle "Petite Musique de nuit", de Mozart.

### "sifflera" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il sifflera "Matinada fiorentina".

### "signe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le dernier devra porter le signe "âne" sur son dos.

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

### "société" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On crée la société "Bagrov Frères".

### "soldat" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Quand on monte l'échelle, on trouve le 'soldat'.

### "sors" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'où tu sors "gentil" ?

### "soupire" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Es-tu " celui qui soupire " ?

### "sourire" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est un sourire "joyeux anniversaire".

### "sous" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Enregistré sous "Hello Kitty".

### "spectaculaire" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il est une technique spectaculaire .

### "suis" + "--" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je suis -- Douglas Kelley.

### "sujet" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ne pas parler à ma mère à ce sujet .

### "supporter" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce sont en mesure de supporter .

### "surnom" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ton surnom "Mark" ?

### "surnommait" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On la surnommait "La Sorcière".

### "surnomme" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On le surnomme "Stups" dans son dos.

### "séparer" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Swimmy leur apprit à ne jamais se séparer "et à ne jamais quitter leur poste.

### "sérieux" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et cela ne me prend pas au sérieux .

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

### "tambours" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Des tambours "dans les profondeurs.

### "tant" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Eh ben, tant "pire" !

### "tapes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Dans "site", tu tapes "URL".

### "tapette" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Le dernier mot qu'il a entendu était 'tapette'.

### "tard" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Jamais trop tard», dit-il.

### "tasse" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Je passe après 'tasse', 'piscine' et 'girafe'.

### "taïnos" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Surpris, des enfants taïnos "sont dissimulés dans les herbes.

### "technicien" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un professeur, et un coach de chant et un technicien "Watsu".

### "techniques" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Et ses techniques "faibles" t'ont fait gagner les Nationales.

### "temps" + "»" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mais «ce beau temps» était passé.

### "terminé" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - J'ai terminé "L'Addiction".

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

### "timide" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Oh regarde, la timide "Licosus".

### "tires" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - D'où tu tires "Darlene" ?

### "toi" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Sauf que 'toi' ne veut plus rien dire.

### "toi" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Tous étaient partis par toi .

### "toi" + ":" (1 occurrences)
- Predicted: Nbsp
- Actual: None
- Examples:
  - Souviens-toi: ouvre le avant de le manger.

### "touche" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je n'ai plus qu'à presser la touche "envoi" maintenant.

### "tour" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Nanny m'a enfermée à double tour "à cause de ma sortie de la veille.

### "tourmentée" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Une âme tourmentée "errant entre la vie et la mort, "sans issue."

### "tours" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Les enfants adoraient me jouer des tours; et je présume que Dieu aussi.

### "tous" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous m'appeliez tous "Julika le gros tas".

### "tous" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a raison, nous avons tous .

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

### "toutes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Elles s'appellent toutes "Missy" ici.

### "trait" + "point" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Point trait point point.

### "tranquillement" + "," (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je me suis assis sur le canapé, tranquillement , à contrôler.

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

### "trouver" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On doit trouver "Moonlight" Graham "On" ?

### "trouvez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que vous trouvez "impossible", elle le rendra "possible".

### "tu" + ""dois" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qu'est ce que tu "dois" plus que ta famille ?

### "uniquement" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je ne dirai plus "madame", uniquement "toi".

### "va" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - S'il ne mange pas, il va "clamser".

### "venir" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Peut-être que je n'aurais pas dû venir "mais j'en avais tellement envie !

### "venu" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je faisais des cornichons hier quand il est venu "Pourquoi autant ?" il demande.

### "verre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Verse-moi un autre verre "roi des Doms" !

### "vers" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous allons vers "Livorno", comme on dit ici.

### "vidéos" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Le monde entier envoie des vidéos "mauvais numéro".

### "vies" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Est le cas de nos vies .

### "visage" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il a été frappé par votre visage "lumineux et vif".

### "vissés" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il me semble que nous sommes vissés .

### "vivifie" + ";" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - La solitude vivifie; l'isolement tue.

### "vivre" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est pour vivre "blanc" et ne plus commettre de péché, qu'on mange du tofu.

### "vivre" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - En espagnol, ça veut dire 'Je te laisse vivre'.

### "voir" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Cela n'a rien à voir .

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

### "vont" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ni vu, ni connu, ils vont "prendre l'air" !.

### "voter" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous avons tous une raison de voter "Non".

### "votre" + ""ouste" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Très féroce, votre "ouste".

### "voulait" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je n'aurais jamais cru qu'elle voulait "vraiment" se reposer.

### "voulez" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je dois ce que vous voulez .

### "vous" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous ne voulez pas des gars comme vous .

### "vous" + "…" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce qui fait de vous … un moins que rien.

### "vrai" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Si c'est un vrai "Ghost", je le trouverai.

### "vraie" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'est de la vraie "daube".

### "vu" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Qui étaient sur le DVD que nous avons vu .

### "vue" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il était l'amour à première vue .

### "véritable" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - C'était une véritable "douche industrielle".

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

### "«" + "Aimez" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Aimez-vous les gâteaux ?» «Oui, j'aime ça.»

### "«" + "Bien" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Puis-je venir avec toi ?» «Bien sûr !»

### "«" + "Bonjour" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Dis juste «Bonjour !» et ils vont penser que tu es français !

### "«" + "Ce" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Et qu'est-ce que c'est que ça ?» «Ce sont tes photos.»

### "«" + "Cela" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Cela me va, cela me va !» se dit Passepartout.

### "«" + "Ces" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Ces cordes donnent la quinte et l'octave», dit Mr. Fogg.

### "«" + "Combien" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Combien de temps va durer la réunion ?» «Deux heures.»

### "«" + "Crois" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Crois-tu vraiment cela ?» «Je le sais.»

### "«" + "Deux" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Combien de temps va durer la réunion ?» «Deux heures.»

### "«" + "Décidément" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Décidément, il faut avouer que je coûte cher à mon maître !»

### "«" + "Encore" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Encore cinq minutes», dit Andrew Stuart.

### "«" + "Envoie" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Je peux te poser une question ?» «Envoie.»

### "«" + "Es" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Es-tu sûr de vouloir que je te le dise ?» «Je suis tout ouïe !»

### "«" + "Feu" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Feu !» dit Mr. Fogg.

### "«" + "Fix" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Fix, détective.»

### "«" + "Great Indian peninsular railway" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Voici, en somme, le tracé à grands points du «Great Indian peninsular railway».

### "«" + "Il" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Il va pleuvoir ?» «J'espère que non.»

### "«" + "Jamais" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Jamais trop tard», dit-il.

### "«" + "L'" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «L'examen se tiendra aujourd'hui en huit», dit le professeur.

### "«" + "La" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «La première cause», dit-il.

### "«" + "Les" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Les voyageurs descendent ici.»

### "«" + "Mais" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Mais, ajouta le pilote, il y aurait peut-être moyen de s'arranger autrement.»

### "«" + "Me" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Me voici, messieurs», disait-il.

### "«" + "Mes" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Mes amis, il y a mille livres pour vous si nous sauvons les prisonniers !»

### "«" + "Moi" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «J'aime voyager.» «Moi aussi.»

### "«" + "Naturellement" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Est-ce que ça va si je me joins à vous ?» «Naturellement !»

### "«" + "Non" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Est-ce qu'il vient ?» «Non, je ne pense pas.».

### "«" + "Nous" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Nous sommes arrivés.»

### "«" + "Oui" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Aimez-vous les gâteaux ?» «Oui, j'aime ça.»

### "«" + "Positivement" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Positivement il sera enragé !»

### "«" + "Poussez" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Poussez les feux et faites route jusqu'à complet épuisement du combustible.»

### "«" + "Puis" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Puis-je venir avec toi ?» «Bien sûr !»

### "«" + "Rien" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Rien à faire !» se dit le détective en dissimulant son désappointement.

### "«" + "Si" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Si rien ne casse, dit Mudge, nous arriverons !»

### "«" + "Sur" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Sur le pont», qu’est-ce que vous allez foutre ?

### "«" + "Tenez" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Tenez, ma brave femme, dit-il, je suis content de vous avoir rencontrée !»

### "«" + "Tiens" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Tiens, Kiouni, tiens, tiens !»

### "«" + "Tout" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Que voulez-vous sur votre pizza ?» «Tout sauf des anchois.»

### "«" + "Traité" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - J'ai lu un «Traité des serrures de combinaison» qu'on lui attribue.

### "«" + "Très" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Tu me trouves comment ?» «Très jolie.»

### "«" + "Un" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Un sutty !» dit-il.

### "«" + "Urban Legends" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Notre prochaine fonctionnalité est «Urban Legends».

### "«" + "Venez" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Venez !» dit le guide à voix basse.

### "«" + "Vous" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Vous, pilote, vous êtes un brave homme !»

### "«" + "Voyez" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Voyez donc ce que c'est.»

### "«" + "ce" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mais «ce beau temps» était passé.

### "«" + "désencombré" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - En quarante minutes, le salon fut «désencombré» selon l'expression d'Arsène.

### "«" + "gant" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Mon mot préféré en allemand est le mot pour «gant».

### "«" + "ha" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et je sais que la posture «ha».

### "«" + "hang" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - L'ivresse du «hang» leur avait été probablement épargnée.

### "«" + "mahout" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le Parsi connaissait parfaitement le métier de «mahout» ou cornac.

### "«" + "merci" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Je voudrais dire «merci» à Tom.

### "«" + "non" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Peu importe ce que tu dis, la réponse est «non ».

### "«" + "notre" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Et dès l'abord on traita «notre sauveur» comme un ami d'ancienne date.

### "«" + "pouvoir" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Le temps n’est pas un «pouvoir».

### "«" + "très" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Cela parut «très américain» à Passepartout.

### "«" + "À" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «À coup sûr, pensait-il, c'est un coquin fort poli, mais c'est un coquin !»

### "«" + "Ça" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - «Ça te dérange pas si je fume ?» «Pas le moins du monde.»

### "«" + "ça" (1 occurrences)
- Predicted: NarrowNbsp
- Actual: None
- Examples:
  - Passepartout, tout honteux, répétait que «ça n'en valait pas la peine» !

### "À" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - À "trois", ils avancent.

### "Çorum" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Nous écrivons Çorum "Ĉorum" en espéranto.

### "Écoutons" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Écoutons "Fuir la nuit".

### "État" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ce que nous ne pouvons pas admettre, c'est qu'un représentant de l'État ..

### "Être" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Être "Fermeur" est une tradition familiale depuis des générations.

### "à" + ""Youpi l'École" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je lâche le mot, et ta Betty Boop cachetonne à "Youpi l'École".

### "à" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Il ya encore une chose que je voulais dire à .

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

### "écoutez" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous écoutez "le Défilé des chansons" de Séoul FM.

### "écrira" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - On écrira "Potts" sur la prochaine tour.

### "écrire" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Mais avec le vieux Tony, rien à lire, rien à écrire - tout dans la tête.

### "église" + "-" (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Clochers d'église - des entonnoirs renversés pour conduire les prières au ciel.

### "éjacule" + ".." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Un poussin qui éjacule ..

### "épaisse" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Je pense gonzesses comme la chose la plus épaisse .

### "épeler" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Personne ne peut épeler "spaghetti".

### "épreuve" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Mais ce soir, il va être mis à l'épreuve "et sa vraie nature sera révélée.

### "équipe" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - "Yuriko, Rei et l'équipe "m'avaient organisé une belle fête.

### "équipées" + "." (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Ces hommes sont bien équipées .

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

### "éviter" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Non, on va éviter "sa mort".

### "événement" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Préparation de l'événement "Expérience de pêcheur".

### "êtes" + """ (1 occurrences)
- Predicted: None
- Actual: Space
- Examples:
  - Vous êtes "photomane" ?

### "île" + "'" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est 'île' en espagnol mais à l'envers.

### "—" + "Phileas Fogg" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tous—Phileas Fogg excepté sans doute—sentaient leur cœur battre d'impatience.

### "—" + "en" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'était un homme qui avait dû voyager partout,—en esprit, tout au moins.

### "—" + "fille" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - C'est pas facile, les rapports père—fille.

### "—" + "mais" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il mangea cependant,—sur le pouce, il est vrai,—mais enfin il mangea.

### "—" + "sentaient" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Tous—Phileas Fogg excepté sans doute—sentaient leur cœur battre d'impatience.

### "—" + "sur" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - Il mangea cependant,—sur le pouce, il est vrai,—mais enfin il mangea.

### "”" + "pour" (1 occurrences)
- Predicted: Space
- Actual: None
- Examples:
  - "Peut-être qu'un jour tu finiras par venir à Londres ”pour une de tes missions.

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

