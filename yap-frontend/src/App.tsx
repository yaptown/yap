import { useState, useEffect, Profiler, useSyncExternalStore, useMemo, useCallback } from 'react'
import { BrowserRouter, Routes, Route, Outlet, useNavigate, useOutletContext } from 'react-router-dom'
import { CardSummary, Deck, type AddCardOptions, type CardType, type Challenge, type ChallengeRequirements, type Course, type Language, type Lexeme, type /* comes from TranscriptionChallenge */ PartGraded, type Rating } from '../../yap-frontend-rs/pkg'
import { Button } from "@/components/ui/button.tsx"
import { Progress } from "@/components/ui/progress.tsx"
import { Skeleton } from "@/components/ui/skeleton"
import { Card } from "@/components/ui/card"
import { ThemeProvider } from "@/components/theme-provider"
import { supabase } from '@/lib/supabase'
import type { Session as SupabaseSession } from '@supabase/supabase-js'
import { useInterval, useNetworkState } from 'react-use';
import { Flashcard } from '@/components/Flashcard'
import { TranslationChallenge } from '@/components/challenges/TranslationChallenge'
import { profilerOnRender } from './lib/utils'
import { ResetPassword } from '@/pages/reset-password'
import { ConfirmEmail } from '@/pages/confirm-email'
import { AcceptInvite } from '@/pages/accept-invite'
import { ForgotPassword } from '@/pages/forgot-password'
import { UserProfilePage } from '@/pages/user-profile'
import { playSoundEffect } from '@/lib/sound-effects'
import { registerSW } from 'virtual:pwa-register'
import { NoCardsReady } from '@/components/no-cards-ready'
import { SetDisplayName } from '@/components/SetDisplayName'

import type { Dispatch, SetStateAction } from 'react'
import type { RegisterSWOptions } from 'vite-plugin-pwa/types'
declare module 'virtual:pwa-register/react' {
  export function useRegisterSW(options?: RegisterSWOptions): {
    needRefresh: [boolean, Dispatch<SetStateAction<boolean>>]
    offlineReady: [boolean, Dispatch<SetStateAction<boolean>>]
    updateServiceWorker: (reloadPage?: boolean) => Promise<void>
  }
}
import { useRegisterSW } from 'virtual:pwa-register/react'
import { TranscriptionChallenge } from './components/challenges/TranscriptionChallenge'
import { LanguageSelector } from './components/LanguageSelector'
import { WeaponProvider, useAsyncMemo, useWeapon, useWeaponState, useWeaponSupport, type WeaponToken } from './weapon'
import { Toaster } from 'sonner'
import { BrowserNotSupported } from '@/components/browser-not-supported'
import { Stats } from '@/components/stats'
import { About } from '@/components/about'
import { Dictionary } from '@/components/Dictionary'
import { Leeches } from '@/components/Leeches'
import { TopPageLayout } from '@/components/TopPageLayout'
import { match, P } from 'ts-pattern';
import { ErrorMessage } from '@/components/ui/error-message'
import { BackgroundShader } from '@/components/BackgroundShader'

// Essential user info to persist for offline functionality
export interface UserInfo {
  id: string
  email: string
  displayName: string | null | undefined
}

export type AppContextType = {
  userInfo: UserInfo | undefined
  accessToken: string | undefined
}

function AppMain() {
  // register service worker
  const updateIntervalMS = 60 * 5 * 1000; // every 5 minutes
  useEffect(() => {
    registerSW({ immediate: true })
  }, [])

  useRegisterSW({
    onRegistered(r) {
      if (r) {
        setInterval(() => {
          r.update()
        }, updateIntervalMS)
      }
    }
  });

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <BackgroundShader>
        <AppCheckBrowserSupport />
        <Toaster />
      </BackgroundShader>
    </ThemeProvider>
  )
}

function AppCheckBrowserSupport() {
  const token = useWeaponSupport()
  const supported = token.browserSupported
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    if (supported !== null) return

    const start = Date.now()
    const timer = setInterval(() => {
      const diff = Date.now() - start
      setProgress(Math.max(1, Math.min(diff / 30, 100)))
    }, 480)

    return () => clearInterval(timer)
  }, [supported])

  if (supported === null) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center space-y-4">
        <p className="text-muted-foreground animate-fade-in-delay-2">Checking device compatibility...</p>
        <Progress value={progress} className="w-64 animate-fade-in-delay-2" />
      </div>
    )
  }
  else if (supported === false) {
    return <BrowserNotSupported />
  }
  else {
    return <AppCheckLoggedIn weaponToken={{ browserSupported: supported }} />
  }
}

