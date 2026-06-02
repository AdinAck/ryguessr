"use client";
import NumberFlow, { continuous } from "@number-flow/react";
import ScoreData from "@/types/score-data";
import { memo } from "react";
import { CSSProperties, useState, useEffect } from "react";

export const Scoreboard = memo(function Scoreboard({
  score_data,
}: {
  score_data: ScoreData;
}) {
  const [animateToRealScore, setAnimateToRealScore] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setAnimateToRealScore(true);
    }, 300);

    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="flex flex-col gap-3 w-[50vw] h-full max-h-[40vh] overflow-y-auto pr-2">
      {score_data.player_scores.map((score) => {
        return (
          <div
            key={score.name}
            className="flex justify-between items-center w-full bg-black/80 border-2 rounded-lg py-2 text-white shadow-md"
          >
            <span className="text-sm px-2">{score.name}</span>
            <NumberFlow
              className="text-right px-2 text-sm font-semibold overflow-hidden"
              plugins={[continuous]}
              value={
                animateToRealScore
                  ? score.cum_score
                  : score.cum_score - score.last_score || 0
              }
              style={{ "--number-flow-mask-height": "0.3em" } as CSSProperties}
            />
          </div>
        );
      })}
    </div>
  );
});
