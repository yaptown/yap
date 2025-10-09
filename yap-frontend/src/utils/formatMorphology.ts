import type { Morphology } from '../../../yap-frontend-rs/pkg'

export function formatMorphology(morphology: Morphology): string {
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
