import { useState } from 'react';
import { ArrowLeft, LogIn } from 'lucide-react';

interface JoinMatchProps {
  onBack?: () => void;
}

export function JoinMatch({ onBack }: JoinMatchProps) {
  const [matchCode, setMatchCode] = useState('');

  const handleJoin = (e: React.FormEvent) => {
    e.preventDefault();
    console.log('Beitritt zu Match:', matchCode);
    alert(`Trete Match ${matchCode} bei...`);
    // Logic to join the match with the code
  };

  const handleBack = () => {
    if (onBack) {
      onBack();
    } else {
      console.log('Zurück zur Lobby');
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-cyan-900 flex items-center justify-center p-4">
      <div className="w-full max-w-2xl">
        {/* Logo/Title */}
        <div className="text-center mb-8">
          <h1 className="text-5xl font-bold text-white mb-2 tracking-tight">
            Sudokuroyale
          </h1>
          <p className="text-cyan-200 text-lg">Match beitreten</p>
        </div>

        {/* Main Card */}
        <div className="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
          <h2 className="text-2xl font-bold text-white mb-6 text-center">
            Match-Code eingeben
          </h2>

          <form onSubmit={handleJoin} className="space-y-6">
            {/* Match Code Input */}
            <div>
              <label className="block text-white/90 mb-3 text-sm font-medium">
                Match-Code
              </label>
              <input
                type="text"
                value={matchCode}
                onChange={(e) => setMatchCode(e.target.value.toUpperCase())}
                placeholder="z.B. ABC123"
                required
                className="w-full bg-white/10 border border-white/30 rounded-xl py-6 px-6 text-white placeholder-white/40 text-center text-2xl font-bold tracking-widest focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
                maxLength={10}
              />
              <p className="mt-2 text-white/50 text-sm text-center">
                Gib den Code ein, den du von deinem Gegner erhalten hast
              </p>
            </div>

            {/* Join Button */}
            <button
              type="submit"
              className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-600 hover:to-blue-700 text-white font-bold py-5 px-8 rounded-xl shadow-lg hover:shadow-xl transition-all transform hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-3"
            >
              <LogIn className="w-6 h-6" />
              <span className="text-xl">Beitreten</span>
            </button>
          </form>

          {/* Back Button */}
          <button
            onClick={handleBack}
            className="w-full mt-4 bg-white/5 hover:bg-white/10 border border-white/20 hover:border-white/30 text-white/80 hover:text-white font-semibold py-4 px-8 rounded-xl transition-all flex items-center justify-center gap-2"
          >
            <ArrowLeft className="w-5 h-5" />
            <span>Zurück zur Lobby</span>
          </button>
        </div>

        {/* Additional Info */}
        <div className="mt-6 text-center text-white/70 text-sm">
          <p>Sobald du beigetreten bist, beginnt das Battle!</p>
        </div>
      </div>
    </div>
  );
}
