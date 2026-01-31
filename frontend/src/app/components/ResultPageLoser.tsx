import { Clock, X, ArrowLeft, Frown } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function ResultPageLoser() {
  const navigate = useNavigate();

  // Mock data
  const winner = {
    name: "Gegner",
    time: "05:42",
    errors: 2,
  };

  const loser = {
    name: "Du",
    time: "06:15",
    errors: 4,
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-cyan-900 flex items-center justify-center p-4 relative overflow-hidden">
      {/* Animated background elements */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-10 left-10 w-20 h-20 bg-cyan-400/20 rounded-full animate-pulse"></div>
        <div
          className="absolute top-40 right-20 w-32 h-32 bg-blue-400/20 rounded-full animate-pulse"
          style={{ animationDelay: "1s" }}
        ></div>
        <div
          className="absolute bottom-20 left-1/4 w-24 h-24 bg-cyan-300/20 rounded-full animate-pulse"
          style={{ animationDelay: "2s" }}
        ></div>
        <div
          className="absolute bottom-40 right-1/3 w-16 h-16 bg-blue-300/20 rounded-full animate-pulse"
          style={{ animationDelay: "1.5s" }}
        ></div>
      </div>

      <div className="w-full max-w-2xl relative z-10">
        {/* Logo/Title */}
        <div className="text-center mb-8">
          <h1 className="text-5xl font-bold text-white mb-2 tracking-tight">
            Sudokuroyale
          </h1>
          <p className="text-cyan-200 text-lg">Battle beendet</p>
        </div>

        {/* Main Card */}
        <div className="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
          {/* Sad Face Icon */}
          <div className="flex justify-center mb-6">
            <div className="bg-gradient-to-br from-gray-500 to-gray-700 rounded-full p-6 shadow-lg">
              <Frown className="w-16 h-16 text-white" />
            </div>
          </div>

          {/* Loser Announcement */}
          <div className="text-center mb-8">
            <h2 className="text-4xl font-bold text-white mb-2">Niederlage</h2>
            <p className="text-xl text-cyan-200">{winner.name} hat gewonnen</p>
          </div>

          {/* Results Grid */}
          <div className="grid grid-cols-2 gap-4 mb-8">
            {/* Winner Stats */}
            <div className="bg-white/10 rounded-xl p-4 border-2 border-yellow-400/50">
              <div className="text-center">
                <div className="text-lg font-bold text-white mb-3">
                  {winner.name}
                </div>
                <div className="space-y-2">
                  <div className="flex items-center justify-center gap-2 text-cyan-200">
                    <Clock className="w-4 h-4" />
                    <span className="text-sm">{winner.time}</span>
                  </div>
                  <div className="flex items-center justify-center gap-2 text-red-300">
                    <X className="w-4 h-4" />
                    <span className="text-sm">{winner.errors} Fehler</span>
                  </div>
                </div>
                <div className="mt-2 text-yellow-400 text-xs font-semibold">
                  GEWINNER
                </div>
              </div>
            </div>

            {/* Loser Stats */}
            <div className="bg-white/10 rounded-xl p-4 border border-white/20">
              <div className="text-center">
                <div className="text-lg font-bold text-white mb-3">
                  {loser.name}
                </div>
                <div className="space-y-2">
                  <div className="flex items-center justify-center gap-2 text-cyan-200">
                    <Clock className="w-4 h-4" />
                    <span className="text-sm">{loser.time}</span>
                  </div>
                  <div className="flex items-center justify-center gap-2 text-red-300">
                    <X className="w-4 h-4" />
                    <span className="text-sm">{loser.errors} Fehler</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Motivational Text */}
          <div className="text-center mb-6">
            <p className="text-white/70 text-sm">
              Gut gespielt! Versuch es noch einmal und werde besser!
            </p>
          </div>

          {/* Back to Lobby Button */}
          <button
            onClick={() => navigate("/lobby")}
            className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-600 hover:to-blue-700 text-white font-bold py-5 px-8 rounded-xl shadow-lg hover:shadow-xl transition-all transform hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-3"
          >
            <ArrowLeft className="w-5 h-5" />
            <span className="text-xl">Zur Lobby zurück</span>
          </button>
        </div>
      </div>
    </div>
  );
}
