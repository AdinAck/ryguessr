"use client";
import { useState, useEffect, useCallback } from "react";
import { Separator } from "../ui/separator";
import { Item, ItemContent, ItemTitle } from "../ui/item";
import { MapPin } from "lucide-react";
import { Field } from "../ui/field";
import { Button } from "../ui/button";
import { Input } from "@/components/ui/input";

import { X } from "lucide-react";

// Zustand useRoomSettings
import {
  playerActions,
  RoomSettingsActions,
  useRoomSettings,
  useGameSession,
  settingsStateActions,
} from "@/store/useSettingsStore";

// Error Code enum
import StatusCodes from "@/types/status-codes";

// RoomPeek reseponse type
import PlayerData from "@/types/player-data";

// Signals
import { useSignal, refreshPlayerList } from "@/lib/signal";
import { refreshSseEventStream } from "@/lib/signal";

const RoomSettings = () => {
  const roomCode = useRoomSettings((state) => state.roomCode);
  const [roomCodeInput, setRoomCodeInput] = useState<string>(roomCode);
  const [requestStatusCode, setRequestStatusCode] = useState<StatusCodes>(
    StatusCodes.OK,
  );
  const [playerData, setPlayerData] = useState<PlayerData[]>([]);
  const username = useGameSession((state) => state.activeUsername);

  const roomPeek = useCallback(async (roomCode: string) => {
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
  }, []);

  useEffect(() => {
    roomPeek(roomCode);
  }, [roomPeek]);

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
      const joinResponse: PlayerData = await response.json();
      playerActions.setActiveColor(joinResponse.color);
      RoomSettingsActions.updateRoomCode(requestedRoomCode);
      refreshSseEventStream.emit();
      setRoomCodeInput(requestedRoomCode);
      roomPeek(requestedRoomCode);
    } catch (error) {
      throw new Error(
        `Network error while attempting to join a room: ${error}`,
      );
    }
  };

  const tryRoomCode = (roomCodeCapture: string) => {
    setRoomCodeInput(roomCodeCapture);
    if (roomCodeCapture.length == 4 && roomCodeCapture != roomCode) {
      roomPeek(roomCodeCapture.toUpperCase());
    }
  };

  const handleRoomCodeSubmit = (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const submittedRoomCode = formData.get("room_code") as string;
    handleRoomJoinRequest(submittedRoomCode.toUpperCase());
  };

  useSignal(refreshPlayerList, () => {
    console.log("room refresh");
    roomPeek(roomCode);
  });

  return (
    <>
      <div className="flex flex-row justify-between items-center">
        <p className="text-xl font-semibold">Room</p>
        <Button
          className="bg-transparent hover:cursor-pointer border-1"
          onClick={() => settingsStateActions.updateSettingsVisibility(false)}
          variant={"secondary"}
        >
          <X color="white" />
        </Button>
      </div>
      <Separator />
      <form
        className="flex flex-col gap-3 w-full"
        onSubmit={(e) => handleRoomCodeSubmit(e)}
      >
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
          onBlur={(e) => {
            if (e.relatedTarget && e.relatedTarget.id == "join-room-button") {
              return;
            }
            setRoomCodeInput(roomCode);
            setRequestStatusCode(StatusCodes.OK);
          }}
          onChange={(e) => tryRoomCode(e.target.value)}
        />

        <Separator />
        <div className="flex flex-col gap-3 w-full">
          <Field orientation="horizontal">
            <Button type="submit" className={`w-full`} id="join-room-button">
              Join Room
            </Button>
          </Field>
          <Field orientation="horizontal">
            <Button type="button" className="w-full">
              Inactive
            </Button>
          </Field>
        </div>
        <Separator />
      </form>

      {playerData.length > 0 && roomCodeInput.length == 4 && (
        <>
          <p className="text-lg font-medium">Players</p>
          <Separator />
          <div className="flex flex-col w-full gap-3 overflow-y-auto">
            {playerData &&
              playerData.map((player: PlayerData) => {
                return (
                  <Item
                    variant="outline"
                    className="flex flex-row justify-between items-center"
                    key={player.username}
                  >
                    <div>
                      <ItemContent
                        key={player.username}
                        className="flex flex-row gap-3 text-xl justify-between"
                      >
                        <MapPin
                          color={player.color}
                          strokeWidth={1.5}
                          size="1em"
                        />
                        <ItemTitle>
                          {player.username == username
                            ? player.username + " (ME)"
                            : player.username}
                        </ItemTitle>
                      </ItemContent>
                    </div>
                    <p className="text-gray-300">{player.score}</p>
                  </Item>
                );
              })}
          </div>
        </>
      )}
    </>
  );
};

export default RoomSettings;
