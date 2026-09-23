import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import {
  get_ai_server_url,
  mint_anki_deck,
  refresh_clip_manifest,
  type AnkiCardTypes,
  type Deck,
  type Language,
  type MintedAnkiDeck,
  type PlacementSession,
} from "../../../yap-frontend-rs/pkg";
import type { AppContextType } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useWeapon } from "@/core/weapon";
import { PlacementTest } from "@/review/ladder/PlacementTest";
import type { MediaProgress } from "./apkg";

// Stores the built package so it can be fetched by link (AnkiMobile's
// "Download link"). The backend keeps it for 8 days.
async function uploadPackage(minted: MintedAnkiDeck, languageCode: string, blob: Blob, accessToken?: string): Promise<string> {
  const query = new URLSearchParams({ d: minted.token, code: languageCode });
  const response = await fetch(`${get_ai_server_url()}/anki/deck/${encodeURIComponent(minted.deck_id)}/package?${query}`, {
    method: "PUT",
    headers: { Authorization: `Bearer ${accessToken ?? "anonymous"}`, "Content-Type": "application/octet-stream" },
    body: blob,
  });
  if (!response.ok) throw new Error(`${response.status} ${await response.text()}`);
  const { url } = (await response.json()) as { url: string };
  return url;
}

export function AnkiPage() {
  return <DeckPage prefetchAudio={false}>{(props) => <AnkiScreen key={props.targetLanguage} {...props} />}</DeckPage>;
}

function AnkiPlacement({ deck, targetLanguage }: { deck: Deck; targetLanguage: Language }) {
  const weapon = useWeapon();
  const [session, setSession] = useState<PlacementSession>(() => deck.start_placement_session());
  return <PlacementTest
    deck={deck}
    targetLanguage={targetLanguage}
    session={session}
    setSession={setSession}
    onComplete={({ known_words, unknown_words }) => {
      weapon.add_deck_event(deck.complete_placement_test(known_words, unknown_words));
    }}
  />;
}

