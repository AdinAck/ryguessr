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
  const [isPeeking, setIsPeeking] = useState<boolean>(false);
  const [playerData, setPlayerData] = useState<PlayerData[]>([]);
  const username = useGameSession((state) => state.activeUsername);

  const roomPeek = useCallback(async (roomCode: string) => {
    setIsPeeking(true);
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
      setIsPeeking(false);
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

  const upperInput = roomCodeInput.toUpperCase();
  const isRightLength = upperInput.length === 4;
  const isNewRoom = upperInput !== roomCode;
  const isRoomValid = requestStatusCode === StatusCodes.OK && !isPeeking;

  const canJoinRoom = isRightLength && isNewRoom && isRoomValid;

  const tryRoomCode = (roomCodeCapture: string) => {
    const upperCode = roomCodeCapture.toUpperCase();
    setRoomCodeInput(upperCode);

    if (roomCodeCapture.length == 4) {
      roomPeek(upperCode);
    } else {
      setRequestStatusCode(StatusCodes.OK);
    }
  };

  const handleRoomCodeSubmit = (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!canJoinRoom) return;
    handleRoomJoinRequest(upperInput);
  };

  useSignal(refreshPlayerList, () => {
    console.log("room refresh");
    roomPeek(roomCode);
  });

  const getTextColorClass = () => {
    if (upperInput === roomCode) return "text-green-500";
    if (upperInput.length < 4 || isPeeking) return "text-gray-600";
    if (requestStatusCode === StatusCodes.OK) return "text-amber-400";
    return "text-red-400";
  };

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
          className={`!bg-transparent not-focus:text-6xl sm:not-focus:text-7xl tracking-wider border-none select-none focus-visible:ring-0 focus-visible:ring-offset-0 placeholder:text-6xl sm:placeholder:text-7xl placeholder:text-white focus:text-6xl sm:focus:text-7xl font-semibold h-fit max-w-full uppercase placeholder:tracking-wider ${getTextColorClass()}`}
          id="input-field-roomcode"
          name="room_code"
          type="text"
          maxLength={4}
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
            <Button
              type={`${!canJoinRoom ? `button` : `submit`}`}
              className={`w-full ${!canJoinRoom ? `cursor-not-allowed` : `cursor-pointer`}`}
              id="join-room-button"
            >
              Join Room
            </Button>
          </Field>
          <Field orientation="horizontal">
            <Button type="button" className="hover:cursor-pointer w-full">
              Inactive
            </Button>
          </Field>
        </div>
        <Separator />
      </form>

      {!isPeeking &&
        requestStatusCode == StatusCodes.OK &&
        playerData.length > 0 &&
        roomCodeInput.length == 4 && (
          <>
            <p className="text-lg font-medium">Players</p>
            <Separator />
            {playerData &&
              playerData.map((player: PlayerData) => {
                return (
                  <Item
                    variant="outline"
                    className="flex flex-row justify-between items-center flex-wrap h-fit"
                    key={player.username}
                  >
                    <div>
                      <ItemContent
                        key={player.username}
                        className="flex flex-row gap-3 justify-between items-center"
                      >
                        <MapPin
                          color={player.color}
                          strokeWidth={1.5}
                          className="text-xs sm:text-base"
                        />
                        <ItemTitle className="gap-1 text-xs sm:text-base">
                          {player.username == username ? (
                            <>
                              {player.username}
                              <span className="font-bold">(ME)</span>
                            </>
                          ) : (
                            player.username
                          )}
                        </ItemTitle>
                      </ItemContent>
                    </div>
                    <p className="text-gray-300 text-xs sm:text-base">
                      {player.score}
                    </p>
                  </Item>
                );
              })}
          </>
        )}
    </>
  );
};

export default RoomSettings;