function AppCheckLoggedIn({ weaponToken }: { weaponToken: WeaponToken }) {
  void weaponToken
  const [session, setSession] = useState<SupabaseSession | null>(null)
  const [signedOut, setSignedOut] = useState(false)
  const [displayName, setDisplayName] = useState<string | null | undefined>(undefined)

  useEffect(() => {
    supabase.auth.getSession().then(({ data: { session } }) => {
      setSession(session)
    })

    const { data: authListener } = supabase.auth.onAuthStateChange((event, session) => {
      setSession(session)
      if (event === 'SIGNED_IN') {
        localStorage.setItem('yap-user-info', JSON.stringify({
          id: session?.user.id,
          email: session?.user.email,
          displayName: undefined // Will be fetched from profiles table
        }))
        setSignedOut(false)
      } else if (event === 'SIGNED_OUT') {
        localStorage.removeItem('yap-user-info')

        if (window.OneSignal) {
          window.OneSignal.logout()
        }

        setSession(null)
        setDisplayName(undefined)
        setSignedOut(true)
      }
    })

    return () => {
      authListener.subscription.unsubscribe()
    }
  }, [])

  // Fetch display name from Supabase when logged in
  useEffect(() => {
    if (!session?.user.id) {
      setDisplayName(undefined)
      return
    }

    // Fetch initial display name
    const fetchDisplayName = async () => {
      const { data, error } = await supabase
        .from('profiles')
        .select('display_name')
        .eq('id', session.user.id)
        .single()

      if (!error && data) {
        setDisplayName(data.display_name)
      }
    }

    fetchDisplayName()

    // Set up realtime subscription for display_name changes
    const channel = supabase
      .channel(`profile_${session.user.id}`)
      .on(
        'postgres_changes',
        {
          event: 'UPDATE',
          schema: 'public',
          table: 'profiles',
          filter: `id=eq.${session.user.id}`
        },
        (payload) => {
          if (payload.new && 'display_name' in payload.new) {
            setDisplayName(payload.new.display_name as string | null)
          }
        }
      )
      .subscribe()

    return () => {
      supabase.removeChannel(channel)
    }
  }, [session?.user.id])

  // Update localStorage when displayName changes (only when it's been fetched)
  useEffect(() => {
    if (session?.user.id && session?.user.email && displayName !== undefined) {
      localStorage.setItem('yap-user-info', JSON.stringify({
        id: session.user.id,
        email: session.user.email,
        displayName: displayName
      }))
    }
  }, [session?.user.id, session?.user.email, displayName])

  let userInfo: UserInfo | undefined;

  if (session) {
    userInfo = {
      id: session.user.id,
      email: session.user.email!,
      displayName: displayName
    }
  } else if (!signedOut) {
    const cachedUserInfo = localStorage.getItem('yap-user-info')
    if (cachedUserInfo) {
      try {
        userInfo = JSON.parse(cachedUserInfo)
      } catch {
        localStorage.removeItem('yap-user-info')
      }
    }
  }

  const accessToken = session?.access_token

  return (
    <WeaponProvider userId={userInfo?.id} accessToken={accessToken}>
      <AppTestWeapon userInfo={userInfo} accessToken={accessToken} />
    </WeaponProvider>
  )
}

function AppTestWeapon({ userInfo, accessToken }: AppContextType) {
  const weaponState = useWeaponState()

  if (weaponState.type === 'loading') {
    return (
      <div>
        <div className="min-h-screen flex items-center justify-center">
          <p className="text-muted-foreground animate-fade-in-delayed">Loading...</p>
        </div>
      </div>
    )
  }
  else if (weaponState.type === 'error') {
    return (
      <div>
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="max-w-md w-full p-6 text-center gap-0">
            <div className="w-12 h-12 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mx-auto mb-4">
              <span className="text-red-600 dark:text-red-400 text-xl">⚠</span>
            </div>
            <h2 className="text-lg font-semibold mb-2">Failed to Initialize Deck</h2>
            <p className="text-muted-foreground mb-4">{weaponState.message}</p>
            <Button
              onClick={() => window.location.reload()}
              variant="outline"
            >
              Try Again
            </Button>
          </Card>
        </div>
      </div>
    )
  }
  else if (weaponState.type === 'ready') {
    return <AppContent userInfo={userInfo} accessToken={accessToken} />
  }
}

function AppContent({ userInfo, accessToken }: AppContextType) {
  return (
    <Profiler id="App" onRender={profilerOnRender}>
      <div className="px-2">
        <div className="min-h-screen text-foreground">
          <div className="max-w-2xl mx-auto">
            <Profiler id="Content" onRender={profilerOnRender}>
              <Outlet context={{ userInfo, accessToken }} />
              <About />
            </Profiler>
            <div className="p-2"></div>
          </div>
        </div>
      </div>
    </Profiler>
  )
}

