import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

// Zustand
import { useShallow } from "zustand/shallow";
import { audioStateActions } from "@/store/useSettingsStore";
import { useAudioState } from "@/store/useSettingsStore";
import { settingsStateActions } from "@/store/useSettingsStore";

// ui
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";

const AudioSettings = () => {
  const { markerAudio, lockAudio } = useAudioState(
    useShallow((state) => ({
      markerAudio: state.markerAudioEnabled,
      lockAudio: state.lockAudioEnabled,
    })),
  );

  return (
    <>
      <div className="flex flex-row justify-between items-center">
        <p className="text-xl font-semibold">Audio</p>
        <Button
          className="bg-transparent hover:cursor-pointer"
          onClick={() => settingsStateActions.updateSettingsVisibility(false)}
          variant={"secondary"}
        >
          <X color="white" />
        </Button>
      </div>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="panningGesturesEnabled">Marker Audio</Label>
        <Switch
          checked={markerAudio}
          onCheckedChange={audioStateActions.setMarkerAudioEnabled}
          id="markerAudioEnabled"
        />
      </div>
      <Separator />

      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="streetNamesEnabled">Lock Audio</Label>
        <Switch
          checked={lockAudio}
          onCheckedChange={audioStateActions.setLockAudioEnabled}
          id="lockAudioEnabled"
        />
      </div>
      <Separator />
    </>
  );
};

export default AudioSettings;
