"use client"
import { memo, useState } from "react";
import { Toggle } from "@/components/ui/toggle"
import { Separator } from "../ui/separator";
import { KeyRound, MapPinPen, AudioLines, SquarePen } from "lucide-react";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Field, FieldLabel } from "../ui/field";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Copy } from 'lucide-react';

const Settings = memo(({ room_code }: { room_code: string }) => {
  const [roomCodeInput, setRoomCodeInput] = useState<string | undefined>(undefined);

  return (
    <div className="z-[99] w-md md:w-xl lg:w-2xl min-h-[50vh] flex flex-row border-2 rounded-lg bg-background">
      <div dir="ltr" className="flex flex-col grow-0.5 gap-2 p-2 rounded-s-lg" id="side-bar">
        <Toggle className="justify-start items-center">
          <KeyRound />
          Room
        </Toggle>
        <Toggle className="justify-start items-center">
          <MapPinPen />
          Streetview
        </Toggle>
        <Toggle className="justify-start items-center">
          <AudioLines />
          Audio
        </Toggle>
      </div>
      <div className="flex flex-col grow-2 gap-3 py-2 px-2 bg-black rounded-lg w-full" id="settings_body">
        <p className="text-md">Room</p>
        <Separator />

        <p className="text-7xl">3JK5</p>
        <Separator />
        <form className="flex flex-col gap-3 w-full">
          <Field className="flex flex-row">
            <Input
              id="input-field-roomcode"
              name="room_code"
              type="text"
              placeholder="Enter Room Code"
              className="shrink"
            />


          </Field>
          <Field orientation="horizontal">
            <Button type="submit" className="w-full">Join Room</Button>
          </Field>
          <Field orientation="horizontal">
            <Button type="submit" className="w-full">New Room</Button>
          </Field>

        </form>
        {/* <Field orientation="horizontal" className="w-fit"> */}
        {/*   <FieldLabel htmlFor="2fa">Multi-factor authentication</FieldLabel> */}
        {/*   <Switch id="2fa" /> */}
        {/* </Field> */}

      </div>
      <div>

      </div>

    </div>
  );
});

export default Settings;
