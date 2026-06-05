"use client";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { MapPin } from "lucide-react";
import { ColorPicker, useColor } from "react-color-palette";
import "@/components/ui/react-color-palette.css";

// Zustand
import { useGameSession } from "@/store/useSettingsStore";
import { useShallow } from "zustand/shallow";
import { settingsStateActions } from "@/store/useSettingsStore";

import { X } from "lucide-react";

const UsernameSettings = ({
  handleColorRequest,
  handleUsernameRequest,
}: {
  handleColorRequest: (color: string) => void;
  handleUsernameRequest: (username: string) => void;
}) => {
  const { activeUsername, activeIconColor } = useGameSession(
    useShallow((state) => ({
      activeUsername: state.activeUsername,
      activeIconColor: state.activeIconColor,
    })),
  );

  const handleUsernameSubmit = (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const username = formData.get("username") as string;
    handleUsernameRequest(username);
  };

  const [color, setColor] = useColor(
    activeIconColor ? activeIconColor : "#FFFFFF",
  );

  return (
    <>
      <div className="flex flex-row justify-between items-center">
        <p className="text-xl font-semibold">User</p>
        <Button
          className="bg-transparent hover:cursor-pointer"
          onClick={() => settingsStateActions.updateSettingsVisibility(false)}
          variant={"secondary"}
        >
          <X color="white" />
        </Button>
      </div>
      <Separator />
      <form
        onSubmit={(e) => handleUsernameSubmit(e)}
        className="flex flex-col w-full gap-3"
      >
        <Field className="flex flex-row">
          <Input
            id="input-field-roomcode"
            defaultValue={activeUsername}
            name="username"
            type="text"
            className="shrink md:text-3xl font-semibold h-auto"
          />
        </Field>
        <Field orientation="horizontal">
          <Button type="submit" className={`w-full`}>
            Save Username
          </Button>
        </Field>
      </form>
      <Separator />
      <div className="flex flex-col justify-start items-center w-full h-full gap-3">
        <MapPin
          color={color.hex}
          strokeWidth={"1.5"}
          className="w-full min-h-32 max-h-44 md:max-h-64 md:min-h-44 lg:max-h-96 lg-min-h-64"
        />
        <div className="w-full">
          <ColorPicker
            color={color}
            onChange={setColor}
            hideAlpha={true}
            onChangeComplete={(color) => handleColorRequest(color.hex)}
          />
        </div>
      </div>
    </>
  );
};

export default UsernameSettings;
