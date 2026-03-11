"use client"
import Streetview from "./StreetView";
import { MapOverlay } from "./MapOverlay";
import { useCallback, useState, useEffect, useMemo, useRef } from "react"
import { EventSource } from "eventsource";
import { Scoreboard } from "./Scoreboard";
import { Field, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button";
import Settings from "@/components/game/Settings"
// Type Data
import RoundData from "@/types/round-data";
import ScoreData from "@/types/score-data";
import RoundStart from "@/types/round-start";
import Coordinates from "@/types/coordinate_type";

// Settings icon
import { Settings2 } from "lucide-react";

export const GameView = ({ userID }: { userID: string }) => {
  const [panoId, setPanoId] = useState<string | undefined>(undefined)
  const [hasGuessed, setHasGuessed] = useState<boolean>(false);
  const [roundData, setRoundData] = useState<RoundData | undefined>(undefined);
  const [hasContinued, setHasContinued] = useState<boolean>(false);
  const [roundNumber, setRoundNumber] = useState<number>(1);
  const [shownScoreboard, setShownScoreboard] = useState<boolean>(false);
  const [showSettings, setShowSettings] = useState<boolean>(false);

  // Room code
  const roomCode = useRef<string | undefined>(undefined);

  // User Name
  const [userName, setUserName] = useState<string | null>(null);

  const score_data = useMemo(() => {
    if (roundData) {
      const score_data: ScoreData = { player_scores: [] } as ScoreData;
      for (const [name, entry] of Object.entries(roundData.player_results)) {
        score_data.player_scores.push({ name: name, last_score: entry.last_score, cum_score: entry.cum_score });
      };
      return score_data;
    };
    return null;
  }, [roundData]);

  useEffect(() => {
    if (userName) {
      const es = new EventSource('/api/events', {
        fetch: (input, init) =>
          fetch(input, {
            ...init,
            headers: {
              ...init.headers,
              "Client-Id": userID,
            },
          })
      });

      es.addEventListener('round-start', (event) => {
        const round_start: RoundStart = JSON.parse(event.data);
        console.log("round start")
        console.log(round_start.round);
        setPanoId(round_start.pano_id);
        setRoundNumber(round_start.round);
        setHasGuessed(false);
        setHasContinued(false);
        setShownScoreboard(false);
        setRoundData(undefined);
      })


      es.addEventListener('round-end', (event) => {
        const round_data = JSON.parse(event.data);
        // console.log(event);
        console.log(round_data);
        setRoundData(round_data);
      })
      return () => {
        es.close();
      };
    };



  }, [userName]);

  const handleSubmit = async (event: React.SubmitEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);

    const userName = formData.get("username") as string;
    if (!userName || !userName.trim()) return;
    const response = await fetch("/api/init", {
      method: 'POST',
      body: `"${userName}"`,
      headers: {
        'Content-Type': 'application/json',
        'Client-Id': userID
      },
    });
    if (!response.ok) {
      console.log(response);
      console.log("Error posting username");
    } else {
      const room_code = await response.json();
      roomCode.current = room_code;
      console.log(room_code);
      setUserName(userName);
    };
  };

  const handleGuess = useCallback(async (guess: boolean, selectedLocation: Coordinates) => {
    setHasGuessed(guess);
    // console.log(selectedLocation);
    const response = await fetch('/api/guess', {
      method: 'POST',
      body: JSON.stringify(selectedLocation),
      headers: {
        'Content-Type': 'application/json',
        'Client-Id': userID,
      },
    });
    if (!response.ok) {
      console.log(response);
    };
  }, []);


  const handleContinue = useCallback(async () => {
    setHasContinued(true);
    const response = await fetch('/api/next', {
      method: 'POST',
      headers: {

        "Client-Id": userID,
      },
    });
    if (!response.ok) {
      console.log(response);
    };
  }, []);


  const handleScoreboard = useCallback(() => {
    setShownScoreboard(true);
  }, [])

  return (
    <main className="bg-black relative h-dvh w-full overflow-hidden">
      <div className="absolute top-5 right-5 z-[99] bg-background border-1 border-white rounded-lg transition-all duration-100 active:scale-90">
        <Button variant={"outline"} onClick={() => { setShowSettings((prev) => !prev) }} size="icon" color="black" className="aspect-video origin-top-right">
          <Settings2 color="white" />
        </Button>
      </div>
      {showSettings && roomCode.current &&
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-[99]">
          <Settings room_code={roomCode.current} />
        </div>
      }

      {userName
        ?
        <div className={`${shownScoreboard && score_data ? 'blur-sm pointer-events-none' : undefined} absolute inset-0 z-0`}>
          {location
            ?
            <Streetview panoId={panoId} />
            :
            <p className="text-white absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">Loading ....</p>}
        </div>
        :
        <div className="overflow-hidden absolute transition-all duration-300 ease-in-out top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
          <form onSubmit={handleSubmit} className="flex flex-col gap-3">
            <Field>
              <FieldLabel className="text-white" htmlFor="input-field-username">Username</FieldLabel>
              <Input
                id="input-field-username"
                name="username"
                type="text"
                placeholder="Enter your username"
                className="text-white"
              />
            </Field>
            <Field orientation="horizontal">
              <Button type="submit">Submit</Button>
            </Field>

          </form>

        </div>
      }

      <div className={` bottom-5 left-5 flex flex-col gap-2 absolute z-10 aspect-video transition-all duration-300 origin-bottom-left 
          ${roundData && !shownScoreboard
          ? " w-[calc(100vw-2.5rem)] max-h-[calc(100vh-2.5rem)] opacity-100 ease-out"
          : shownScoreboard ? "w-40 md:w-64 lg:w-90 opacity-90 " : "w-40 md:w-64 lg:w-90 opacity-90 hover:w-[50vw] xl:hover:w-[40vw] ease-in-out"
        }`}>
        <MapOverlay key={`map-round${roundNumber}`} handleScoreboard={handleScoreboard} shownScoreboard={shownScoreboard} roundData={roundData} hasContinued={hasContinued} hasGuessed={hasGuessed} handleGuess={handleGuess} handleContinue={handleContinue} /> {/*handleScoreboard={handleScoreboard} shownScoreboard={shownScoreboard} */}

      </div>
      {score_data && shownScoreboard &&
        <div className="overflow-hidden absolute transition-all duration-300 ease-in-out top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
          <Scoreboard score_data={score_data} />
        </div>
      }

    </main>


  );
};
