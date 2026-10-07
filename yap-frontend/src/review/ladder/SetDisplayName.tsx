import { t } from "@/lib/i18n";
import { useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { Sparkles } from "lucide-react";

// Language learning themed name components
const ADJECTIVES = [
  "Fluent",
  "Polyglot",
  "Eloquent",
  "Bilingual",
  "Linguistic",
  "Verbal",
  "Articulate",
  "Multilingual",
  "Speaking",
  "Learning",
  "Studying",
  "Practicing",
  "Reading",
  "Writing",
  "Listening",
  "Translating",
  "Conversing",
  "Chatting",
  "Global",
  "Worldly",
  "Cultural",
  "International",
  "Traveling",
  "Curious",
  "Dedicated",
  "Persistent",
  "Motivated",
  "Eager",
  "Passionate",
  "Enthusiastic",
  "Ambitious",
  "Diligent",
  "Focused",
  "Committed",
  "Determined",
  "Aspiring",
  "Emerging",
  "Rising",
  "Growing",
  "Developing",
  "Advancing",
  "Progressing",
  "Native",
  "Expert",
  "Master",
  "Scholar",
  "Student",
  "Learner",
];

const NOUNS = [
  "Linguist",
  "Polyglot",
  "Scholar",
  "Student",
  "Learner",
  "Speaker",
  "Reader",
  "Writer",
  "Translator",
  "Interpreter",
  "Communicator",
  "Conversationalist",
  "Explorer",
  "Traveler",
  "Wanderer",
  "Adventurer",
  "Nomad",
  "Globetrotter",
  "Enthusiast",
  "Devotee",
  "Aficionado",
  "Fan",
  "Buff",
  "Lover",
  "Prodigy",
  "Genius",
  "Talent",
  "Wizard",
  "Expert",
  "Master",
  "Apprentice",
  "Novice",
  "Beginner",
  "Starter",
  "Rookie",
  "Freshman",
  "Teacher",
  "Tutor",
  "Coach",
  "Mentor",
  "Guide",
  "Instructor",
  "Champion",
  "Star",
  "Ace",
  "Pro",
  "Virtuoso",
  "Maestro",
];

function generateRandomName(): string {
  const adjective = ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
  const noun = NOUNS[Math.floor(Math.random() * NOUNS.length)];
  const number = Math.floor(Math.random() * 1000);
  return `${adjective}${noun}${number}`;
}

interface SetDisplayNameProps {
  onSave: (name: string) => Promise<void> | void;
  onSkip: () => void;
  totalReviewsCompleted: bigint;
}

export function SetDisplayName({
  onSave,
  onSkip,
  totalReviewsCompleted,
}: SetDisplayNameProps) {
  const [displayName, setDisplayName] = useState(generateRandomName);
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!displayName.trim()) {
      toast.error(t("set-display-name-please-enter-a-display-name"));
      return;
    }

    try {
      setSaving(true);
      await onSave(displayName.trim());
      toast.success(t("set-display-name-display-name-set"));
    } catch (err) {
      console.error("Error setting display name:", err);
      toast.error(
        t("set-display-name-failed-to-set-display-name-please-try-again"),
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="flex items-center justify-center w-full">
      <Card className="w-full">
        <CardHeader>
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-primary" />
            <CardTitle>{t("home-choose-your-display-name")}</CardTitle>
          </div>
          <CardDescription>
            {t("profile-display-name-prompt", {
              count: String(totalReviewsCompleted),
            })}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="displayName">
              {t("user-profile-display-name")}
            </Label>
            <Input
              id="displayName"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder={t("user-profile-enter-your-display-name")}
              maxLength={50}
              disabled={saving}
            />
            <p className="text-xs text-muted-foreground">
              {t("set-display-name-you-can-change-this-at-any-time")}
            </p>
          </div>

          <div className="flex gap-2">
            <Button
              onClick={onSkip}
              variant="outline"
              className="flex-1"
              disabled={saving}
            >
              {t("common-skip")}
            </Button>
            <Button onClick={handleSave} className="flex-1" disabled={saving}>
              {saving ? t("common-saving") : t("common-save")}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