function AnkiScreen({ deck, targetLanguage, userInfo, accessToken }: AppContextType & { deck: Deck; targetLanguage: Language }) {
  const navigate = useNavigate();
  const [size, setSize] = useState("300");
  const [cardTypes, setCardTypes] = useState<AnkiCardTypes>("Both");
  const [manifest, setManifest] = useState<"loading" | "ready" | "error">("loading");
  const [retry, setRetry] = useState(0);
  const [phase, setPhase] = useState<string>();
  const [progress, setProgress] = useState<MediaProgress>();
  const [result, setResult] = useState<string>();
  const [downloadLink, setDownloadLink] = useState<string>();
  const busy = phase !== undefined;
  const view = useMemo(() => ({ ...deck.anki_export_view(), manifest }), [deck, manifest]);
  const maximum = view.clip_sentence_count;
  const displayedSize = size !== "" && maximum > 0 ? String(Math.min(Number(size), maximum)) : size;

  useEffect(() => {
    let active = true;
    refresh_clip_manifest(targetLanguage, accessToken).then(() => {
      if (active) setManifest("ready");
    }).catch((error: unknown) => {
      if (active) {
        setManifest("error");
        toast.error(String(error));
      }
    });
    return () => { active = false; };
  }, [targetLanguage, accessToken, retry]);

  async function download() {
    setPhase("Preparing deck…");
    setProgress(undefined);
    setResult(undefined);
    setDownloadLink(undefined);
    try {
      const options = { size: Number(displayedSize), card_types: cardTypes };
      const minted = await mint_anki_deck(options, accessToken);
      // Let the preparation status paint before entering the synchronous WASM planner.
      await new Promise((resolve) => setTimeout(resolve, 0));
      const plan = deck.anki_deck_plan(options, minted.token, Date.now());
      const { buildApkg } = await import("./apkg");
      setPhase("Fetching media…");
      const blob = await buildApkg(plan, (source) => deck.anki_bundled_media(source), (next) => {
        setProgress(next);
        if (next.done === next.total) setPhase("Writing Anki package…");
      });
      setPhase("Uploading deck…");
      try {
        setDownloadLink(await uploadPackage(minted, plan.course_code, blob, accessToken));
      } catch (error) {
        toast.error(`Could not create a download link. Your deck will still download. ${String(error)}`);
      }
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `yap-${plan.course_code}.apkg`;
      document.body.append(anchor);
      anchor.click();
      anchor.remove();
      setTimeout(() => URL.revokeObjectURL(url), 60_000);
      setResult(`${plan.stats.sentence_count.toLocaleString()} sentence notes + ${plan.stats.word_count.toLocaleString()} word notes · ${(blob.size / 1024 / 1024).toFixed(1)} MB`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setPhase(undefined);
      setProgress(undefined);
    }
  }

  return (
    <TopPageLayout userInfo={userInfo} headerProps={{ backButton: { label: "Home", onBack: () => navigate("/home") } }}>
      <main className="mx-auto flex w-full max-w-2xl flex-col gap-6 px-5 py-8">
        <div className="flex flex-col gap-2">
          <p className="font-mono text-xs text-muted-foreground">Anki · {view.language_name}</p>
          <h1 className="text-2xl font-semibold">{view.title}</h1>
          <p className="text-muted-foreground">Movie sentences picked for your level, with clips, audio, and word definitions.</p>
        </div>
        {view.needs_placement ? (
          <div className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">First, find your starting level. No account needed.</p>
            <AnkiPlacement deck={deck} targetLanguage={targetLanguage} />
          </div>
        ) : (
          <form className="flex flex-col gap-6" onSubmit={(event) => { event.preventDefault(); void download(); }}>
            <div className="flex flex-col gap-2 border-y py-4">
              <p className="font-mono text-sm">{view.level_line}</p>
              {view.too_advanced_message && <p className="text-sm text-muted-foreground">{view.too_advanced_message}</p>}
            </div>
            <fieldset className="flex flex-col gap-5" disabled={busy}>
              <div className="flex flex-col gap-2">
                <Label htmlFor="anki-size">Sentence notes</Label>
                <Input id="anki-size" className="max-w-40" type="number" min={1} max={maximum || undefined} step={1} required value={displayedSize} onChange={(event) => setSize(event.target.value)} />
                <p className="text-sm text-muted-foreground">New words get their own notes, before the sentence that uses them. The final deck may be smaller if there aren’t enough suitable clips.</p>
              </div>
              <div className="flex flex-col gap-2">
                <span id="anki-types-label" className="text-sm font-medium">Sentence cards</span>
                <Tabs value={cardTypes} onValueChange={(value) => setCardTypes(value as AnkiCardTypes)}>
                  <TabsList aria-labelledby="anki-types-label">
                    <TabsTrigger value="Reading" disabled={busy}>Reading</TabsTrigger>
                    <TabsTrigger value="Listening" disabled={busy}>Listening</TabsTrigger>
                    <TabsTrigger value="Both" disabled={busy}>Both</TabsTrigger>
                  </TabsList>
                </Tabs>
              </div>
            </fieldset>
            <p className="text-sm text-muted-foreground">Every sentence and word recording is downloaded, so the cards work offline. Movie clips need an internet connection.</p>
            <p className="text-sm text-muted-foreground">Re-downloading updates matching notes. If you remove a card type, use Tools → Empty Cards in Anki to remove the old cards.</p>
            {view.manifest === "error" ? (
              <Button type="button" variant="outline" onClick={() => { setManifest("loading"); setRetry((value) => value + 1); }}>Retry loading movie clips</Button>
            ) : (
              <Button type="submit" disabled={busy || view.manifest !== "ready" || maximum === 0}>{view.download_label}</Button>
            )}
            <div className="flex flex-col gap-2 text-sm text-muted-foreground" role="status" aria-live="polite">
              {view.manifest === "loading" && <p>Loading movie clips…</p>}
              {view.manifest === "ready" && maximum === 0 && <p>No movie clips are available for this course.</p>}
              {busy && <p>{progress && progress.done < progress.total ? `Fetching media ${progress.done.toLocaleString()} of ${progress.total.toLocaleString()}` : phase}</p>}
              {busy && progress && <Progress value={progress.total ? 100 * progress.done / progress.total : 100} />}
              {downloadLink && <div className="flex flex-col gap-2">
                <Label htmlFor="anki-download-link">Download link</Label>
                <div className="flex gap-2">
                  <Input id="anki-download-link" readOnly value={downloadLink} className="min-w-0 font-mono" />
                  <Button type="button" variant="outline" onClick={async () => {
                    try {
                      await navigator.clipboard.writeText(downloadLink);
                      toast.success("Link copied");
                    } catch {
                      toast.error("Could not copy the link. Select and copy it manually.");
                    }
                  }}>Copy</Button>
                </div>
                <p>In AnkiMobile: Decks → Add → Download link. The link works for 8 days.</p>
              </div>}
              {result && <p>Downloaded {result}</p>}
            </div>
          </form>
        )}
      </main>
    </TopPageLayout>
  );
}
