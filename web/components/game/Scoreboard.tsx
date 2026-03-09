import { Table, TableHead, TableBody, TableCaption, TableCell, TableHeader, TableRow } from "../ui/table";
import ScoreData from "@/types/score-data";
import { memo } from "react";

export const Scoreboard = memo(({ score_data }: { score_data: ScoreData }) => {

  return (
    <div className="flex flex-col gap-3 w-[50vw] h-full max-h-[40vh] overflow-y-auto pr-2">
      {score_data.player_scores.map((score, index) => {
        return (
          <div
            className="flex justify-between items-center w-full bg-black/80 border-2 rounded-lg p-2 text-white shadow-md transition-all"
          >
            <span className="text-sm">{score.name}</span>
            <span className="text-right text-sm">{score.cum_score}</span>
          </div>
        );
      })};

    </div>

  );

});
