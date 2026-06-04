"use client";
import { Separator } from "../ui/separator";
import { Item, ItemContent, ItemTitle } from "../ui/item";
import { MapPin } from "lucide-react";
import { Field } from "../ui/field";
import { Button } from "../ui/button";
import { useState } from "react";
import { Input } from "@/components/ui/input";

// Zustand useRoomSettings
import {
  useGameSession,
  playerActions,
  RoomSettingsActions,
  useRoomSettings,
} from "@/store/useSettingsStore";

// Error Code enum
import StatusCodes from "@/types/status-codes";

// RoomPeek reseponse type
import PlayerData from "@/types/player-data";

// SSE Refresh
import { refreshSseEventStream } from "@/lib/signal";

const RoomSettings = () => {
  const roomCode = useRoomSettings((state) => state.roomCode);
  const [roomCodeInput, setRoomCodeInput] = useState<string>(roomCode);
  const [requestStatusCode, setRequestStatusCode] = useState<StatusCodes>(
    StatusCodes.OK,
  );
  const [playerData, setPlayerData] = useState<PlayerData[]>([]);

  const username = useGameSession((state) => state.activeUsername);

  const roomPeek = async (roomCode: string) => {
    try {
      const response = await fetch(`api/room/${roomCode}`, {
        method: "GET",
      });
      setRequestStatusCode(response.status);
      if (!response.ok) {
        console.log(response.status);
        throw new Error(`Error peeking into room. Status: ${response.status} `);
      } else {
        const roomData: PlayerData[] = await response.json();
        setPlayerData(roomData);
      }
    } catch (error) {
      throw new Error(`Network error peeking into room: ${error}`);
    } finally {
    }
  };

  const handleRoomJoinRequest = async (requestedRoomCode: string) => {
    try {
      const response = await fetch("api/join", {
        method: "POST",
        body: JSON.stringify(requestedRoomCode),
        headers: {
          "Content-Type": "application/json",
        },
      });
      setRequestStatusCode(response.status);
      if (!response.ok) {
        throw new Error(`Room join request failed status: ${response.status}`);
      }
      const joinResponse: PlayerData[] = await response.json();
      RoomSettingsActions.updateRoomCode(requestedRoomCode);
      for (const value of Object.values(joinResponse)) {
        for (const [k, v] of Object.entries(value)) {
          if (k == "username" && v.toLowerCase() == username.toLowerCase()) {
            playerActions.saveCustomColor(value.color);
          }
        }
      }
      setPlayerData(joinResponse);
      refreshSseEventStream.emit();
      setRoomCodeInput(requestedRoomCode);
    } catch (error) {
      throw new Error(
        `Network error while attempting to join a room: ${error}`,
      );
    } finally {
    }
  };

  const tryRoomCode = (roomCodeCapture: string) => {
    setRoomCodeInput(roomCodeCapture);
    if (roomCodeCapture.length == 4 && roomCodeCapture != roomCode) {
      roomPeek(roomCodeCapture.toUpperCase());
    }
    // else if (roomCodeCapture.length == 3) {
    //   setPlayerData([]);
    // }
  };

  const handleRoomCodeSubmit = (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const submittedRoomCode = formData.get("room_code") as string;
    handleRoomJoinRequest(submittedRoomCode.toUpperCase());
  };
  return (
    <>
      <p className="text-xl font-semibold">Room</p>
      <Separator />
      <form onSubmit={(e) => handleRoomCodeSubmit(e)}>
        <Input
          className={`!bg-transparent not-focus:text-7xl tracking-wider border-none select-none focus-visible:ring-0 focus-visible:ring-offset-0 placeholder:text-7xl placeholder:text-white focus:text-7xl font-semibold h-fit max-w-full uppercase placeholder:tracking-wider 
        ${
          roomCodeInput == roomCode
            ? "text-green-500"
            : roomCodeInput && roomCodeInput.length <= 3
              ? `text-gray-600`
              : roomCodeInput.length == 4 && requestStatusCode == StatusCodes.OK
                ? `text-amber-400`
                : roomCodeInput.length == 4 &&
                    (StatusCodes.NOT_FOUND || StatusCodes.UNAUTHORIZED)
                  ? `text-red-400`
                  : undefined
        }`}
          id="input-field-roomcode"
          name="room_code"
          type="text"
          maxLength={4}
          defaultValue={roomCodeInput}
          value={roomCodeInput}
          enterKeyHint="enter"
          onBlur={() => {
            setRoomCodeInput(roomCode);
            setRequestStatusCode(StatusCodes.OK);
          }}
          onChange={(e) => tryRoomCode(e.target.value)}
        />
      </form>

      <Separator />
      <form className="flex flex-col gap-3 w-full">
        <Field orientation="horizontal">
          <Button type="submit" className={`w-full`}>
            Join Room
          </Button>
        </Field>
        <Field orientation="horizontal">
          <Button type="submit" className="w-full">
            Inactive
          </Button>
        </Field>
      </form>
      <Separator />
      {playerData.length > 0 && roomCodeInput.length == 4 && (
        <>
          <p className="text-lg font-medium">Players</p>
          <Separator />
          <div className="flex flex-col w-full">
            <Item variant="outline">
              <ItemContent className="flex flex-row justify-between">
                {playerData &&
                  playerData.map((player: PlayerData) => {
                    return (
                      <>
                        <div
                          key={player.username}
                          className="flex flex-row gap-3 text-xl items-center"
                        >
                          <MapPin
                            color={player.color}
                            strokeWidth={1.5}
                            size="1em"
                          />
                          <ItemTitle>{player.username}</ItemTitle>
                        </div>
                        <p className="text-gray-300">{player.score}</p>
                      </>
                    );
                  })}
              </ItemContent>
            </Item>
          </div>
        </>
      )}
    </>
  );
};

export default RoomSettings;
