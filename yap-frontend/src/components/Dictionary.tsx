import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { type Deck, type Morphology, type Weapon, type Language } from '../../../yap-frontend-rs/pkg'
import { ArrowLeft, CirclePlus, CircleCheckBig } from 'lucide-react'
import { toast } from 'sonner'

function formatMorphology(morphology: Morphology): string {
  const parts: string[] = []

  if (morphology.person) {
    const personMap = {
      Zeroth: '0th-person',
      First: '1st-person',
      Second: '2nd-person',
      Third: '3rd-person',
      Fourth: '4th-person'
    } as const
    parts.push(personMap[morphology.person])
  }

  if (morphology.gender) {
    const genderMap = {
      Masculine: 'masculine',
      Feminine: 'feminine',
      Neuter: 'neuter',
      Common: 'common'
    } as const
    parts.push(genderMap[morphology.gender])
  }

  if (morphology.tense) {
    const tenseMap = {
      Past: 'past tense',
      Present: 'present tense',
      Future: 'future tense',
      Imperfect: 'imperfect tense',
      Pluperfect: 'pluperfect tense'
    } as const
    parts.push(tenseMap[morphology.tense])
  }

  if (morphology.politeness) {
    const politeMap = {
      Informal: 'informal',
      Formal: 'formal',
      Elev: 'elevated',
      Humb: 'humble'
    } as const
    parts.push(politeMap[morphology.politeness])
  }

  return parts.length > 0 ? parts.join(', ') : ''
}

export function Dictionary({ deck, weapon, targetLanguage }: { deck: Deck, weapon: Weapon, targetLanguage: Language }) {
  const navigate = useNavigate()
  const [searchQuery, setSearchQuery] = useState('')
  const [addedCards, setAddedCards] = useState<Set<string>>(new Set())

  const entries = deck.get_dictionary_entries()

  // Check which cards are already added
  useEffect(() => {
    const allCards = deck.get_all_cards_summary()
    const added = new Set<string>()
    allCards.forEach(card => {
      if (card.card_indicator.TargetLanguage) {
        const lexeme = card.card_indicator.TargetLanguage.lexeme
        if (lexeme.Heteronym) {
          const key = `${lexeme.Heteronym.word}-${lexeme.Heteronym.lemma}-${lexeme.Heteronym.pos}`
          added.add(key)
        }
      }
    })
    setAddedCards(added)
  }, [deck])

  const handleAddCard = (word: string, heteronym: any) => {
    // Create the card indicator
    const cardIndicator = {
      TargetLanguage: {
        lexeme: {
          Heteronym: heteronym
        }
      }
    }

    // Create the AddCards event
    const event = {
      Language: {
        target_language: targetLanguage,
        content: {
          AddCards: {
            cards: [cardIndicator]
          }
        }
      }
    }

    // Add the event to the weapon
    weapon.add_deck_event(event)

    // Update the added cards set
    const key = `${heteronym.word}-${heteronym.lemma}-${heteronym.pos}`
    setAddedCards(prev => new Set(prev).add(key))
    toast.success(`Added "${word}" to your deck`)
  }

  const isCardAdded = (heteronym: any) => {
    const key = `${heteronym.word}-${heteronym.lemma}-${heteronym.pos}`
    return addedCards.has(key)
  }

  const filteredEntries = (() => {
    if (!searchQuery.trim()) return entries.slice(0, 100) // Only show top 100 when not searching

    const query = searchQuery.toLowerCase()
    return entries.filter(entry => {
      // Search in target language word
      if (entry.word.toLowerCase().includes(query)) return true

      // Search in native language translations
      return entry.entry.definitions.some(def =>
        def.native.toLowerCase().includes(query)
      )
    }).slice(0, 100) // Limit search results to 100 as well
  })()

  return (
    <div className="flex-1 overflow-hidden flex flex-col">
      <div className="p-4 border-b">
        <div className="flex items-center gap-3 mb-4">
          <button
            onClick={() => navigate('/')}
            className="p-2 hover:bg-muted rounded-lg transition-colors"
            aria-label="Back to home"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <h1 className="text-2xl font-bold">Dictionary</h1>
        </div>
        <input
          type="text"
          placeholder="Search in French or English..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-4 py-2 border rounded-lg bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
        />
        <p className="text-sm text-muted-foreground mt-2">
          Showing {filteredEntries.length} of {entries.length} {entries.length === 1 ? 'entry' : 'entries'}
          {searchQuery && ` matching "${searchQuery}"`}
        </p>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-4">
          {filteredEntries.map((entry, index) => (
            <div key={`${entry.word}-${index}`} className="border rounded-lg p-4 bg-card relative">
              <div className="flex items-baseline justify-between gap-4 mb-2">
                <h2 className="text-xl font-semibold">{entry.word}</h2>
                {(() => {
                  const morphologyText = formatMorphology(entry.entry.morphology)
                  return morphologyText && (
                    <span className="text-sm text-muted-foreground italic">
                      {morphologyText}
                    </span>
                  )
                })()}
              </div>

              <div className="space-y-3">
                {entry.entry.definitions.map((def, defIndex) => (
                  <div key={defIndex} className="pl-4 border-l-2 border-muted">
                    <div className="font-medium text-primary">{def.native}</div>
                    {def.note && (
                      <div className="text-sm text-muted-foreground italic mt-1">
                        {def.note}
                      </div>
                    )}
                    <div className="mt-2 text-sm space-y-1">
                      <div className="text-foreground">
                        <span className="text-muted-foreground">FR:</span> {def.example_sentence_target_language}
                      </div>
                      <div className="text-muted-foreground">
                        <span>EN:</span> {def.example_sentence_native_language}
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              <div className="absolute bottom-3 right-3">
                {isCardAdded(entry.heteronym) ? (
                  <button
                    disabled
                    className="flex items-center gap-2 px-3 py-1.5 text-sm text-muted-foreground cursor-default"
                  >
                    <CircleCheckBig className="w-4 h-4" />
                    <span>Added</span>
                  </button>
                ) : (
                  <button
                    onClick={() => handleAddCard(entry.word, entry.heteronym)}
                    className="flex items-center gap-2 px-3 py-1.5 text-sm text-foreground hover:bg-muted rounded-md transition-colors"
                  >
                    <CirclePlus className="w-4 h-4" />
                    <span>Add to deck</span>
                  </button>
                )}
              </div>
            </div>
          ))}

          {filteredEntries.length === 0 && (
            <div className="text-center py-12 text-muted-foreground">
              {searchQuery ? 'No entries found matching your search.' : 'No dictionary entries available.'}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