function ReviewPage() {
  const { userInfo, accessToken } = useOutletContext<AppContextType>()
  const deck = useDeck()
  const navigate = useNavigate()

  useEffect(() => {
    if (deck?.type === 'noLanguageSelected') {
      navigate('/select-language')
    }
  }, [deck, navigate])

  return (
    <div className="flex flex-col gap-6">
      {
        match(deck)
          .with({ type: "deck", deck: null }, () => (
            <TopPageLayout
              userInfo={userInfo}
              headerProps={{
                onChangeLanguage: () => navigate('/select-language'),
                showSignupNag: false
              }}
            >
              <div className="flex-1 flex items-center justify-center">
                <p className="text-muted-foreground animate-fade-in-delayed">Loading...</p>
              </div>
            </TopPageLayout>
          ))
          .with({ type: "deck", deck: P.not(P.nullish) }, ({ deck, targetLanguage, nativeLanguage }) => {
            const reviewInfo = deck.get_review_info([], Date.now());
            return (
            <>
              <TopPageLayout
                userInfo={userInfo}
                headerProps={{
                  onChangeLanguage: () => navigate('/select-language'),
                  showSignupNag: deck !== null,
                  language: targetLanguage,
                  dueCount: reviewInfo.due_count || 0
                }}
              >
                <Review
                  userInfo={userInfo}
                  accessToken={accessToken}
                  deck={deck}
                  targetLanguage={targetLanguage}
                  nativeLanguage={nativeLanguage}
                />
              </TopPageLayout>
              <Tools deck={deck} />
              <Stats deck={deck} />
            </>
            );
          })
          .with({ type: "noLanguageSelected" }, () => (
            <TopPageLayout
              userInfo={userInfo}
              headerProps={{ showSignupNag: false }}
            >
              <div className="flex-1 flex items-center justify-center">
                <p className="text-muted-foreground animate-fade-in-delayed">Loading...</p>
              </div>
            </TopPageLayout>
          ))
          .with({ type: "error" }, ({ message, retry }) => (
            <TopPageLayout
              userInfo={userInfo}
              headerProps={{
                onChangeLanguage: () => navigate('/select-language'),
                showSignupNag: false
              }}
            >
              <div className="flex-1 flex items-center justify-center p-4">
                <Card className="max-w-md w-full p-6 gap-0">
                  <div className="w-12 h-12 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mx-auto mb-4">
                    <span className="text-red-600 dark:text-red-400 text-xl">⚠</span>
                  </div>
                  <h2 className="text-lg font-semibold mb-2 text-center">Failed to Load Language Data</h2>
                  <p className="text-muted-foreground mb-4 text-center">
                    Unable to download the language pack. Please check your internet connection.
                  </p>
                  <ErrorMessage message={message} title="Failed to load language data" className="mb-4" />
                  <Button onClick={retry} variant="outline" className="w-full">
                    Try Again
                  </Button>
                </Card>
              </div>
            </TopPageLayout>
          ))
          .with(null, () => (
            <TopPageLayout
              userInfo={userInfo}
              headerProps={{ showSignupNag: false }}
            >
            <div className="flex items-center justify-center p-4 animate-fade-in-delayed">
              <Skeleton className="h-48 w-full max-w-2xl" />
             </div>
            </TopPageLayout>
          ))
          .exhaustive()
      }
    </div>
  )
}

