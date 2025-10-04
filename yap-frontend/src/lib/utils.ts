import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { get_audio, invalidate_audio_cache, type AudioRequest } from '../../../yap-frontend-rs/pkg'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

const SINGLE_QUOTE_VARIANTS = /[‘’‚‛′‵❛❜＇ʻʼʽʹ`´]/g
const DOUBLE_QUOTE_VARIANTS = /[“”„‟″‶❝❞＂]/g
const HYPHEN_VARIANTS = /[‐‑‒–—―−﹘﹣－]/g

export function normalizeSpecialCharacters(text: string): string {
  return text
    .normalize('NFKC')
    .replace(SINGLE_QUOTE_VARIANTS, "'")
    .replace(DOUBLE_QUOTE_VARIANTS, '"')
    .replace(HYPHEN_VARIANTS, '-')
}

export const profilerOnRender = (id: string, phase: string, actualDuration: number, baseDuration: number, startTime: number, commitTime: number) => {
  void id
  void phase
  void actualDuration
  void baseDuration
  void startTime
  void commitTime
  // console.log(`id:`, id, `, phase:`, phase, `, actualDuration:`, actualDuration, `, baseDuration:`, baseDuration, `, startTime:`, startTime, `, commitTime:`, commitTime);
}

let isPlayingAudio = false;

export async function playAudio(audioRequest: AudioRequest, accessToken: string | undefined, needsAuth: () => void): Promise<void> {
  if (isPlayingAudio) {
    console.log('Audio already playing, skipping...');
    return;
  }

  isPlayingAudio = true;
  try {
    const audioData = await get_audio(audioRequest, accessToken);
    
    const audioBlob = new Blob([audioData], { type: 'audio/mpeg' });
    const audioUrl = URL.createObjectURL(audioBlob);
    
    const audio = new Audio(audioUrl);
    
    return new Promise((resolve, reject) => {
      const invalidateCache = () => {
        void (async () => {
          try {
            await invalidate_audio_cache(audioRequest);
          } catch (invalidateError) {
            console.error('Failed to invalidate audio cache:', invalidateError);
          }
        })();
      };

      const handlePlaybackFailure = (error: unknown) => {
        URL.revokeObjectURL(audioUrl);
        invalidateCache();
        if (error instanceof Error) {
          reject(error);
        } else {
          reject(new Error(String(error)));
        }
      };

      audio.onended = () => {
        URL.revokeObjectURL(audioUrl);
        resolve();
      };

      audio.onerror = () => {
        handlePlaybackFailure(new Error('Audio playback failed'));
      };

      audio
        .play()
        .catch((error) => {
          handlePlaybackFailure(error);
        });
    });
  } catch (error) {
    if (typeof error === 'string' && error.includes('400')) {
      needsAuth();
    }
    console.error('Failed to play audio:', error);
    throw error;
  } finally {
    isPlayingAudio = false;
  }
}
