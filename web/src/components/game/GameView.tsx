"use client";
import Streetview from "./StreetView";
import { MapOverlay } from "./MapOverlay";
import { useCallback, useState, useEffect, useMemo, useRef } from "react";
import { EventSource } from "eventsource";
import { Scoreboard } from "./Scoreboard";
import { Button } from "@/components/ui/button";
import Settings from "@/components/game/Settings";
// Type Data
import RoundData from "@/types/round-data";
import ScoreData from "@/types/score-data";
import RoundStart from "@/types/round-start";
import Coordinates from "@/types/coordinate_type";

// Settings icon
import { Settings2 } from "lucide-react";

// SSE Refresh
import { refreshSseEventStream, useSignal } from "@/lib/signal";

export const GameView = () => {
  const [panoId, setPanoId] = useState<string | undefined>(undefined);
  const [hasGuessed, setHasGuessed] = useState<boolean>(false);
  const [roundData, setRoundData] = useState<RoundData | undefined>(undefined);
  const [hasContinued, setHasContinued] = useState<boolean>(false);
  const [roundNumber, setRoundNumber] = useState<number>(1);
  const [shownScoreboard, setShownScoreboard] = useState<boolean>(false);
  const [showSettings, setShowSettings] = useState<boolean>(false);
  const es = useRef<EventSource | null>(null);

  const score_data = useMemo(() => {
    if (roundData) {
      const score_data: ScoreData = { player_scores: [] } as ScoreData;
      for (const entry of Object.values(roundData.player_results)) {
        score_data.player_scores.push({
          name: entry.player.username,
          last_score: entry.round_score,
          cum_score: entry.player.score,
        });
      }
      console.log(score_data);
      return score_data;
    }
    return null;
  }, [roundData]);

  const SSEConnect = useCallback(() => {
    if (es.current) {
      es.current.close();
    }
    es.current = new EventSource("/api/events", {
      fetch: (input, init) =>
        fetch(input, {
          ...init,
          headers: {
            ...init.headers,
          },
        }),
    });

    es.current.addEventListener("round-start", (event) => {
      const round_start: RoundStart = JSON.parse(event.data);
      console.log("round start");
      // console.log(round_start.round);
      // PanoIDUpdateAction.updatePanoID(round_start.pano_id);
      console.log(round_start.pano_id);
      setPanoId(round_start.pano_id);
      setRoundNumber(round_start.round);
      setHasGuessed(false);
      setHasContinued(false);
      setShownScoreboard(false);
      setRoundData(undefined);
    });

    es.current.addEventListener("round-end", (event) => {
      const round_data = JSON.parse(event.data);
      console.log(round_data);
      setRoundData(round_data);
    });
  }, []);

  useSignal(refreshSseEventStream, () => {
    SSEConnect();
  });

  useEffect(() => {
    SSEConnect();
    return () => {
      if (es.current) {
        es.current?.close();
        es.current = null;
      }
    };
  }, [SSEConnect]);

  const handleGuess = useCallback(
    async (guess: boolean, selectedLocation: Coordinates) => {
      setHasGuessed(guess);
      // console.log(selectedLocation);
      const response = await fetch("/api/guess", {
        method: "POST",
        body: JSON.stringify(selectedLocation),
        headers: {
          "Content-Type": "application/json",
        },
      });
      if (!response.ok) {
        console.log(response);
      }
    },
    [],
  );

  const handleContinue = useCallback(async () => {
    setHasContinued(true);
    const response = await fetch("/api/next", {
      method: "POST",
      headers: {},
    });
    if (!response.ok) {
      console.log(response);
    }
  }, []);

  const handleScoreboard = useCallback(() => {
    setShownScoreboard(true);
  }, []);

  return (
    <main className="bg-black relative h-dvh w-full overflow-hidden">
      <div className="absolute top-5 right-5 z-[99] bg-background border-1 border-white rounded-lg transition-all duration-100 active:scale-90">
        <Button
          variant={"outline"}
          onClick={() => {
            setShowSettings((prev) => !prev);
          }}
          size="icon"
          color="black"
          className="aspect-video origin-top-right"
        >
          <Settings2 color="white" />
        </Button>
      </div>
      {showSettings && (
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-[99]">
          <Settings />
        </div>
      )}

      <div
        className={`${shownScoreboard && score_data ? "blur-sm pointer-events-none" : undefined} absolute inset-0 z-0`}
      >
        {location ? (
          <Streetview panoId={panoId} />
        ) : (
          <p className="text-white absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
            Loading ....
          </p>
        )}
      </div>
      <div
        className={` bottom-5 left-5 flex flex-col gap-2 absolute z-10 aspect-video transition-all duration-300 origin-bottom-left 
          ${
            roundData && !shownScoreboard
              ? " w-[calc(100vw-2.5rem)] max-h-[calc(100vh-2.5rem)] opacity-100 ease-out"
              : shownScoreboard
                ? "w-40 md:w-64 lg:w-90 opacity-90 "
                : "w-40 md:w-64 lg:w-90 opacity-90 hover:w-[50vw] xl:hover:w-[40vw] ease-in-out"
          }`}
      >
        <MapOverlay
          key={`map-round${roundNumber}`}
          handleScoreboard={handleScoreboard}
          shownScoreboard={shownScoreboard}
          roundData={roundData}
          hasContinued={hasContinued}
          hasGuessed={hasGuessed}
          handleGuess={handleGuess}
          handleContinue={handleContinue}
        />{" "}
        {/*handleScoreboard={handleScoreboard} shownScoreboard={shownScoreboard} */}
      </div>
      {score_data && shownScoreboard && (
        <div className="overflow-hidden absolute transition-all duration-300 ease-in-out top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
          <Scoreboard score_data={score_data} />
        </div>
      )}
    </main>
  );
};
