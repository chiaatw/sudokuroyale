import { useState } from "react";
import { ArrowLeft, LogIn } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function JoinMatch() {
  const navigate = useNavigate();
  const [matchCode, setMatchCode] = useState("");

  const handleJoin = (e: React.FormEvent) => {
    e.preventDefault();
    console.log("Beitritt zu Match:", matchCode);
    alert(`Trete Match ${matchCode} bei...`);
    navigate("/game");
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-cyan-900 flex items-center justify-center p-4">
      <div className="w-full max-w-2xl">
        <div className="text-center mb-8">
          <h1 className="text-5xl font-bold text-white mb-2 tracking-tight">
            Sudokuroyale
          </h1>
          <p className="text-cyan-200 text-lg">Match beitreten</p>
        </div>

        <div className="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
          <h2 className="text-2xl font-bold text-white mb-6 text-center">
            Match-Code eingeben
          </h2>

          <form onSubmit={handleJoin} className="space-y-6">
            <div>
              <label className="block text-white/90 mb-3 text-sm font-medium">
                Match-Code
              </label>
              <input
                type="text"
                value={matchCode}
                onChange={(e) =>
                  setMatchCode(e.target.value.toUpperCase())
                }
                placeholder="z.B. ABC123"
                required
                maxLength={10}
                className="w-full bg-white/10 border border-white/30 rounded-xl py-6 px-6 text-white text-center text-2xl font-bold tracking-widest"
              />
            </div>

            <button
              type="submit"
              className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-bold py-5 rounded-xl flex items-center justify-center gap-3"
            >
              <LogIn className="w-6 h-6" />
              Beitreten
            </button>
          </form>

          <button
            onClick={() => navigate("/lobby")}
            className="w-full mt-4 bg-white/5 border border-white/20 text-white py-4 rounded-xl flex items-center justify-center gap-2"
          >
            <ArrowLeft className="w-5 h-5" />
            Zurück zur Lobby
          </button>
        </div>
      </div>
    </div>
  );
}