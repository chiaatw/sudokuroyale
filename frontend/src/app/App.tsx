import { Routes, Route, Navigate } from "react-router-dom";

import { LoginRegisterPage } from "./components/LoginRegisterPage";
import { MatchLobby } from "./components/MatchLobby";
import { JoinMatch } from "./components/JoinMatch";
import { WaitingPage } from "./components/WaitingPage";
import { GamePage } from "./components/GamePage";
import { ResultPage } from "./components/ResultPage";
import { ResultPageLoser } from "./components/ResultPageLoser";

export default function App() {
  return (
    <div className="size-full">
      <Routes>
        {/* Start */}
        <Route path="/" element={<Navigate to="/login" replace />} />

        {/* Pages */}
        <Route path="/login" element={<LoginRegisterPage />} />
        <Route path="/lobby" element={<MatchLobby />} />
        <Route path="/join" element={<JoinMatch />} />
        <Route path="/waiting" element={<WaitingPage />} />
        <Route path="/game" element={<GamePage />} />
        <Route path="/result/win" element={<ResultPage />} />
        <Route path="/result/lose" element={<ResultPageLoser />} />

        {/* Fallback */}
        <Route path="*" element={<Navigate to="/login" replace />} />
      </Routes>
    </div>
  );
}