function Tools({ deck }: { deck: Deck }) {
  const navigate = useNavigate()
  const movieStats = useMemo(() => deck.get_movie_stats(), [deck])
  const [showAllMovies, setShowAllMovies] = useState(false)

  // Find movie closest to next milestone
  const closestToMilestone = useMemo(() => {
    return movieStats
      .filter(m => m.cards_to_next_milestone !== null && m.cards_to_next_milestone !== undefined)
      .sort((a, b) => (a.cards_to_next_milestone || 0) - (b.cards_to_next_milestone || 0))[0]
  }, [movieStats])

  const visibleMovies = showAllMovies ? movieStats : movieStats.slice(0, 8)

  // Helper function to convert poster bytes to data URL
  const getPosterDataUrl = (posterBytes: number[] | undefined) => {
    if (!posterBytes) return null
    const uint8Array = new Uint8Array(posterBytes)
    let binaryString = ''
    const chunkSize = 8192
    for (let i = 0; i < uint8Array.length; i += chunkSize) {
      const chunk = uint8Array.subarray(i, i + chunkSize)
      binaryString += String.fromCharCode(...chunk)
    }
    return `data:image/jpeg;base64,${btoa(binaryString)}`
  }

  return (
    <div className="">
      <h2 className="text-2xl font-semibold animate-fade-in-delay-2">Tools</h2>
      <Card className="p-4 mt-3 space-y-2 gap-0" animate>
        <button
          onClick={() => {
            navigate('/dictionary');
            window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
          }}
          className="w-full flex items-center justify-between px-3 py-2 rounded-md hover:bg-muted transition-colors mb-0"
        >
          <span>📖 Dictionary</span>
          <span className="text-muted-foreground">→</span>
        </button>
        <button
          onClick={() => {
            navigate('/leeches');
            window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
          }}
          className="w-full flex items-center justify-between px-3 py-2 rounded-md hover:bg-muted transition-colors"
        >
          <span>🩹 Leeches</span>
          <span className="text-muted-foreground">→</span>
        </button>
      </Card>

      {/* Movie Comprehensibility Section */}
      {movieStats.length > 0 && (
        <div className="mt-6">
          <h2 className="text-2xl font-semibold mb-3">Movies</h2>
          <p className="text-sm text-muted-foreground mb-4">
            These movies are sorted by how much of the dialogue you already know. You can usually watch a movie comfortably once you know 95% of the words.
          </p>

          {/* Featured movie closest to milestone */}
          {closestToMilestone && (
            <Card className="mb-6 overflow-hidden p-0 border-primary/50" animate>
              <div className="flex flex-col sm:flex-row gap-0">
                <div className="sm:w-32 w-full aspect-[2/3] sm:aspect-[2/3] bg-muted relative">
                  {getPosterDataUrl(closestToMilestone.poster_bytes) ? (
                    <img
                      src={getPosterDataUrl(closestToMilestone.poster_bytes)!}
                      alt={closestToMilestone.title}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-4xl">
                      🎬
                    </div>
                  )}
                </div>
                <div className="flex-1 p-4 flex flex-col justify-center">
                  <div className="text-xs font-medium text-primary mb-1">ALMOST THERE</div>
                  <h3 className="text-lg font-semibold mb-1">{closestToMilestone.title}</h3>
                  {closestToMilestone.year && (
                    <div className="text-sm text-muted-foreground mb-2">{closestToMilestone.year}</div>
                  )}
                  <p className="text-sm mb-3">
                    You're just <span className="font-semibold text-foreground">{closestToMilestone.cards_to_next_milestone} {closestToMilestone.cards_to_next_milestone === 1 ? 'card' : 'cards'}</span> away from reaching <span className="font-semibold text-foreground">{Math.ceil(closestToMilestone.percent_known / 5) * 5}%</span> comprehension!
                  </p>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 h-2 bg-muted rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary transition-all duration-300"
                        style={{ width: `${closestToMilestone.percent_known}%` }}
                      />
                    </div>
                    <span className="text-xs font-mono font-semibold">
                      {closestToMilestone.percent_known.toFixed(0)}%
                    </span>
                  </div>
                </div>
              </div>
            </Card>
          )}

          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
            {visibleMovies.map((movie) => {
              const posterDataUrl = getPosterDataUrl(movie.poster_bytes)

              return (
                <Card
                  key={movie.id}
                  className="overflow-hidden p-0 hover:ring-2 hover:ring-primary transition-all cursor-pointer group gap-0"
                  animate
                >
                  <div className="relative aspect-[2/3] bg-muted">
                    {posterDataUrl ? (
                      <img
                        src={posterDataUrl}
                        alt={movie.title}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center text-4xl">
                        🎬
                      </div>
                    )}
                    <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
                      <div className="absolute bottom-0 left-0 right-0 p-3">
                        <div className="text-white text-sm font-semibold line-clamp-2">
                          {movie.title}
                        </div>
                        {movie.year && (
                          <div className="text-white/70 text-xs mt-1">
                            {movie.year}
                          </div>
                        )}
                        {movie.cards_to_next_milestone !== null && movie.cards_to_next_milestone !== undefined && (
                          <div className="text-white/90 text-xs mt-2 font-medium">
                            {movie.cards_to_next_milestone} {movie.cards_to_next_milestone === 1 ? 'card' : 'cards'} to {Math.ceil(movie.percent_known / 5) * 5}%
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="p-2 text-center relative overflow-hidden">
                    <div
                      className="absolute inset-0 bg-foreground/10"
                      style={{
                        clipPath: `inset(0 ${100 - movie.percent_known}% 0 0)`
                      }}
                    />
                    <span className="relative text-sm font-mono font-semibold text-foreground">
                      {movie.percent_known.toFixed(0)}% known
                    </span>
                  </div>
                </Card>
              );
            })}
          </div>
          {!showAllMovies && movieStats.length > 10 && (
            <div className="mt-4">
              <button
                onClick={() => setShowAllMovies(true)}
                className="w-full py-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors duration-200 font-medium rounded-md border border-border"
              >
                Show all {movieStats.length} movies
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function DictionaryPage() {
  const { userInfo } = useOutletContext<AppContextType>()
  const deck = useDeck()
  const weapon = useWeapon()
  const navigate = useNavigate()

  useEffect(() => {
    if (deck?.type === 'noLanguageSelected') {
      navigate('/', { replace: true })
    }
  }, [deck, navigate])

  if (deck?.type === 'noLanguageSelected') {
    return null
  }

  if (deck?.type !== 'deck') {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Dictionary', onBack: () => navigate('/') }
        }}
      >
        <div className="flex-1 flex items-center justify-center">
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </TopPageLayout>
    )
  }

  if (!deck.deck) {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Dictionary', onBack: () => navigate('/') }
        }}
      >
        <div className="flex-1 bg-background flex items-center justify-center">
          <p className="text-muted-foreground">Loading dictionary...</p>
        </div>
      </TopPageLayout>
    )
  }

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: 'Dictionary', onBack: () => navigate('/') }
      }}
    >
      <Dictionary deck={deck.deck} weapon={weapon} targetLanguage={deck.targetLanguage} nativeLanguage={deck.nativeLanguage} />
    </TopPageLayout>
  )
}

