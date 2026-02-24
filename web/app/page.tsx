import { GameView } from "@/components/game/GameView"
import { Suspense } from "react"


export default function Landing() {


  return (
    <main className="m-0 p-0">
      <Suspense>
        <GameView />
      </Suspense>
    </main>
  )
}


// location
// continuing
// score
// distance
// real-coordinates
