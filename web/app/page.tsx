import { Suspense } from "react"
import { GameView } from "@/components/game/GameView"

export default function Landing() {
  return (
    <main className="m-0 p-0">
      <Suspense>
        <GameView />
      </Suspense>
    </main>
  )
}
