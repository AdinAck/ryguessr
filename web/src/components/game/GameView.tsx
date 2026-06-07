"use client";
import Streetview from "./StreetView";
import { MapOverlay } from "./MapOverlay";
import { useCallback, useState, useEffect, useMemo, useRef } from "react";
import { EventSource } from "eventsource";
import { Scoreboard } from "./Scoreboard";
import { Button } from "@/components/ui/button";
import Settings from "@/components/game/Settings";
import { AnimatePresence, isEasingArray } from "framer-motion";
// Type Data
import RoundData from "@/types/round-data";
import ScoreData from "@/types/score-data";
import RoundStart from "@/types/round-start";
import Coordinates from "@/types/coordinate_type";

// Lucide icons
import { Settings2 } from "lucide-react";

// Signals
import {
  refreshSseEventStream,
  useSignal,
  refreshPlayerList,
} from "@/lib/signal";
import PlayerScore from "@/types/player-score";

// Zustand
import {
  useSettingsState,
  settingsStateActions,
} from "@/store/useSettingsStore";
import PlayerData from "@/types/player-data";
import PlayerLeave from "@/types/player-leave";
import { AutoDismissAlert } from "./AlertManager";
import NotificationItem from "@/types/notification-item";

export const GameView = () => {
  const [panoId, setPanoId] = useState<string | undefined>(undefined);
  const [hasGuessed, setHasGuessed] = useState<boolean>(false);
  const [roundData, setRoundData] = useState<RoundData | undefined>(undefined);
  const [hasContinued, setHasContinued] = useState<boolean>(false);
  const [roundNumber, setRoundNumber] = useState<number>(1);
  const [shownScoreboard, setShownScoreboard] = useState<boolean>(false);
  const [notifications, addNotifications] = useState<NotificationItem[]>([]);
  const [mapExpanded, setMapExpanded] = useState<boolean>(false);
  const mapRef = useRef<HTMLDivElement>(null);

  const es = useRef<EventSource | null>(null);

  const settingsVisibility = useSettingsState(
    (state) => state.settingsVisibility,
  );

  const score_data = useMemo(() => {
    if (roundData) {
      const score_data: ScoreData = { player_scores: [] } as ScoreData;
      for (const entry of Object.values(roundData.player_results)) {
        score_data.player_scores.push({
          name: entry.player.username,
          last_score: entry.round_score,
          cum_score: entry.player.score,
          IconColor: entry.player.color,
        });
      }
      score_data.player_scores = score_data.player_scores.sort(
        (a: PlayerScore, b: PlayerScore): number => {
          return Number(b.cum_score) - Number(a.cum_score);
        },
      );
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

    es.current.addEventListener("player-joined", (event) => {
      const playerJoinData: PlayerData = JSON.parse(event.data);
      addNotifications((prev) => [
        ...prev,
        {
          id: playerJoinData.username,
          title: "Player Join",
          description: `${playerJoinData.username} has joined the Room!`,
        },
      ]);
      refreshPlayerList.emit();
    });

    es.current.addEventListener("player-left", (event) => {
      const playerLeaveData: PlayerLeave = JSON.parse(event.data);
      addNotifications((prev) => [
        ...prev,
        {
          id: playerLeaveData.username,
          title: "Player Left",
          description: `${playerLeaveData.username} has left the Room!`,
        },
      ]);

      refreshPlayerList.emit();
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
    setMapExpanded(false);
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

  const removeNotification = (id: string) => {
    addNotifications((prev) => prev.filter((noti) => noti.id !== id));
  };

  useEffect(() => {
    function handleClickOutside(event: PointerEvent) {
      if (mapRef.current && !mapRef.current.contains(event.target as Node)) {
        setMapExpanded(false);
      }
    }

    if (mapExpanded) {
      document.addEventListener("pointerdown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("pointerdown", handleClickOutside);
    };
  }, [mapExpanded]);

  const handleSetMapExpanded = (mapExpanded: boolean) => {
    setMapExpanded(mapExpanded);
  };

  return (
    <main className="bg-black relative h-dvh w-full overflow-hidden">
      <div className="absolute top-5 right-5 z-[99] bg-background border-1 border-white rounded-lg transition-all duration-100 active:scale-90">
        <Button
          variant={"outline"}
          onClick={() => {
            settingsStateActions.updateSettingsVisibility(!settingsVisibility);
          }}
          size="icon"
          color="black"
          className="aspect-video origin-top-right"
        >
          <Settings2 color="white" />
        </Button>
      </div>
      <div className="absolute flex flex-col gap-3 pointer-events-none top-5 left-5 z-[99] rounded-lg">
        <AnimatePresence>
          {notifications.map((noti, index) => {
            return (
              <AutoDismissAlert
                key={noti.id}
                title={noti.title}
                description={noti.description}
                onDismiss={() => removeNotification(noti.id)}
              />
            );
          })}
        </AnimatePresence>
      </div>
      {settingsVisibility && (
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-[99]">
          <Settings />
        </div>
      )}

      <div
        className={`${shownScoreboard && score_data ? "blur-sm pointer-events-none" : undefined} absolute inset-0 z-0`}
      >
        {location ? (
          <>
            {mapExpanded && (
              <div className="sm:hidden absolute inset-0 z-10 cursor-default bg-transparent" />
            )}

            <Streetview panoId={panoId} />
          </>
        ) : (
          <p className="text-white absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
            Loading ....
          </p>
        )}
      </div>
      <div
        ref={mapRef}
        className={` bottom-5 left-5 flex flex-col gap-2 absolute z-10 sm:aspect-video transition-all duration-300 origin-bottom-left 
          ${
            roundData && !shownScoreboard
              ? " w-[calc(100vw-2.5rem)] max-h-[calc(100vh-2.5rem)] opacity-100 ease-out"
              : shownScoreboard
                ? "w-40 md:w-64 lg:w-90 opacity-90 "
                : mapExpanded
                  ? "w-[calc(100vw-2.5rem)] h-[50dvh] opacity-100, ease-in-out"
                  : "w-40 h-28 sm:h-auto md:w-64 lg:w-90 opacity-90 sm:hover:w-[50vw] xl:hover:w-[40vw] ease-in-out"
          }`}
      >
        <MapOverlay
          key={`map-round${roundNumber}`}
          mapExpanded={mapExpanded}
          handleScoreboard={handleScoreboard}
          shownScoreboard={shownScoreboard}
          roundData={roundData}
          hasContinued={hasContinued}
          hasGuessed={hasGuessed}
          handleGuess={handleGuess}
          handleContinue={handleContinue}
          handleSetMapExpanded={handleSetMapExpanded}
        />{" "}
      </div>
      {score_data && shownScoreboard && (
        <div className="overflow-hidden absolute transition-all duration-300 ease-in-out top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
          <Scoreboard score_data={score_data} />
        </div>
      )}
    </main>
  );
};
