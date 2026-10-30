import { t } from "@/lib/i18n";
import { useState, type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { CirclePlus } from "lucide-react";
import { toast } from "sonner";
import type {
  AssumedKnownScreenView,
  AssumedKnownWord,
  Deck,
  Language,
  YourCardsScreenView,
} from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { CardSummaryList } from "@/browse/CardSummaryList";
import { useStudyScreenInputs } from "@/review/useStudyScreenInputs";
import { useWeapon } from "@/core/weapon";
import { Ruby, TargetLanguageText } from "@/components/TargetLanguageText";
import { AudioButton } from "@/audio/AudioButton";

// Both lists can run to tens of thousands of words, so they grow a page at a time.
const PAGE_SIZE = 200;

export function YourCardsPage() {
  return (
    <DeckPage>
      {({ deck, userInfo }) => (
        <YourCardsScreen deck={deck} userInfo={userInfo} />
      )}
    </DeckPage>
  );
}

export function AssumedKnownPage() {
  return (
    <DeckPage>
      {({ deck, userInfo, accessToken }) => (
        <AssumedKnownScreen
          deck={deck}
          userInfo={userInfo}
          accessToken={accessToken}
        />
      )}
    </DeckPage>
  );
}

export function YourCardsScreen({
  deck,
  userInfo,
  view: injectedView,
}: {
  view?: YourCardsScreenView;
  deck: Deck;
  userInfo: UserInfo | undefined;
}) {
  const [limit, setLimit] = useState(PAGE_SIZE);
  const view = injectedView ?? deck.your_cards_view(limit);
  const { timestamp_ms } = useStudyScreenInputs(!injectedView);
  return (
    <WordListLayout
      userInfo={userInfo}
      title={view.summary.title}
      label={view.summary.label}
      showMore={view.show_more}
      onShowMore={() => setLimit((limit) => limit + PAGE_SIZE)}
    >
      {view.cards.length > 0 && (
        <CardSummaryList
          cards={view.cards}
          targetLanguage={view.target_language}
          timestampMs={timestamp_ms}
        />
      )}
    </WordListLayout>
  );
}

export function AssumedKnownScreen({
  deck,
  userInfo,
  accessToken,
  view: injectedView,
}: {
  view?: AssumedKnownScreenView;
  deck: Deck;
  userInfo: UserInfo | undefined;
  accessToken: string | undefined;
}) {
  const weapon = useWeapon();
  const [limit, setLimit] = useState(PAGE_SIZE);
  const view = injectedView ?? deck.assumed_known_view(limit);
  // An added word leaves the list once the next deck snapshot lands.
  const add = (word: AssumedKnownWord) => {
    const event = deck.add_gram_by_frequency_index(word.frequency_index);
    if (!event) return;
    weapon.add_deck_event(event);
    toast.success(t("dictionary-added-word", { word: word.display_text }));
  };
  return (
    <WordListLayout
      userInfo={userInfo}
      title={view.summary.title}
      label={view.summary.label}
      showMore={view.show_more}
      onShowMore={() => setLimit((limit) => limit + PAGE_SIZE)}
    >
      <p className="text-sm text-muted-foreground">{view.note}</p>
      {view.words.length > 0 && (
        <Card className="overflow-hidden p-0 gap-0">
          {view.words.map((word) => (
            <AssumedKnownRow
              key={word.frequency_index}
              word={word}
              language={view.target_language}
              accessToken={accessToken}
              onAdd={() => add(word)}
            />
          ))}
        </Card>
      )}
    </WordListLayout>
  );
}

function AssumedKnownRow({
  word,
  language,
  accessToken,
  onAdd,
}: {
  word: AssumedKnownWord;
  language: Language;
  accessToken: string | undefined;
  onAdd: () => void;
}) {
  return (
    <div className="flex items-center gap-3 p-3 border-b last:border-b-0">
      <AudioButton
        audioRequest={word.audio_request}
        accessToken={accessToken}
        className="h-8 w-8 shrink-0"
      />
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span className="font-medium break-words">
          <TargetLanguageText language={language}>
            {word.prefix && (
              <span className="text-muted-foreground/60">
                {word.prefix.prefix}
                {word.prefix.separator}
              </span>
            )}
            <Ruby segments={word.ruby}>{word.display_text}</Ruby>
          </TargetLanguageText>
        </span>
        {word.gloss && (
          <span className="text-sm text-muted-foreground break-words">
            {word.gloss}
          </span>
        )}
      </div>
      <Button variant="ghost" size="sm" className="shrink-0" onClick={onAdd}>
        <CirclePlus />
        {word.add_label}
      </Button>
    </div>
  );
}

function WordListLayout({
  userInfo,
  title,
  label,
  showMore,
  onShowMore,
  children,
}: {
  userInfo: UserInfo | undefined;
  title: string;
  label: string;
  showMore: string | undefined;
  onShowMore: () => void;
  children: ReactNode;
}) {
  const navigate = useNavigate();
  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: {
          label: t("common-stats"),
          onBack: () => navigate("/stats"),
        },
      }}
    >
      <main className="flex flex-col gap-4 py-4">
        <h1 className="text-2xl font-bold">{title}</h1>
        <p className="text-muted-foreground">{label}</p>
        {children}
        {showMore && (
          <Button variant="outline" className="self-center" onClick={onShowMore}>
            {showMore}
          </Button>
        )}
      </main>
    </TopPageLayout>
  );
}
