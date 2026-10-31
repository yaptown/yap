import { t } from "@/lib/i18n";
import { lazy, Suspense, useState, type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { ChevronLeft, ChevronRight, CirclePlus } from "lucide-react";
import { toast } from "sonner";
import type {
  AssumedKnownScreenView,
  AssumedKnownWord,
  Deck,
  Language,
  PaginationView,
  YourCardsScreenView,
} from "../../../yap-frontend-rs/pkg";
import type { UserInfo } from "@/app/context";
import { DeckPage } from "@/app/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { CardSummaryList } from "@/browse/CardSummaryList";
import { useStudyScreenInputs } from "@/review/useStudyScreenInputs";
import { useWeapon } from "@/core/weapon";
import { Ruby, TargetLanguageText } from "@/components/TargetLanguageText";
import { AudioButton } from "@/audio/AudioButton";

const FrequencyKnowledgeChart = lazy(() =>
  import("./FrequencyKnowledgeChart").then((module) => ({
    default: module.FrequencyKnowledgeChart,
  })),
);

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
  const [page, setPage] = useState(0);
  const view = injectedView ?? deck.your_cards_view(page);
  const { timestamp_ms } = useStudyScreenInputs(!injectedView);
  return (
    <WordListLayout
      userInfo={userInfo}
      title={view.summary.title}
      label={view.summary.label}
      pagination={view.pagination}
      onPage={setPage}
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
  const [page, setPage] = useState(0);
  const view = injectedView ?? deck.assumed_known_view(page);
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
      pagination={view.pagination}
      onPage={setPage}
    >
      <p className="text-sm text-muted-foreground">{view.note}</p>
      <Card className="p-4 gap-4">
        <h2 className="text-lg font-semibold">
          {view.frequency_knowledge_chart_title}
        </h2>
        <Suspense
          fallback={
            <div className="h-[400px] flex items-center justify-center text-muted-foreground">
              {t("stats-loading-chart")}
            </div>
          }
        >
          <FrequencyKnowledgeChart
            points={view.frequency_knowledge_chart_data}
            targetLanguage={view.target_language}
          />
        </Suspense>
      </Card>
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
      <span
        className="shrink-0 text-xs tabular-nums text-muted-foreground"
        title={word.frequency_rank_accessibility_label}
        aria-label={word.frequency_rank_accessibility_label}
      >
        {word.frequency_rank_label}
      </span>
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
  pagination,
  onPage,
  children,
}: {
  userInfo: UserInfo | undefined;
  title: string;
  label: string;
  pagination: PaginationView | undefined;
  onPage: (page: number) => void;
  children: ReactNode;
}) {
  const navigate = useNavigate();
  const goTo = (page: number) => {
    onPage(page);
    window.scrollTo({ top: 0 });
  };
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
        {pagination && <Pagination view={pagination} onPage={goTo} />}
      </main>
    </TopPageLayout>
  );
}

function Pagination({
  view,
  onPage,
}: {
  view: PaginationView;
  onPage: (page: number) => void;
}) {
  return (
    <nav className="flex items-center justify-center gap-2">
      <Button
        variant="outline"
        size="icon"
        aria-label={view.previous_label}
        title={view.previous_label}
        disabled={view.previous === undefined}
        onClick={() => view.previous !== undefined && onPage(view.previous)}
      >
        <ChevronLeft />
      </Button>
      <Select
        value={String(view.page)}
        onValueChange={(page) => onPage(Number(page))}
      >
        <SelectTrigger className="min-w-40 justify-between">
          <SelectValue>{view.status}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          {view.options.map((option) => (
            <SelectItem key={option.page} value={String(option.page)}>
              <span className="tabular-nums">{option.label}</span>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Button
        variant="outline"
        size="icon"
        aria-label={view.next_label}
        title={view.next_label}
        disabled={view.next === undefined}
        onClick={() => view.next !== undefined && onPage(view.next)}
      >
        <ChevronRight />
      </Button>
    </nav>
  );
}
