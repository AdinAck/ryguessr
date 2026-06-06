"use client";
import NumberFlow, { continuous } from "@number-flow/react";
import ScoreData from "@/types/score-data";
import { memo } from "react";
import { CSSProperties, useState, useEffect } from "react";
import { MapPin } from "lucide-react";

import { useGameSession } from "@/store/useSettingsStore";

export const Scoreboard = memo(function Scoreboard({
  score_data,
}: {
  score_data: ScoreData;
}) {
  const [animateToRealScore, setAnimateToRealScore] = useState(false);
  const username = useGameSession((state) => state.activeUsername);

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
            className="py-3 px-3 flex justify-between items-center w-full bg-black/80 border-2 rounded-lg text-white shadow-md"
          >
            <div className="flex text-md flex-row items-center gap-3">
              <MapPin color={score.IconColor} strokeWidth={1.5} />
              <span className="flex items-center gap-1">
                {score.name === username ? (
                  <>
                    {score.name} <span className="font-bold">(ME)</span>
                  </>
                ) : (
                  score.name
                )}
              </span>
            </div>
            <NumberFlow
              className="text-right font-semibold overflow-hidden"
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
