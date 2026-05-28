"use client";
import { Separator } from "../ui/separator";
import { Item, ItemContent, ItemTitle } from "../ui/item";
import { MapPin } from "lucide-react";
import { Field } from "../ui/field";
import { Button } from "../ui/button";
import { useState } from "react";
import { Input } from "@/components/ui/input";

// Zustand useRoomSettings
import { useRoomSettings } from "@/store/useSettingsStore";
// import { RoomSettingsActions } from "@/store/useSettingsStore";

// Error Code enum
import StatusCodes from "@/types/status-codes";

// RoomPeek reseponse type
import PlayerData from "@/types/player-data";

const RoomSettings = () => {
  const roomCode = useRoomSettings((state) => state.roomCode);
  const [roomCodeInput, setRoomCodeInput] = useState<string | undefined>(
    roomCode,
  );
  const [requestStatusCode, setRequestStatusCode] = useState<StatusCodes>(
    StatusCodes.OK,
  );
  const [playerData, setPlayerData] = useState<PlayerData[]>([]);

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
        const playerData: PlayerData[] = await response.json();
        setPlayerData(playerData);
      }
    } catch (error) {
      throw new Error("Network error peeking into room");
    } finally {
    }
  };

  const tryRoomCode = (roomCode: string) => {
    setRoomCodeInput(roomCode);
    if (roomCode.length == 4) {
      roomPeek(roomCode);
    }
  };

  return (
    <>
      <p className="text-xl font-semibold">Room</p>
      <Separator />

      <Input
        className={`not-focus:text-7xl tracking-wider border-none select-none bg-black focus-visible:ring-0 focus-visible:ring-offset-0 placeholder:text-7xl placeholder:text-white focus:text-7xl font-semibold h-fit max-w-full uppercase placeholder:tracking-wider 
        ${
          roomCodeInput && roomCodeInput?.length <= 3
            ? `text-gray-600`
            : roomCodeInput?.length == 4 && requestStatusCode == StatusCodes.OK
              ? `text-amber-500`
              : roomCodeInput?.length == 4 &&
                  (StatusCodes.NOT_FOUND || StatusCodes.UNAUTHORIZED)
                ? `text-red-500`
                : undefined
        }`}
        id="input-field-roomcode"
        name="room_code"
        type="text"
        maxLength={4}
        defaultValue={roomCodeInput}
        value={roomCodeInput}
        onBlur={() => {
          setRoomCodeInput(roomCode);
          setRequestStatusCode(StatusCodes.OK);
        }}
        onChange={(e) => tryRoomCode(e.target.value)}
      />

      <Separator />
      <form className="flex flex-col gap-3 w-full">
        <Field orientation="horizontal">
          <Button type="submit" className={`w-full`}>
            Join Room
          </Button>
        </Field>
        <Field orientation="horizontal">
          <Button type="submit" className="w-full">
            New Room
          </Button>
        </Field>
      </form>
      <Separator />
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
  );
};

export default RoomSettings;
