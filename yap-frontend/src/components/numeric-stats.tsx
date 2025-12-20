import React from "react";
import { Card } from "@/components/ui/card";

interface NumericStatsProps {
  xp: number;
  totalCards: number;
  cardsReady: number;
  percentKnown: number;
  dailyStreak: number;
  totalReviews: number | bigint;
}

export const NumericStats: React.FC<NumericStatsProps> = ({
  xp,
  totalCards,
  cardsReady,
  percentKnown,
  dailyStreak,
  totalReviews,
}) => {
  return (
    <div className="mb-4">
      <h2 className="text-2xl font-semibold">Stats</h2>
      <div className="grid grid-cols-1 md:grid-cols-1 gap-4 mt-3">
        <Card className="p-4 gap-0">
          <p className="text-sm text-muted-foreground mb-1">XP</p>
          <p className="text-2xl font-bold">{xp}</p>
          <p className="text-sm text-muted-foreground mt-1">
            You get more XP for words you didn't remember.
          </p>
        </Card>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-3">
        <Card className="p-4 gap-0">
          <p className="text-sm text-muted-foreground mb-1">Total Cards</p>
          <p className="text-2xl font-bold">{totalCards}</p>
          <p className="text-sm text-muted-foreground mt-1">{cardsReady} ready now</p>
        </Card>
        <Card className="p-4 gap-0">
          <p className="text-sm text-muted-foreground mb-1">Words Known</p>
          <p className="text-2xl font-bold">{percentKnown.toFixed(2)}%</p>
          <p className="text-sm text-muted-foreground mt-1">of total</p>
        </Card>
        <Card className="p-4 gap-0">
          <p className="text-sm text-muted-foreground mb-1">Daily Streak</p>
          <p className="text-2xl font-bold">{dailyStreak}</p>
          <p className="text-sm text-muted-foreground mt-1">days</p>
        </Card>
        <Card className="p-4 gap-0">
          <p className="text-sm text-muted-foreground mb-1">Total Reviews</p>
          <p className="text-2xl font-bold">{totalReviews.toString()}</p>
          <p className="text-sm text-muted-foreground mt-1">all time</p>
        </Card>
      </div>
    </div>
  );
};
