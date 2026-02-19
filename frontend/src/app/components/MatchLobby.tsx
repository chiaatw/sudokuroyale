import { Plus, Search } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { createMatch } from "../api/match";


export function MatchLobby() {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-cyan-900 flex items-center justify-center p-4">
      <div className="w-full max-w-2xl">
        <div className="text-center mb-8">
          <h1 className="text-5xl font-bold text-white mb-2 tracking-tight">
            Sudokuroyale
          </h1>
          <p className="text-cyan-200 text-lg">Bereit für den Battle?</p>
        </div>

        <div className="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
          <h2 className="text-2xl font-bold text-white mb-6 text-center">
            Match-Lobby
          </h2>

          <div className="space-y-4">
            <button
              onClick={async () => {
                try {
                  const matchId = await createMatch();
                  navigate(`/waiting?matchId=${matchId}`);
                } catch {
                  alert("Konnte kein Match erstellen. Bist du eingeloggt?");
                }
              }}
              className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-600 hover:to-blue-700 text-white font-bold py-6 px-8 rounded-xl shadow-lg hover:shadow-xl transition-all transform hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-3 group"
            >
              <Plus className="w-6 h-6 group-hover:rotate-90 transition-transform" />
              <span className="text-xl">Match erstellen</span>
            </button>

            <button
              onClick={() => navigate("/join")}
              className="w-full bg-white/10 hover:bg-white/20 border-2 border-white/30 hover:border-white/50 text-white font-bold py-6 px-8 rounded-xl shadow-lg hover:shadow-xl transition-all transform hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-3 group"
            >
              <Search className="w-6 h-6 group-hover:scale-110 transition-transform" />
              <span className="text-xl">Match beitreten</span>
            </button>
          </div>

          <div className="mt-8 text-center text-white/60 text-sm">
            <p>Erstelle ein neues Match oder tritt einem bestehenden bei</p>
          </div>
        </div>

        <div className="mt-6 text-center text-white/70 text-sm">
          <p>Zeige deine Sudoku-Fähigkeiten im direkten Duell!</p>
        </div>
      </div>
    </div>
  );
}