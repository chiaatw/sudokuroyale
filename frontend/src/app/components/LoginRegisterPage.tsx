import { useState } from 'react';
import { User, Lock, Mail, UserPlus, LogIn } from 'lucide-react';
import { useNavigate } from "react-router-dom";
import { login, register } from '../api/auth';

export function LoginRegisterPage() {
  const [isLogin, setIsLogin] = useState(true);
  const [email, setEmail] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
  
    try {
      if (isLogin) {
        await login({ username, password });
        navigate("/lobby");
      } else {
        await register({ username, email, password });
        alert("Registrierung erfolgreich. Bitte einloggen.");
        setIsLogin(true);
      }
    } catch (err) {
      console.error(err);
      alert(
        (err as Error)?.message
          ? (err as Error).message
          : (isLogin ? "Login fehlgeschlagen" : "Registrierung fehlgeschlagen")
      );
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-cyan-900 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo/Title */}
        <div className="text-center mb-8">
          <h1 className="text-5xl font-bold text-white mb-2 tracking-tight">
            Sudokuroyale
          </h1>
          <p className="text-cyan-200 text-lg">1 vs 1 Sudoku Battle</p>
        </div>

        {/* Main Card */}
        <div className="bg-white/10 backdrop-blur-lg rounded-2xl shadow-2xl p-8 border border-white/20">
          {/* Toggle Buttons */}
          <div className="flex gap-2 mb-6 bg-black/20 rounded-lg p-1">
            <button
              onClick={() => {
                setIsLogin(true);
                setEmail("");
                setUsername("");
                setPassword("");
              }}
              className={`flex-1 py-3 rounded-lg font-semibold transition-all ${
                isLogin
                  ? 'bg-white text-blue-900 shadow-lg'
                  : 'text-white/70 hover:text-white'
              }`}
            >
              <LogIn className="inline-block w-5 h-5 mr-2 mb-1" />
              Login
            </button>
            <button
              onClick={() => {
                setIsLogin(false);
                setEmail("");
                setUsername("");
                setPassword("");
              }}
              className={`flex-1 py-3 rounded-lg font-semibold transition-all ${
                !isLogin
                  ? 'bg-white text-blue-900 shadow-lg'
                  : 'text-white/70 hover:text-white'
              }`}
            >
              <UserPlus className="inline-block w-5 h-5 mr-2 mb-1" />
              Registrieren
            </button>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            
            <div>
              <label className="block text-white/90 mb-2 text-sm font-medium">
                 {isLogin ? "Benutzername" : "E-Mail"}
              </label>

              <div className="relative">
                {isLogin ? (
                   <User className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-white/40" />
                ) : (
                     <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-white/40" />
                )}

                 <input
                  type={isLogin ? "text" : "email"}
                  value={isLogin ? username : email}
                  onChange={(e) => (isLogin ? setUsername(e.target.value) : setEmail(e.target.value))}
                  placeholder={isLogin ? "deinUsername" : "deine@email.com"}
                  required
                  className="w-full bg-white/10 border border-white/30 rounded-lg py-3 px-10 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
                />
              </div>
            </div>

            {/* Username Field (only for register) */}
            {!isLogin && (
              <div>
                <label className="block text-white/90 mb-2 text-sm font-medium">
                  Benutzername
                </label>
                <div className="relative">
                  <User className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-white/40" />
                  <input
                    type="text"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    placeholder="Dein Spielername"
                    required={!isLogin}
                    className="w-full bg-white/10 border border-white/30 rounded-lg py-3 px-10 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
                  />
                </div>
              </div>
            )}

            {/* Password Field */}
            <div>
              <label className="block text-white/90 mb-2 text-sm font-medium">
                Passwort
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-white/40" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  required
                  className="w-full bg-white/10 border border-white/30 rounded-lg py-3 px-10 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all"
                />
              </div>
            </div>

            {/* Submit Button */}
            <button
              type="submit"
              className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-600 hover:to-blue-700 text-white font-bold py-4 rounded-lg shadow-lg hover:shadow-xl transition-all transform hover:scale-[1.02] active:scale-[0.98] mt-6"
            >
              {isLogin ? 'Jetzt einloggen' : 'Jetzt registrieren'}
            </button>
          </form>

          {/* Footer */}
          <div className="mt-6 text-center text-white/60 text-sm">
            {isLogin ? (
              <p>
                Noch kein Account?{' '}
                <button
                  onClick={() => setIsLogin(false)}
                  className="text-cyan-300 hover:text-cyan-200 font-semibold"
                >
                  Registrieren
                </button>
              </p>
            ) : (
              <p>
                Schon ein Account?{' '}
                <button
                  onClick={() => setIsLogin(true)}
                  className="text-cyan-300 hover:text-cyan-200 font-semibold"
                >
                  Einloggen
                </button>
              </p>
            )}
          </div>
        </div>

        {/* Additional Info */}
        <div className="mt-6 text-center text-white/70 text-sm">
          <p>Fordere Freunde heraus und werde zum Sudoku-Champion!</p>
        </div>
      </div>
    </div>
  );
}