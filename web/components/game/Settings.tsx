import { memo } from "react";
import { Toggle } from "@/components/ui/toggle"
import { Separator } from "../ui/separator";
import { KeyRound, MapPinPen, AudioLines } from "lucide-react";

const Settings = memo(({ room_code }: { room_code: string }) => {
  return (
    <div className="z-[99] w-[50vw] h-[50vh] flex flex-row bg-black border-2 py-5 rounded-lg opacity-90">
      <div className="text-white flex flex-col grow-1 gap-2 p-2" id="side-bar">
        <Toggle className="justify-start items-center">
          <KeyRound />
          Room Code
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
      <Separator orientation="vertical" />
      <div className="flex flex-col grow-2 gap-2 px-2" id="settings_body">
        <p className="text-white">Room Code</p>
      </div>
      <div>

      </div>

    </div>
  );
});

export default Settings;
