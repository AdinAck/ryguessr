"use client";
import { useState } from "react";
import { KeyRound, MapPinPen, AudioLines, UserRoundPen } from "lucide-react";
import { Button } from "../ui/button";
import RoomSettings from "../settings_content/room-settings";
import StreetviewSettings from "../settings_content/streetview-settings";
import UsernameSettings from "../settings_content/username-settings";
import AudioSettings from "../settings_content/audio-settings";

// Zustand
import { playerActions } from "@/store/useSettingsStore";

enum SettingsToggle {
  Room,
  Username,
  Streetview,
  Audio,
}

const Settings = () => {
  const [currentSettingToggle, setCurrentSettingToggle] =
    useState<SettingsToggle>(SettingsToggle.Room);

  const handleColorRequest = (color: string) => {
    playerActions.saveCustomColor(color);
    try {
      fetch("/api/color", {
        method: "POST",
        body: JSON.stringify(color),
        headers: {
          "Content-Type": "application/json",
        },
      });
    } catch (error) {
      console.log(error);
    } finally {
      console.log("Color Saved");
    }
  };

  const handleUsernameRequest = (username: string) => {
    playerActions.saveCustomUsername(username);
    try {
      fetch("api/username", {
        method: "POST",
        body: JSON.stringify(username),
        headers: {
          "Content-Type": "application/json",
        },
      });
    } catch (error) {
      console.log(error);
    } finally {
      console.log("Username Saved!");
    }
  };

  return (
    <div className="z-[99] w-sm md:w-xl lg:w-2xl h-[50dvh] min-h-[400px] max-h-[800px] flex flex-row border-2 rounded-lg bg-background ">
      <div
        dir="ltr"
        className="flex flex-col grow-0.5 gap-2 p-2 rounded-s-lg"
        id="side-bar"
      >
        <Button
          variant={
            currentSettingToggle === SettingsToggle.Room ? "secondary" : "ghost"
          }
          onClick={() => setCurrentSettingToggle(SettingsToggle.Room)}
          className="justify-start items-center"
        >
          <KeyRound />
          Room
        </Button>

        <Button
          variant={
            currentSettingToggle === SettingsToggle.Streetview
              ? "secondary"
              : "ghost"
          }
          onClick={() => setCurrentSettingToggle(SettingsToggle.Streetview)}
          className="justify-start items-center"
        >
          <MapPinPen />
          Streetview
        </Button>

        <Button
          variant={
            currentSettingToggle === SettingsToggle.Audio
              ? "secondary"
              : "ghost"
          }
          onClick={() => {
            setCurrentSettingToggle(SettingsToggle.Audio);
          }}
          className="justify-start items-center"
        >
          <AudioLines />
          Audio
        </Button>

        <Button
          variant={
            currentSettingToggle === SettingsToggle.Username
              ? "secondary"
              : "ghost"
          }
          onClick={() => {
            setCurrentSettingToggle(SettingsToggle.Username);
          }}
          className="justify-start items-center"
        >
          <UserRoundPen />
          User
        </Button>
      </div>

      <div
        className="flex flex-col grow-2 gap-3 py-2 px-4 bg-black rounded-lg w-full overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
        id="settings_body"
      >
        {currentSettingToggle === SettingsToggle.Room && <RoomSettings />}
        {currentSettingToggle === SettingsToggle.Streetview && (
          <StreetviewSettings />
        )}
        {currentSettingToggle === SettingsToggle.Username && (
          <UsernameSettings
            handleColorRequest={handleColorRequest}
            handleUsernameRequest={handleUsernameRequest}
          />
        )}
        {currentSettingToggle === SettingsToggle.Audio && <AudioSettings />}
      </div>
    </div>
  );
};

export default Settings;
