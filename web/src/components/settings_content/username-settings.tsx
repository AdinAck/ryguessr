"use client"
import { Separator } from "@/components/ui/separator";
import { Button } from "../ui/button";
import { Field } from "../ui/field";
import { Input } from "../ui/input";
import { MapPin } from 'lucide-react';
import { ColorPicker, useColor } from "react-color-palette";
import "@/components/ui/react-color-palette.css";
// import "react-color-palette/css";

// Zustand
import { useUserSettings } from "@/store/useSettingsStore";
import { userStateActions } from "@/store/useSettingsStore";



const UsernameSettings = () => {
  const { username, iconColor } = useUserSettings();
  const [color, setColor] = useColor(iconColor ? iconColor : "#FFFFFF");

  return (
    <>
      <p className="text-xl font-semibold">User</p>
      <Separator />
      <form className="flex flex-col w-full gap-3">
        <Field className="flex flex-row">
          <Input
            id="input-field-roomcode"
            defaultValue={username}
            name="room_code"
            type="text"
            className="shrink md:text-3xl font-semibold h-auto"
          />


        </Field>
        <Field orientation="horizontal">
          <Button type="submit" className={`w-full`}>Save Username</Button>
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
            color={color} onChange={setColor} hideAlpha={true} onChangeComplete={(color) => userStateActions.setIconColor(color.hex)}
          />
        </div>
      </div>
    </>
  );
};

export default UsernameSettings;
