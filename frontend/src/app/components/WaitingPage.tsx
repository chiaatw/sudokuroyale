import { useState } from 'react';
import { Copy, X, Loader2 } from 'lucide-react';

interface WaitingPageProps {
  onLeave: () => void;
  onStartGame: () => void;
}

export function WaitingPage({ onLeave, onStartGame }: WaitingPageProps) {
  const [matchCode] = useState('XYZ789'); // Mock match code
  const [copied, setCopied] = useState(false);

  const handleCopyCode = () => {
    navigator.clipboard.writeText(matchCode);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleLeave = () => {
    if (onLeave) {
      onLeave();
    } else {
      console.log('Match verlassen');
      alert('Match verlassen');
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
          <p className="text-cyan-200 text-lg">Match wird vorbereitet...</p>
        </div>

        {/* Main Card */}
        <div className="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
          {/* Loading Animation */}
          <div className="flex justify-center mb-6">
            <Loader2 className="w-16 h-16 text-cyan-400 animate-spin" />
          </div>

          {/* Status Text */}
          <h2 className="text-2xl font-bold text-white mb-8 text-center">
            Warten auf zweiten Spieler...
          </h2>

          {/* Match Code Display */}
          <div className="mb-8">
            <label className="block text-white/90 mb-3 text-sm font-medium text-center">
              Teile diesen Match-Code
            </label>
            <div className="bg-white/10 border-2 border-cyan-400/50 rounded-xl p-6 text-center">
              <div className="text-5xl font-bold text-white tracking-widest mb-4">
                {matchCode}
              </div>
              <button
                onClick={handleCopyCode}
                className="inline-flex items-center gap-2 bg-cyan-500/20 hover:bg-cyan-500/30 border border-cyan-400/50 text-cyan-200 font-semibold py-2 px-4 rounded-lg transition-all"
              >
                <Copy className="w-4 h-4" />
                {copied ? 'Kopiert!' : 'Code kopieren'}
              </button>
            </div>
            <p className="mt-3 text-white/50 text-sm text-center">
              Sende den Code an deinen Gegner
            </p>
          </div>

          {/* Waiting Indicator */}
          <div className="flex items-center justify-center gap-2 mb-6">
            <div className="flex gap-1">
              <div className="w-2 h-2 bg-cyan-400 rounded-full animate-pulse" style={{ animationDelay: '0ms' }}></div>
              <div className="w-2 h-2 bg-cyan-400 rounded-full animate-pulse" style={{ animationDelay: '150ms' }}></div>
              <div className="w-2 h-2 bg-cyan-400 rounded-full animate-pulse" style={{ animationDelay: '300ms' }}></div>
            </div>
          </div>

          {/* Leave Match Button */}
          <button
            onClick={onLeave}
            className="w-full bg-red-500/20 hover:bg-red-500/30 border border-red-400/50 text-red-200 hover:text-red-100 font-semibold py-4 px-8 rounded-xl transition-all flex items-center justify-center gap-2"
          >
            <X className="w-5 h-5" />
            <span>Match verlassen</span>
          </button>
        </div>

        {/* Test Button */}
        <button
        onClick={onStartGame}
        className="w-full mt-4 bg-white/10 hover:bg-white/20 border border-white/20 text-white font-semibold py-3 px-8 rounded-xl transition-all"
      > 
        (Demo) Spiel starten
      </button>

        {/* Additional Info */}
        <div className="mt-6 text-center text-white/70 text-sm">
          <p>Das Spiel startet automatisch, sobald der zweite Spieler beitritt</p>
        </div>
      </div>
    </div>
  );
}
