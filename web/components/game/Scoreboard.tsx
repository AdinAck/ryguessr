import { Table, TableHead, TableBody, TableCaption, TableCell, TableHeader, TableRow } from "../ui/table";
import ScoreData from "@/types/score-data";
import { memo } from "react";

export const Scoreboard = memo(({ score_data }: { score_data: ScoreData }) => {

  return (
    <div className="text-white rounded-lg border-2 w-[50vw] h-full overflow-hidden">
      <Table bgcolor="black" className="rounded-lg opacity-80">
        <TableBody>
          <TableRow className="bg-black/40">
            {score_data.player_scores.map((score, index) => {
              return (
                <>
                  <TableCell className="font-medium">{score.name}</TableCell>
                  <TableCell className="text-right">{score.cum_score}</TableCell>
                </>
              );
            })};
            <TableCell className="font-medium">Player1</TableCell>
            <TableCell className="text-right">250Pts</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>

  );

});
