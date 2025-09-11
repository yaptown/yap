import { Button } from "@/components/ui/button"
import { type Language } from '../../../yap-frontend-rs/pkg/yap_frontend_rs'
import { match } from 'ts-pattern'

interface AccentedCharacterKeyboardProps {
  onCharacterInsert: (char: string) => void
  language: Language
  className?: string
}

export function AccentedCharacterKeyboard({ 
  onCharacterInsert, 
  language,
  className = ""
}: AccentedCharacterKeyboardProps) {
  const characters = match(language)
    .with('French', () => ['à', 'â', 'é', 'è', 'ê', 'ë', 'î', 'ï', 'ô', 'ù', 'û', 'ü', 'ÿ', 'ç', 'œ', 'æ'])
    .with('Spanish', () => ['á', 'é', 'í', 'ó', 'ú', 'ü', 'ñ', '¿', '¡'])
    .with('Korean', () => [])
    .with('English', () => [])
    .exhaustive()

  // Split characters into rows of at most 8 characters each
  const rows: string[][] = []
  for (let i = 0; i < characters.length; i += 8) {
    rows.push(characters.slice(i, i + 8))
  }
  if (characters.length === 0) {
    return null
  }

  return (
    <div className={`accent-keyboard flex flex-col items-center ${className}`}>
      {rows.map((row, rowIndex) => (
        <div key={rowIndex} className="flex justify-center">
          {row.map((char, index) => (
            <Button
              key={char}
              variant="outline"
              size="sm"
              className={`h-8 w-10 text-base font-medium rounded-none border-r-0 last:border-r ${
                index === 0 ? 'rounded-l-md' : ''
              } ${
                index === row.length - 1 ? 'rounded-r-md' : ''
              }`}
              onClick={() => onCharacterInsert(char)}
              onMouseDown={(e) => e.preventDefault()}
              type="button"
            >
              {char}
            </Button>
          ))}
        </div>
      ))}
    </div>
  )
}
