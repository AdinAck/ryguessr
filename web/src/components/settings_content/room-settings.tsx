"use client";
import { Separator } from "../ui/separator";
import { Field } from "../ui/field";
import { Button } from "../ui/button";
import { useState } from "react";
import { Input } from "@/components/ui/input";

// Zustand useRoomSettings
import { useRoomSettings } from "@/store/useSettingsStore";

const RoomSettings = () => {
  const [roomCodeInput, setRoomCodeInput] = useState<string | undefined>(
    undefined,
  );

  const roomCode = useRoomSettings((state) => state.roomCode);

  return (
    <>
      <p className="text-xl font-semibold">Room</p>
      <Separator />

      <p className="text-7xl font-semibold tracking-wider">{roomCode}</p>
      <Separator />
      <form className="flex flex-col gap-3 w-full">
        <Field className="flex flex-row">
          <Input
            id="input-field-roomcode"
            name="room_code"
            type="text"
            placeholder="Enter Room Code"
            onChange={(e) => setRoomCodeInput(e.target.value)}
            className="shrink"
          />
        </Field>
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
      {/* <Field orientation="horizontal" className="w-fit"> */}
      {/*   <FieldLabel htmlFor="2fa">Multi-factor authentication</FieldLabel> */}
      {/*   <Switch id="2fa" /> */}
      {/* </Field> */}
    </>
  );
};

export default RoomSettings;
