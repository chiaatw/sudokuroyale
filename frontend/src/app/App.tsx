import { LoginRegisterPage } from '@/app/components/LoginRegisterPage';
import { MatchLobby } from '@/app/components/MatchLobby';
import { JoinMatch } from '@/app/components/JoinMatch';
import { WaitingPage } from '@/app/components/WaitingPage';
import { GamePage } from '@/app/components/GamePage';
import { ResultPage } from '@/app/components/ResultPage';

export default function App() {
  return (
    <div className="size-full">
      {/* <LoginRegisterPage /> */}
      {/* <MatchLobby /> */}
      {/* <JoinMatch /> */}
      {/* <WaitingPage /> */}
      <GamePage />
      {/* <ResultPage /> */}
    </div>
  );
}