function LeechesPage() {
  const { userInfo } = useOutletContext<AppContextType>()
  const deck = useDeck()
  const navigate = useNavigate()

  useEffect(() => {
    if (deck?.type === 'noLanguageSelected') {
      navigate('/', { replace: true })
    }
  }, [deck, navigate])

  if (deck?.type === 'noLanguageSelected') {
    return null
  }

  if (deck?.type !== 'deck') {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Leeches', onBack: () => navigate('/') }
        }}
      >
        <div className="flex-1 flex items-center justify-center">
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </TopPageLayout>
    )
  }

  if (!deck.deck) {
    return (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Leeches', onBack: () => navigate('/') }
        }}
      >
        <div className="flex-1 bg-background flex items-center justify-center">
          <p className="text-muted-foreground">Loading leeches...</p>
        </div>
      </TopPageLayout>
    )
  }

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: 'Leeches', onBack: () => navigate('/') }
      }}
    >
      <Leeches deck={deck.deck} />
    </TopPageLayout>
  )
}

function findNextDueCard(deck: Deck): CardSummary | null {
  const allCards = deck.get_all_cards_summary()
  const now = Date.now()
  const futureCards = allCards.filter(card => card.due_timestamp_ms > now)
  return futureCards.length > 0 ? futureCards[0] : null
}

interface ReviewProps {
  userInfo: UserInfo | undefined
  accessToken: string | undefined
  deck: Deck
  targetLanguage: Language
  nativeLanguage: Language
}

function Review({ userInfo, accessToken, deck, targetLanguage, nativeLanguage }: ReviewProps) {
  const weapon = useWeapon()

  const CANT_LISTEN_DURATION_MS = 15 * 60 * 1000;

  const [showAnswer, setShowAnswer] = useState(false)
  const network = useNetworkState()
  const [cardsBecameDue, setCardsBecameDue] = useState<number>(0)
  const [lastAutoPlayReviewCount, setLastAutoPlayReviewCount] = useState<bigint | null>(null)
  const [dismissedSetDisplayName, setDismissedSetDisplayName] = useState(() => {
    return localStorage.getItem('yap-skipped-set-display-name') === 'true'
  })

  const totalReviewsCompleted = deck.get_total_reviews()
  const autoplayed = lastAutoPlayReviewCount == totalReviewsCompleted
  const setAutoplayed = useCallback(() => setLastAutoPlayReviewCount(totalReviewsCompleted), [totalReviewsCompleted])

  const nextDueCard = findNextDueCard(deck)

  // Update scheduled push notifications and language stats when the deck state changes
  useEffect(() => {
    try {
      if (accessToken && userInfo?.id) {
        deck.submit_push_notifications(accessToken, userInfo?.id)
        deck.submit_language_stats(accessToken)
      }
    }
    catch {
      console.error("An error occurred when trying to update the notification schedule or language stats");
    }
  }, [deck, userInfo?.id, accessToken])

  // Schedule re-render when next card becomes due
  useEffect(() => {
    const next_due_timestamp_ms = nextDueCard?.due_timestamp_ms;
    if (next_due_timestamp_ms) {
      const timeUntilDueMs = next_due_timestamp_ms - Date.now();

      if (timeUntilDueMs > 0 && timeUntilDueMs < 24 * 60 * 60 * 1000) { // Only schedule if within 24 hours
        const timeout = setTimeout(() => {
          setCardsBecameDue(cardsBecameDue => cardsBecameDue + 1000)
        }, timeUntilDueMs + 1)

        return () => clearTimeout(timeout)
      }
    }
  }, [nextDueCard?.due_timestamp_ms])

  const [bannedChallengeTypes, setBannedChallengeTypes] = useState<ChallengeRequirements[]>(() => {
    const banned: ChallengeRequirements[] = [];
    
    const cantListenTimestamp = localStorage.getItem('yap-cant-listen-timestamp');
    if (cantListenTimestamp) {
      const timestamp = parseInt(cantListenTimestamp);
      const elapsed = Date.now() - timestamp;

      if (elapsed < CANT_LISTEN_DURATION_MS) {
        banned.push('Listening');
      } else {
        localStorage.removeItem('yap-cant-listen-timestamp');
      }
    }
    
    const cantSpeakTimestamp = localStorage.getItem('yap-cant-speak-timestamp');
    if (cantSpeakTimestamp) {
      const timestamp = parseInt(cantSpeakTimestamp);
      const elapsed = Date.now() - timestamp;

      if (elapsed < CANT_LISTEN_DURATION_MS) {
        banned.push('Speaking');
      } else {
        localStorage.removeItem('yap-cant-speak-timestamp');
      }
    }
    
    return banned;
  });

  useEffect(() => {
    const timeouts: any = [];
    
    if (bannedChallengeTypes.includes('Listening')) {
      const cantListenTimestamp = localStorage.getItem('yap-cant-listen-timestamp');
      if (cantListenTimestamp) {
        const timestamp = parseInt(cantListenTimestamp);
        const elapsed = Date.now() - timestamp;
        const remaining = CANT_LISTEN_DURATION_MS - elapsed;

        if (remaining > 0) {
          const timeout = setTimeout(() => {
            setBannedChallengeTypes(banned => banned.filter(t => t !== 'Listening'));
            localStorage.removeItem('yap-cant-listen-timestamp');
          }, remaining);
          timeouts.push(timeout);
        } else {
          setBannedChallengeTypes(banned => banned.filter(t => t !== 'Listening'));
          localStorage.removeItem('yap-cant-listen-timestamp');
        }
      }
    }
    
    if (bannedChallengeTypes.includes('Speaking')) {
      const cantSpeakTimestamp = localStorage.getItem('yap-cant-speak-timestamp');
      if (cantSpeakTimestamp) {
        const timestamp = parseInt(cantSpeakTimestamp);
        const elapsed = Date.now() - timestamp;
        const remaining = CANT_LISTEN_DURATION_MS - elapsed;

        if (remaining > 0) {
          const timeout = setTimeout(() => {
            setBannedChallengeTypes(banned => banned.filter(t => t !== 'Speaking'));
            localStorage.removeItem('yap-cant-speak-timestamp');
          }, remaining);
          timeouts.push(timeout);
        } else {
          setBannedChallengeTypes(banned => banned.filter(t => t !== ('Speaking' as any)));
          localStorage.removeItem('yap-cant-speak-timestamp');
        }
      }
    }
    
    return () => timeouts.forEach((timeout: any) => clearTimeout(timeout));
  }, [bannedChallengeTypes, CANT_LISTEN_DURATION_MS]);

  const reviewInfo = useMemo(() => {
    const now = Date.now();
    return deck.get_review_info(bannedChallengeTypes, now)
    // cardsBecameDue is intentionally included to trigger recalculation when cards become due
  }, [deck, bannedChallengeTypes, cardsBecameDue]);

  useInterval(() => setCardsBecameDue(cardsBecameDue => cardsBecameDue + 1), reviewInfo.due_count === 0 ? 1000 : 60000);

  const currentChallenge: Challenge<string> | undefined = useMemo(() => reviewInfo.get_next_challenge(deck), [reviewInfo, deck]);
  const addCardOptionsRaw = deck.add_card_options(bannedChallengeTypes);
  const addCardOptions: AddCardOptions = userInfo === undefined
    ? { smart_add: 0, manual_add: addCardOptionsRaw.manual_add.map(([count, card_type]) => [card_type == "TargetLanguage" || card_type == "LetterPronunciation" ? count : 0, card_type] as [number, CardType]) }
    : addCardOptionsRaw;

  useEffect(() => {
    const abortController = new AbortController();

    deck.cache_challenge_audio(accessToken, abortController.signal);

    return () => {
      abortController.abort();
    };
  }, [deck, accessToken, reviewInfo])


  const addNextCards = useCallback(async (card_type: CardType | undefined, count: number) => {
    const event = deck.add_next_unknown_cards(card_type, count, bannedChallengeTypes);
    if (event) {
      weapon.add_deck_event(event);
    }
  }, [deck, weapon, bannedChallengeTypes])

  const handleRating = async (rating: Rating) => {
    if (!currentChallenge || currentChallenge.type !== 'FlashCardReview') {
      console.error("handleRating called with no current challenge or no FlashCardReview in current challenge");
      return
    };

    // Play sound effect in background based on rating
    if (rating === 'again') {
      playSoundEffect('fail'); // Don't await - play in background
    } else {
      playSoundEffect('success'); // Don't await - play in background
    }

    const event = deck.review_card(currentChallenge.indicator, rating);
    if (event) {
      weapon.add_deck_event(event);
      setShowAnswer(false);
    }
  }

  const handleTranslationComplete = useCallback(async (grade: { wordStatuses: [Lexeme<string>, boolean | null][] } | { perfect: string | null }, wordsTapped: Lexeme<string>[], submission: string) => {
    if (!currentChallenge || currentChallenge.type !== 'TranslateComprehensibleSentence') {
      console.error("handleTranslationComplete called with no current challenge or no TranslateComprehensibleSentence in current challenge");
      return
    };

    // Play success sound in background for sentence completion (regardless of perfect or errors)
    playSoundEffect('success'); // Don't await - play in background

    if ("perfect" in grade) {
      // Perfect sentence review
      const event = deck.translate_sentence_perfect(wordsTapped, currentChallenge.target_language);
      if (event) {
        weapon.add_deck_event(event);
      }
      setShowAnswer(false);
    } else {
      // Wrong sentence review with word statuses
      const wordsRemembered: Lexeme<string>[] = [];
      const wordsForgotten: Lexeme<string>[] = [];

      grade.wordStatuses.forEach(([word, status]) => {
        if (status === true) {
          wordsRemembered.push(word);
        } else if (status === false) {
          wordsForgotten.push(word);
        }
      });

      const event = deck.translate_sentence_wrong(
        currentChallenge.target_language,
        submission,
        wordsRemembered,
        wordsForgotten,
        wordsTapped
      );
      if (event) {
        weapon.add_deck_event(event);
      }
      setShowAnswer(false);
    }
  }, [deck, currentChallenge, weapon])

  const handleTranscriptionComplete = useCallback((grade: /* comes from TranscriptionChallenge*/ PartGraded[]) => {
    if (!currentChallenge || currentChallenge.type !== 'TranscribeComprehensibleSentence') {
      console.error("handleTranscriptionComplete called with no current challenge or no TranscribeComprehensibleSentence in current challenge");
      return
    };

    // Play success sound in background for sentence completion (regardless of perfect or errors)
    playSoundEffect('success'); // Don't await - play in background

    const event = deck.transcribe_sentence(grade);
    if (event) {
      weapon.add_deck_event(event);
    }
    setShowAnswer(false)
  }, [deck, currentChallenge, weapon])

  const toggleAnswer = () => {
    setShowAnswer(!showAnswer)
  }

  const handleCantListen = () => {
    const timestamp = Date.now();
    localStorage.setItem('yap-cant-listen-timestamp', timestamp.toString());
    setBannedChallengeTypes(banned => banned.includes('Listening') ? banned : [...banned, 'Listening']);
  }
  
  const handleCantSpeak = () => {
    const timestamp = Date.now();
    localStorage.setItem('yap-cant-speak-timestamp', timestamp.toString());
    setBannedChallengeTypes(banned => banned.includes('Speaking') ? banned : [...banned, 'Speaking']);
  }

  useEffect(() => {
    const handleKeyPress = (event: KeyboardEvent) => {
      // Don't handle shortcuts if user is typing in an input field
      const target = event.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') {
        return;
      }

      if (event.code === 'Space' || event.code === 'Enter') {
        if (deck.num_cards() === 0) {
          event.preventDefault();
          addNextCards(undefined, 1);
        } else if (reviewInfo.due_count === 0 && !currentChallenge) {
          event.preventDefault();
          if (addCardOptions.smart_add > 0) {
            addNextCards(undefined, addCardOptions.smart_add);
          } else {
            for (const [count, card_type] of addCardOptions.manual_add) {
              if (card_type === "TargetLanguage") {
                addNextCards(card_type, count);
                break;
              }
            }
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyPress);

    return () => {
      window.removeEventListener('keydown', handleKeyPress);
    };
  }, [addNextCards, deck, reviewInfo, currentChallenge, addCardOptions.smart_add, addCardOptions.manual_add]);

  // Check if we should show the SetDisplayName prompt
  const shouldShowSetDisplayName =
    reviewInfo.due_count === 0 &&
    !currentChallenge &&
    totalReviewsCompleted >= 25n &&
    userInfo?.displayName === null &&
    network.online === true &&
    !dismissedSetDisplayName &&
    accessToken !== undefined;

  return (
    <>
      {/* main content */}
      <div className="flex flex-col flex-1 gap-2">
        {shouldShowSetDisplayName ? (
          <SetDisplayName
            accessToken={accessToken!}
            totalReviewsCompleted={totalReviewsCompleted}
            onComplete={() => setDismissedSetDisplayName(true)}
            onSkip={() => {
              localStorage.setItem('yap-skipped-set-display-name', 'true')
              setDismissedSetDisplayName(true)
            }}
          />
        ) : reviewInfo.due_count === 0 && !currentChallenge ? (
          <NoCardsReady
            nextDueCard={nextDueCard}
            addNextCards={addNextCards}
            showEngagementPrompts={reviewInfo.total_count > 5 && network.online === true && userInfo !== undefined}
            addCardOptions={addCardOptions}
            targetLanguage={targetLanguage}
            deck={deck}
          />
        ) : currentChallenge ? (
          (currentChallenge.type === 'FlashCardReview') ? (
            <Flashcard
              audioRequest={currentChallenge.audio}
              content={currentChallenge.content}
              isNew={currentChallenge.is_new}
              showAnswer={showAnswer}
              onToggle={toggleAnswer}
              totalCount={reviewInfo.total_count}
              onRating={handleRating}
              accessToken={accessToken}
              key={totalReviewsCompleted}
              onCantListen={handleCantListen}
              onCantSpeak={handleCantSpeak}
              targetLanguage={targetLanguage}
              listeningPrefix={currentChallenge.listening_prefix}
              autoplayed={autoplayed}
              setAutoplayed={setAutoplayed}
            />
          ) : (currentChallenge.type === 'TranslateComprehensibleSentence') ? (
            <TranslationChallenge
              sentence={currentChallenge}
              onComplete={handleTranslationComplete}
              accessToken={accessToken}
              key={totalReviewsCompleted}
              unique_target_language_lexeme_definitions={currentChallenge.unique_target_language_lexeme_definitions}
              targetLanguage={targetLanguage}
              nativeLanguage={nativeLanguage}
              autoplayed={autoplayed}
              setAutoplayed={setAutoplayed}
            />
          ) : (
            <TranscriptionChallenge
              challenge={currentChallenge}
              onComplete={handleTranscriptionComplete}
              totalCount={reviewInfo.total_count}
              accessToken={accessToken}
              key={totalReviewsCompleted}
              onCantListen={handleCantListen}
              targetLanguage={targetLanguage}
              nativeLanguage={nativeLanguage}
              autoplayed={autoplayed}
              setAutoplayed={setAutoplayed}
            />
          )
        ) : <div>Unexpected challenge state. This is a bug. currentChallenge: {JSON.stringify(currentChallenge)}</div>}
      </div>
      {/* /main content */}


    </>
  )
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/reset-password" element={<ResetPassword />} />
        <Route path="/confirm-email" element={<ConfirmEmail />} />
        <Route path="/accept-invite" element={<AcceptInvite />} />
        <Route path="/forgot-password" element={<ForgotPassword />} />
        <Route path="/*" element={<AppMain />}>
          <Route index element={<ReviewPage />} />
          <Route path="dictionary" element={<DictionaryPage />} />
          <Route path="leeches" element={<LeechesPage />} />
          <Route path="select-language" element={<SelectLanguagePage />} />
          <Route path="user/id/:id" element={<UserProfilePage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

function SelectLanguagePage() {
  const { userInfo } = useOutletContext<AppContextType>()
  const weapon = useWeapon()
  const deck = useDeck()
  const navigate = useNavigate()

  return match(deck)
    .with({ type: "deck", deck: P.not(P.nullish) }, ({ targetLanguage }) => (
      <LanguageSelector
        skipOnboarding={true}
        currentTargetLanguage={targetLanguage}
        showResumeButton={true}
        onResume={() => navigate('/')}
        onLanguagesConfirmed={(native, target) => {
          weapon.add_deck_selection_event({ SelectBothLanguages: { native, target } })
          navigate('/')
        }}
        userInfo={userInfo}
        onBack={() => navigate('/')}
      />
    ))
    .with({ type: "noLanguageSelected" }, () => (
      <LanguageSelector
        skipOnboarding={false}
        onLanguagesConfirmed={(native, target) => {
          weapon.add_deck_selection_event({ SelectBothLanguages: { native, target } })
          navigate('/')
        }}
        userInfo={userInfo}
      />
    ))
    .otherwise(() => (
      <TopPageLayout
        userInfo={userInfo}
        headerProps={{
          backButton: { label: 'Yap.Town', onBack: () => navigate('/') }
        }}
      >
        <div className="flex-1 flex items-center justify-center">
          <p className="text-muted-foreground animate-fade-in-delayed">Loading...</p>
        </div>
      </TopPageLayout>
    ))
}


function useDeck(): { type: "deck", nativeLanguage: Language, targetLanguage: Language, deck: Deck | null } | { type: "noLanguageSelected" } | { type: "error", message: string, retry: () => void, retryCount: number } | null {
  const weapon = useWeapon()
  const [retryCount, setRetryCount] = useState(0)

  useEffect(() => {
    weapon.request_deck_selection()
    weapon.request_reviews()
  }, [weapon])

  const getSnapshot = useCallback(() => {
    try {
      const num_reviews = weapon.get_stream_num_events("reviews")
      const num_deck_selection = weapon.get_stream_num_events("deck_selection")
      if (num_reviews === undefined || num_deck_selection === undefined) {
        return null
      }
      return num_reviews + num_deck_selection
    } catch {
      return null
    }
  }, [weapon])

  const subscribe = useCallback((callback: () => void) => {
    const handle_reviews = weapon.subscribe_to_stream("reviews", () => { callback() })
    const handle_deck_selection = weapon.subscribe_to_stream("deck_selection", () => { callback() })

    return () => {
      weapon.unsubscribe(handle_reviews)
      weapon.unsubscribe(handle_deck_selection)
    }
  }, [weapon])

  const numEvents = useSyncExternalStore(subscribe, getSnapshot)

  const retry = useCallback(() => {
    setRetryCount(count => count + 1)
  }, [])

  const state = useAsyncMemo(async () => {
    if (numEvents === null) {
      return null
    }

    const deck_selection = weapon.get_deck_selection_state()
    if (deck_selection === undefined || deck_selection === null) {
      return null
    }
    if (deck_selection.targetLanguage === undefined || deck_selection.targetLanguage === null || deck_selection.nativeLanguage === undefined || deck_selection.nativeLanguage === null) {
      return { type: "noLanguageSelected" } as { type: "noLanguageSelected" }
    } else {
      const course: Course = {
        nativeLanguage: deck_selection.nativeLanguage,
        targetLanguage: deck_selection.targetLanguage,
      }

      try {
        const languagePack = await weapon.get_language_pack(course)
        return {
          type: "deck",
          nativeLanguage: deck_selection.nativeLanguage,
          targetLanguage: deck_selection.targetLanguage,
          deck: await weapon.get_deck_state(languagePack, course),
        } as { type: "deck", nativeLanguage: Language, targetLanguage: Language, deck: Deck | null }
      } catch (error) {
        console.error("Failed to fetch language pack:", error)
        const errorMessage = error instanceof Error ? error.message : String(error)
        return {
          type: "error",
          message: errorMessage,
          retry,
          retryCount,
        } as { type: "error", message: string, retry: () => void, retryCount: number }
      }
    }
  }, [weapon, numEvents, retryCount])

  if (state?.type === "error" && state.retryCount < retryCount) {
    return null
  }

  return state ?? null
}

export default App
